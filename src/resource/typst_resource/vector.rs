use crate::frontend::collection::primitives::path::{Path, PathFillRule};
use crate::frontend::style::Style;
use crate::projection::style::ColorSource;
use anyhow::Result;
use glam::{Vec2, Vec4, vec2};
use resvg::tiny_skia;
use resvg::usvg;

/// Represents a single vectorized character or symbol from a Typst/SVG output.
pub struct VectorSymbol {
    pub key: String,
    pub path: Path,
    pub center: Vec2,
}

/// Parses an SVG string into a collection of morphable Path objects.
pub fn parse_svg_to_paths(svg_str: &str, color: Vec4) -> Result<Vec<VectorSymbol>> {
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_str(svg_str, &opt)?;
    let size = tree.size();

    let mut symbols = Vec::new();
    let mut char_count = 0;

    let half_w = size.width() as f32 * 0.5;
    let half_h = size.height() as f32 * 0.5;

    // Start traversal from the root group
    traverse_group(
        tree.root(),
        &mut symbols,
        &mut char_count,
        half_w,
        half_h,
        color,
    );

    Ok(symbols)
}

/// Backward-compatible alias for Typst-produced SVGs.
pub fn parse_typst_svg_to_paths(svg_str: &str, color: Vec4) -> Result<Vec<VectorSymbol>> {
    parse_svg_to_paths(svg_str, color)
}

fn traverse_group(
    group: &usvg::Group,
    symbols: &mut Vec<VectorSymbol>,
    char_count: &mut usize,
    half_w: f32,
    half_h: f32,
    color: Vec4,
) {
    for node in group.children() {
        match node {
            usvg::Node::Path(usvg_path) => {
                let mut path = Path::new();
                // `abs_transform()` already includes the full ancestor transform chain.
                // Re-applying parent/group transforms here distorts glyph geometry and can
                // produce mirrored or displaced fragments before morphing even begins.
                let transform = usvg_path.abs_transform();

                for segment in usvg_path.data().segments() {
                    match segment {
                        tiny_skia::PathSegment::MoveTo(p) => {
                            let p =
                                transform_point(p.x as f64, p.y as f64, transform, half_w, half_h);
                            path = path.move_to(p);
                        }
                        tiny_skia::PathSegment::LineTo(p) => {
                            let p =
                                transform_point(p.x as f64, p.y as f64, transform, half_w, half_h);
                            path = path.line_to(p);
                        }
                        tiny_skia::PathSegment::QuadTo(p1, p) => {
                            let c = transform_point(
                                p1.x as f64,
                                p1.y as f64,
                                transform,
                                half_w,
                                half_h,
                            );
                            let p =
                                transform_point(p.x as f64, p.y as f64, transform, half_w, half_h);
                            path = path.quad_to(c, p);
                        }
                        tiny_skia::PathSegment::CubicTo(p1, p2, p) => {
                            let c1 = transform_point(
                                p1.x as f64,
                                p1.y as f64,
                                transform,
                                half_w,
                                half_h,
                            );
                            let c2 = transform_point(
                                p2.x as f64,
                                p2.y as f64,
                                transform,
                                half_w,
                                half_h,
                            );
                            let p =
                                transform_point(p.x as f64, p.y as f64, transform, half_w, half_h);
                            path = path.cubic_to(c1, c2, p);
                        }
                        tiny_skia::PathSegment::Close => {
                            path = path.close();
                        }
                    }
                }

                path.style = Style {
                    fill: Some(ColorSource::Solid(color)),
                    stroke: None,
                };
                path.fill_rule = match usvg_path.fill().map(|fill| fill.rule()) {
                    Some(usvg::FillRule::EvenOdd) => PathFillRule::EvenOdd,
                    _ => PathFillRule::NonZero,
                };

                let key = usvg_path.id().to_string();
                let key = if key.is_empty() {
                    *char_count += 1;
                    format!("glyph_{}", *char_count)
                } else {
                    key
                };

                // Calculate center
                let mut min = vec2(f32::INFINITY, f32::INFINITY);
                let mut max = vec2(f32::NEG_INFINITY, f32::NEG_INFINITY);
                for seg in &path.segments {
                    let p = seg.end_point();
                    min.x = min.x.min(p.x);
                    min.y = min.y.min(p.y);
                    max.x = max.x.max(p.x);
                    max.y = max.y.max(p.y);
                }
                let center = (min + max) * 0.5;

                symbols.push(VectorSymbol { key, path, center });
            }
            usvg::Node::Group(g) => {
                traverse_group(g, symbols, char_count, half_w, half_h, color);
            }
            _ => {}
        }
    }
}

fn transform_point(x: f64, y: f64, ts: usvg::Transform, half_w: f32, half_h: f32) -> Vec2 {
    let mut p = tiny_skia::Point::from_xy(x as f32, y as f32);
    ts.map_point(&mut p);

    // Convert to Murali world space (Y-up, centered)
    vec2(p.x - half_w, -(p.y - half_h))
}
