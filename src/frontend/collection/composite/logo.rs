use crate::frontend::collection::primitives::circle::Circle;
use crate::frontend::collection::primitives::ellipse::Ellipse;
use crate::frontend::collection::primitives::line::Line;
use crate::frontend::collection::primitives::path::Path;
use crate::frontend::collection::primitives::rectangle::Rectangle;
use crate::frontend::layout::{Bounded, Bounds};
use crate::frontend::style::{StrokeParams, Style};
use crate::projection::{Project, ProjectionCtx};
use glam::{Vec3, Vec4, vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MuraliLogoPalette {
    Light,
    Dark,
}

#[derive(Debug, Clone)]
pub struct MuraliLogoMark {
    pub scale: f32,
    pub flute_offset_y: f32,
    pub palette: MuraliLogoPalette,
}

impl Default for MuraliLogoMark {
    fn default() -> Self {
        Self::new()
    }
}

impl MuraliLogoMark {
    pub fn new() -> Self {
        Self {
            scale: 1.0,
            flute_offset_y: -0.40,
            palette: MuraliLogoPalette::Dark,
        }
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    pub fn with_flute_offset_y(mut self, flute_offset_y: f32) -> Self {
        self.flute_offset_y = flute_offset_y;
        self
    }

    pub fn with_palette(mut self, palette: MuraliLogoPalette) -> Self {
        self.palette = palette;
        self
    }
}

impl Project for MuraliLogoMark {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let (
            flute_body,
            flute_edge,
            hole,
            accent,
            teal,
            cyan,
            blue,
            green,
            amber,
            faint_graph,
            graph_axis,
            spine_glow,
            eye_sheen,
        ) = match self.palette {
            MuraliLogoPalette::Light => (
                Vec4::new(0.71, 0.52, 0.18, 1.0),
                Vec4::new(0.36, 0.22, 0.04, 1.0),
                Vec4::new(0.12, 0.08, 0.03, 1.0),
                Vec4::new(0.95, 0.78, 0.34, 1.0),
                Vec4::new(0.00, 0.50, 0.48, 1.0),
                Vec4::new(0.04, 0.56, 0.84, 1.0),
                Vec4::new(0.11, 0.29, 0.70, 1.0),
                Vec4::new(0.10, 0.49, 0.28, 1.0),
                Vec4::new(0.83, 0.52, 0.08, 1.0),
                Vec4::new(0.08, 0.34, 0.52, 0.14),
                Vec4::new(0.08, 0.44, 0.68, 0.22),
                Vec4::new(0.55, 0.77, 0.78, 0.72),
                Vec4::new(0.10, 0.48, 0.76, 0.32),
            ),
            MuraliLogoPalette::Dark => (
                Vec4::new(0.96, 0.79, 0.44, 1.0),
                Vec4::new(0.79, 0.56, 0.18, 1.0),
                Vec4::new(0.24, 0.18, 0.08, 1.0),
                Vec4::new(1.00, 0.92, 0.68, 1.0),
                Vec4::new(0.28, 0.94, 0.86, 1.0),
                Vec4::new(0.48, 0.92, 1.00, 1.0),
                Vec4::new(0.43, 0.66, 1.00, 1.0),
                Vec4::new(0.40, 0.90, 0.50, 1.0),
                Vec4::new(1.00, 0.83, 0.30, 1.0),
                Vec4::new(0.44, 0.88, 0.98, 0.30),
                Vec4::new(0.50, 0.94, 1.00, 0.44),
                Vec4::new(0.88, 1.00, 0.98, 0.98),
                Vec4::new(0.46, 0.94, 1.00, 0.62),
            ),
        };

        let body_style = Style::new()
            .with_fill(flute_body)
            .with_stroke(StrokeParams {
                thickness: 0.03,
                color: flute_edge,
                ..Default::default()
            });

        let feather_anchor = vec2(-0.08, -2.18);
        let flute_y = -1.85 + self.flute_offset_y;

        ctx.with_scale(self.scale, |ctx| {
            project_with_offset(
                ctx,
                Rectangle::new(5.2, 0.44, flute_body).with_style(body_style.clone()),
                Vec3::new(-2.6, flute_y, 0.0),
            );
            project_with_offset(
                ctx,
                Circle::new(0.22, 36, flute_body).with_stroke(0.03, flute_edge),
                Vec3::new(-5.2, flute_y, 0.0),
            );
            project_with_offset(
                ctx,
                Circle::new(0.22, 36, flute_body).with_stroke(0.03, flute_edge),
                Vec3::new(0.0, flute_y, 0.0),
            );
            project_with_offset(
                ctx,
                Circle::new(0.10, 28, accent).with_stroke(0.02, flute_edge),
                Vec3::new(-4.45, flute_y, 0.08),
            );
            for x in [-3.45_f32, -2.60, -1.75, -0.90, -0.05] {
                project_with_offset(
                    ctx,
                    Circle::new(0.11, 28, hole)
                        .with_stroke(0.01, Vec4::new(0.05, 0.04, 0.02, 0.45)),
                    Vec3::new(x, flute_y, 0.08),
                );
            }
            project_with_offset(
                ctx,
                Rectangle::new(0.08, 0.58, accent),
                Vec3::new(-4.95, flute_y, 0.08),
            );
            project_with_offset(
                ctx,
                Rectangle::new(0.08, 0.58, accent),
                Vec3::new(-0.28, flute_y, 0.08),
            );

            let graph_left = feather_anchor.x - 1.15;
            let graph_right = feather_anchor.x + 2.55;
            let graph_bottom = feather_anchor.y - 0.02;
            let graph_top = feather_anchor.y + 5.16;

            project_with_offset(
                ctx,
                Line::new(
                    Vec3::new(graph_left, graph_bottom, 0.0),
                    Vec3::new(graph_right, graph_bottom, 0.0),
                    0.015,
                    graph_axis,
                ),
                Vec3::ZERO,
            );
            project_with_offset(
                ctx,
                Line::new(
                    Vec3::new(feather_anchor.x + 0.20, graph_bottom - 0.12, 0.0),
                    Vec3::new(feather_anchor.x + 0.20, graph_top + 0.18, 0.0),
                    0.015,
                    graph_axis,
                ),
                Vec3::ZERO,
            );
            for x in [
                graph_left + 0.85,
                feather_anchor.x + 0.95,
                feather_anchor.x + 1.75,
            ] {
                project_with_offset(
                    ctx,
                    Line::new(
                        Vec3::new(x, graph_bottom, 0.0),
                        Vec3::new(x, graph_top, 0.0),
                        0.010,
                        faint_graph,
                    ),
                    Vec3::ZERO,
                );
            }
            for y in [
                feather_anchor.y + 0.95,
                feather_anchor.y + 2.05,
                feather_anchor.y + 3.15,
                feather_anchor.y + 4.25,
            ] {
                project_with_offset(
                    ctx,
                    Line::new(
                        Vec3::new(graph_left, y, 0.0),
                        Vec3::new(graph_right, y, 0.0),
                        0.010,
                        faint_graph,
                    ),
                    Vec3::ZERO,
                );
            }

            project_with_offset(
                ctx,
                Path::new()
                    .with_thickness(0.020)
                    .with_color(Vec4::new(0.25, 0.82, 0.98, 0.30))
                    .with_dash(0.14, 0.08)
                    .move_to(vec2(-0.78, 0.35))
                    .cubic_to(vec2(-0.32, 1.94), vec2(0.70, 3.84), vec2(2.34, 4.84)),
                Vec3::new(feather_anchor.x, feather_anchor.y, 0.0),
            );

            project_with_offset(
                ctx,
                Path::new()
                    .with_thickness(0.040)
                    .with_color(spine_glow)
                    .move_to(vec2(-0.06, -0.36))
                    .quad_to(vec2(0.05, -0.18), vec2(0.16, 0.12)),
                Vec3::new(feather_anchor.x, feather_anchor.y, 0.0),
            );

            project_with_offset(
                ctx,
                Path::new()
                    .with_thickness(0.060)
                    .with_color(teal)
                    .move_to(vec2(0.16, 0.12))
                    .cubic_to(vec2(0.18, 1.10), vec2(0.42, 2.38), vec2(1.00, 3.62))
                    .cubic_to(vec2(1.34, 4.28), vec2(1.70, 4.82), vec2(1.98, 5.06)),
                Vec3::new(feather_anchor.x, feather_anchor.y, 0.0),
            );

            project_with_offset(
                ctx,
                Path::new()
                    .with_thickness(0.030)
                    .with_color(Vec4::new(0.30, 0.80, 0.96, 0.88))
                    .move_to(vec2(0.10, 0.10))
                    .cubic_to(vec2(-0.98, 1.28), vec2(-1.62, 2.88), vec2(-0.82, 4.34))
                    .cubic_to(vec2(-0.34, 5.10), vec2(0.84, 5.42), vec2(1.98, 5.06)),
                Vec3::new(feather_anchor.x, feather_anchor.y, 0.0),
            );

            project_with_offset(
                ctx,
                Path::new()
                    .with_thickness(0.030)
                    .with_color(Vec4::new(0.18, 0.74, 0.54, 0.92))
                    .move_to(vec2(0.14, 0.10))
                    .cubic_to(vec2(1.32, 1.08), vec2(2.46, 2.54), vec2(2.78, 3.98))
                    .cubic_to(vec2(2.96, 4.78), vec2(2.46, 5.22), vec2(1.98, 5.06)),
                Vec3::new(feather_anchor.x, feather_anchor.y, 0.0),
            );

            project_with_offset(
                ctx,
                Path::new()
                    .with_thickness(0.016)
                    .with_color(Vec4::new(0.32, 0.82, 0.98, 0.38))
                    .move_to(vec2(0.26, 0.56))
                    .cubic_to(vec2(-0.28, 1.64), vec2(-0.42, 2.84), vec2(0.40, 4.28)),
                Vec3::new(feather_anchor.x, feather_anchor.y, 0.0),
            );

            project_with_offset(
                ctx,
                Path::new()
                    .with_thickness(0.016)
                    .with_color(Vec4::new(0.22, 0.78, 0.62, 0.38))
                    .move_to(vec2(0.34, 0.68))
                    .cubic_to(vec2(1.12, 1.86), vec2(1.88, 2.92), vec2(2.28, 4.34)),
                Vec3::new(feather_anchor.x, feather_anchor.y, 0.0),
            );

            let left_barbs = [
                (vec2(0.22, 0.74), vec2(-0.66, 1.00), blue),
                (vec2(0.34, 1.18), vec2(-1.02, 1.66), cyan),
                (vec2(0.48, 1.78), vec2(-1.16, 2.42), blue),
                (vec2(0.66, 2.48), vec2(-1.02, 3.18), teal),
                (vec2(0.88, 3.16), vec2(-0.62, 4.00), cyan),
                (vec2(1.14, 3.88), vec2(0.10, 4.74), blue),
            ];
            for (start, end, color) in left_barbs {
                let mid = vec2(
                    (start.x + end.x) * 0.5 - 0.16,
                    (start.y + end.y) * 0.5 + 0.12,
                );
                project_with_offset(
                    ctx,
                    Path::new()
                        .with_thickness(0.026)
                        .with_color(color)
                        .move_to(vec2(0.0, 0.0))
                        .quad_to(mid - start, end - start),
                    Vec3::new(feather_anchor.x + start.x, feather_anchor.y + start.y, 0.0),
                );
            }

            let right_barbs = [
                (vec2(0.28, 0.80), vec2(1.22, 1.06), green),
                (vec2(0.46, 1.34), vec2(1.76, 1.82), cyan),
                (vec2(0.66, 2.02), vec2(2.14, 2.60), teal),
                (vec2(0.92, 2.76), vec2(2.42, 3.42), cyan),
                (vec2(1.18, 3.54), vec2(2.50, 4.18), green),
                (vec2(1.46, 4.22), vec2(2.28, 4.74), teal),
            ];
            for (start, end, color) in right_barbs {
                let mid = vec2(
                    (start.x + end.x) * 0.5 + 0.14,
                    (start.y + end.y) * 0.5 + 0.10,
                );
                project_with_offset(
                    ctx,
                    Path::new()
                        .with_thickness(0.025)
                        .with_color(color)
                        .move_to(vec2(0.0, 0.0))
                        .quad_to(mid - start, end - start),
                    Vec3::new(feather_anchor.x + start.x, feather_anchor.y + start.y, 0.0),
                );
            }

            for point in [
                vec2(0.22, 0.74),
                vec2(0.48, 1.78),
                vec2(0.88, 3.16),
                vec2(1.18, 3.88),
                vec2(1.48, 4.48),
                vec2(1.82, 4.96),
            ] {
                let t = (point.y / 5.0).clamp(0.0, 1.0);
                project_with_offset(
                    ctx,
                    Circle::new(0.050, 28, blue.lerp(teal, t))
                        .with_stroke(0.012, Vec4::new(0.92, 0.98, 1.0, 0.24)),
                    Vec3::new(feather_anchor.x + point.x, feather_anchor.y + point.y, 0.0),
                );
            }

            let eye_center = feather_anchor + vec2(1.36, 3.58);
            project_with_offset(
                ctx,
                Path::new()
                    .with_thickness(0.024)
                    .with_color(eye_sheen)
                    .move_to(vec2(1.16, 2.86))
                    .cubic_to(vec2(1.64, 3.18), vec2(2.02, 3.78), vec2(1.82, 4.56))
                    .quad_to(vec2(1.56, 5.02), vec2(1.20, 5.14)),
                Vec3::new(feather_anchor.x, feather_anchor.y, 0.0),
            );
            project_with_offset(
                ctx,
                Ellipse::new(0.98, 0.74, Vec4::new(0.0, 0.0, 0.0, 0.0)).with_stroke(0.034, green),
                Vec3::new(eye_center.x, eye_center.y, 0.0),
            );
            project_with_offset(
                ctx,
                Ellipse::new(0.70, 0.50, Vec4::new(0.0, 0.0, 0.0, 0.0)).with_stroke(0.030, cyan),
                Vec3::new(eye_center.x, eye_center.y, 0.0),
            );
            project_with_offset(
                ctx,
                Ellipse::new(0.38, 0.28, Vec4::new(0.0, 0.0, 0.0, 0.0)).with_stroke(0.028, amber),
                Vec3::new(eye_center.x, eye_center.y, 0.0),
            );
            project_with_offset(
                ctx,
                Circle::new(0.10, 28, blue).with_stroke(0.015, Vec4::new(0.92, 0.97, 1.0, 0.32)),
                Vec3::new(eye_center.x, eye_center.y, 0.0),
            );

            project_with_offset(
                ctx,
                Path::new()
                    .with_thickness(0.018)
                    .with_color(Vec4::new(0.28, 0.80, 0.96, 0.24))
                    .with_dash(0.12, 0.08)
                    .move_to(vec2(0.66, 2.12))
                    .cubic_to(vec2(1.04, 2.84), vec2(1.62, 3.74), vec2(2.18, 4.42)),
                Vec3::new(feather_anchor.x, feather_anchor.y, 0.0),
            );

            for point in [
                feather_anchor + vec2(0.20, 1.02),
                feather_anchor + vec2(0.54, 2.04),
                feather_anchor + vec2(1.04, 3.18),
                feather_anchor + vec2(1.56, 4.22),
            ] {
                project_with_offset(
                    ctx,
                    Circle::new(0.036, 28, Vec4::new(0.30, 0.84, 0.96, 0.92))
                        .with_stroke(0.012, Vec4::new(0.92, 0.98, 1.0, 0.18)),
                    Vec3::new(point.x, point.y, 0.0),
                );
            }
        });
    }
}

impl Bounded for MuraliLogoMark {
    fn local_bounds(&self) -> Bounds {
        let base = Bounds::new(vec2(-5.42, -2.54), vec2(2.76, 2.98));
        Bounds::new(base.min * self.scale, base.max * self.scale)
    }
}

fn project_with_offset<T>(ctx: &mut ProjectionCtx, tattva: T, offset: Vec3)
where
    T: Project,
{
    ctx.with_offset(offset, |ctx| tattva.project(ctx));
}
