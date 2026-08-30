use crate::frontend::collection::primitives::path::{Path, PathSegment};
use crate::math::bezier::{cubic_bezier, quadratic_bezier};
use crate::resource::typst_resource::compiler::TypstBackend;
use crate::resource::typst_resource::vector::{VectorSymbol, parse_svg_to_paths, scale_symbols};
use glam::{Vec2, Vec4};

const SAMPLE_COLOR: Vec4 = Vec4::ONE;
const VECTOR_BASE_SIZE: f32 = 32.0;

/// Sample a Typst formula into a centered world-space polyline.
pub fn typst_outline_points(
    typst_formula: &str,
    world_height: f32,
    target_points: usize,
) -> anyhow::Result<Vec<Vec2>> {
    let backend = TypstBackend::new()?;
    let base_size = 32.0;
    let svg = backend.render_to_svg(typst_formula, base_size)?;
    let symbols = parse_svg_to_paths(&svg, SAMPLE_COLOR)?;

    let mut sampled = Vec::new();
    let point_budget_per_symbol = (target_points / symbols.len().max(1)).max(24);
    let world_scale = world_height / base_size;

    for symbol in symbols {
        let mut symbol_points = sample_path(&symbol.path, point_budget_per_symbol);
        for point in &mut symbol_points {
            *point *= world_scale;
        }
        sampled.extend(symbol_points);
    }

    if sampled.len() < 3 {
        anyhow::bail!("formula sampling produced too few points");
    }

    let center = sampled
        .iter()
        .copied()
        .fold(Vec2::ZERO, |acc, point| acc + point)
        / sampled.len() as f32;
    for point in &mut sampled {
        *point -= center;
    }

    Ok(resample_polyline(&sampled, target_points))
}

/// Compile a Typst formula into filled, world-scaled path glyphs.
pub fn typst_vector_paths(
    source: &str,
    world_height: f32,
    color: Vec4,
) -> anyhow::Result<Vec<VectorSymbol>> {
    if source.trim().is_empty() {
        anyhow::bail!("source must not be empty");
    }
    if world_height <= 0.0 || !world_height.is_finite() {
        anyhow::bail!("height must be a positive finite number");
    }
    let backend = TypstBackend::new()?;
    let svg = backend.render_to_svg(source, VECTOR_BASE_SIZE)?;
    let mut symbols = parse_svg_to_paths(&svg, color)?;
    if symbols.is_empty() {
        anyhow::bail!("formula vectorization produced no paths");
    }
    scale_symbols(&mut symbols, world_height / VECTOR_BASE_SIZE);
    Ok(symbols)
}

fn sample_path(path: &Path, samples_per_segment: usize) -> Vec<Vec2> {
    let mut points = Vec::new();
    let mut start = Vec2::ZERO;
    let mut current = Vec2::ZERO;

    for segment in &path.segments {
        match *segment {
            PathSegment::MoveTo(point) => {
                start = point;
                current = point;
                points.push(point);
            }
            PathSegment::LineTo(point) => {
                points.push(point);
                current = point;
            }
            PathSegment::QuadTo(control, end) => {
                for step in 1..=samples_per_segment {
                    let t = step as f32 / samples_per_segment as f32;
                    points.push(quadratic_bezier(current, control, end, t));
                }
                current = end;
            }
            PathSegment::CubicTo(control1, control2, end) => {
                for step in 1..=samples_per_segment {
                    let t = step as f32 / samples_per_segment as f32;
                    points.push(cubic_bezier(current, control1, control2, end, t));
                }
                current = end;
            }
        }
    }

    if path.closed && current.distance(start) > 0.001 {
        points.push(start);
    }

    points
}

pub fn resample_polyline(points: &[Vec2], target_count: usize) -> Vec<Vec2> {
    if points.len() <= 1 || target_count <= 1 {
        return points.to_vec();
    }

    let mut lengths = Vec::with_capacity(points.len());
    lengths.push(0.0);
    let mut total_length = 0.0;
    for pair in points.windows(2) {
        total_length += pair[0].distance(pair[1]);
        lengths.push(total_length);
    }

    if total_length <= f32::EPSILON {
        return points.to_vec();
    }

    let mut resampled = Vec::with_capacity(target_count);
    for index in 0..target_count {
        let target = total_length * index as f32 / (target_count - 1) as f32;
        let segment = lengths.partition_point(|length| *length < target);
        let upper = segment.min(points.len() - 1);
        let lower = upper.saturating_sub(1);
        let span = (lengths[upper] - lengths[lower]).max(f32::EPSILON);
        let local_t = (target - lengths[lower]) / span;
        resampled.push(points[lower].lerp(points[upper], local_t));
    }

    resampled
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::vec2;

    #[test]
    fn resample_polyline_keeps_endpoints() {
        let points = [vec2(0.0, 0.0), vec2(2.0, 0.0), vec2(2.0, 2.0)];
        let resampled = resample_polyline(&points, 5);
        assert_eq!(resampled.len(), 5);
        assert!(resampled[0].abs_diff_eq(vec2(0.0, 0.0), 1e-5));
        assert!(resampled[4].abs_diff_eq(vec2(2.0, 2.0), 1e-5));
    }

    #[test]
    fn typst_vector_paths_emits_scaled_glyphs() {
        let symbols = typst_vector_paths("$x$", 1.0, Vec4::ONE).expect("typst vector paths");
        assert!(!symbols.is_empty());
        assert!(
            symbols
                .iter()
                .all(|symbol| !symbol.path.segments.is_empty())
        );
    }
}
