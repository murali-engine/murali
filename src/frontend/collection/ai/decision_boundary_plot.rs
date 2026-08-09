use glam::{Vec2, Vec3, Vec4, vec2};

use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Mesh, Project, ProjectionCtx, RenderPrimitive};
use crate::validation::ValidationError;

#[derive(Debug, Clone)]
pub struct LabeledPoint {
    pub point: Vec2,
    pub class: usize,
}

#[derive(Clone)]
pub struct DecisionBoundaryPlot {
    pub x_range: (f32, f32),
    pub y_range: (f32, f32),
    pub points: Vec<LabeledPoint>,
    pub grid_resolution: usize,
    pub class_a_color: Vec4,
    pub class_b_color: Vec4,
    pub boundary_color: Vec4,
    pub point_radius: f32,
    pub classifier: fn(Vec2) -> f32,
}

impl DecisionBoundaryPlot {
    pub fn new(x_range: (f32, f32), y_range: (f32, f32), classifier: fn(Vec2) -> f32) -> Self {
        Self {
            x_range,
            y_range,
            points: Vec::new(),
            grid_resolution: 24,
            class_a_color: Vec4::new(0.24, 0.61, 0.93, 1.0),
            class_b_color: Vec4::new(0.93, 0.39, 0.45, 1.0),
            boundary_color: Vec4::new(0.96, 0.96, 0.98, 1.0),
            point_radius: 0.08,
            classifier,
        }
    }

    pub fn project_with_points(&self, ctx: &mut ProjectionCtx, points: &[LabeledPoint]) {
        let nx = self.grid_resolution.max(2);
        let ny = self.grid_resolution.max(2);
        let dx = (self.x_range.1 - self.x_range.0) / nx as f32;
        let dy = (self.y_range.1 - self.y_range.0) / ny as f32;

        for ix in 0..nx {
            for iy in 0..ny {
                let cx = self.x_range.0 + (ix as f32 + 0.5) * dx;
                let cy = self.y_range.0 + (iy as f32 + 0.5) * dy;
                let v = (self.classifier)(vec2(cx, cy));
                let color = if v >= 0.0 {
                    self.class_a_color
                } else {
                    self.class_b_color
                };
                let mesh = Mesh::square(dx.max(dy) * 0.95, color)
                    .as_ref()
                    .translated(Vec3::new(cx, cy, 0.0));
                ctx.emit(RenderPrimitive::Mesh(mesh));
            }
        }

        let contour_steps = self.grid_resolution.max(2).saturating_mul(5);
        let segments = match marching_squares_segments(
            self.x_range,
            self.y_range,
            contour_steps,
            self.classifier,
        ) {
            Ok(segments) => segments,
            Err(error) => {
                ctx.report(error);
                return;
            }
        };
        for (start, end) in segments {
            ctx.emit(RenderPrimitive::Line {
                start: start.extend(0.0),
                end: end.extend(0.0),
                thickness: 0.025,
                color: self.boundary_color,
                dash_length: 0.0,
                gap_length: 0.0,
                dash_offset: 0.0,
            });
        }

        for point in points {
            let color = if point.class == 0 {
                self.class_a_color
            } else {
                self.class_b_color
            };
            let mesh = Mesh::circle(self.point_radius, 20, color)
                .as_ref()
                .translated(Vec3::new(point.point.x, point.point.y, 0.0));
            ctx.emit(RenderPrimitive::Mesh(mesh));
        }
    }
}

fn marching_squares_segments(
    x_range: (f32, f32),
    y_range: (f32, f32),
    steps: usize,
    classifier: fn(Vec2) -> f32,
) -> Result<Vec<(Vec2, Vec2)>, ValidationError> {
    let steps = steps.max(1);
    let width = steps + 1;
    let dx = (x_range.1 - x_range.0) / steps as f32;
    let dy = (y_range.1 - y_range.0) / steps as f32;
    let mut values = Vec::with_capacity(width * width);

    for ix in 0..=steps {
        for iy in 0..=steps {
            let point = vec2(x_range.0 + ix as f32 * dx, y_range.0 + iy as f32 * dy);
            let value = classifier(point);
            if !value.is_finite() {
                return Err(ValidationError::non_finite(
                    "DecisionBoundaryPlot",
                    "classifier",
                    value,
                ));
            }
            values.push(value);
        }
    }

    let value_at = |ix: usize, iy: usize| values[ix * width + iy];
    let mut segments = Vec::new();
    for ix in 0..steps {
        for iy in 0..steps {
            let bottom_left = vec2(x_range.0 + ix as f32 * dx, y_range.0 + iy as f32 * dy);
            let bottom_right = bottom_left + vec2(dx, 0.0);
            let top_right = bottom_left + vec2(dx, dy);
            let top_left = bottom_left + vec2(0.0, dy);
            let corner_values = [
                value_at(ix, iy),
                value_at(ix + 1, iy),
                value_at(ix + 1, iy + 1),
                value_at(ix, iy + 1),
            ];
            let case_index = corner_values
                .iter()
                .enumerate()
                .fold(0_u8, |index, (bit, value)| {
                    index | (u8::from(*value >= 0.0) << bit)
                });
            let center_positive = corner_values.iter().sum::<f32>() >= 0.0;
            let edge_points = [
                interpolate_zero(
                    bottom_left,
                    bottom_right,
                    corner_values[0],
                    corner_values[1],
                ),
                interpolate_zero(bottom_right, top_right, corner_values[1], corner_values[2]),
                interpolate_zero(top_left, top_right, corner_values[3], corner_values[2]),
                interpolate_zero(bottom_left, top_left, corner_values[0], corner_values[3]),
            ];

            for &(start_edge, end_edge) in cell_edge_pairs(case_index, center_positive) {
                segments.push((edge_points[start_edge], edge_points[end_edge]));
            }
        }
    }
    Ok(segments)
}

fn interpolate_zero(start: Vec2, end: Vec2, start_value: f32, end_value: f32) -> Vec2 {
    let denominator = start_value - end_value;
    let t = if denominator.abs() <= f32::EPSILON {
        0.5
    } else {
        (start_value / denominator).clamp(0.0, 1.0)
    };
    start.lerp(end, t)
}

fn cell_edge_pairs(case_index: u8, center_positive: bool) -> &'static [(usize, usize)] {
    match case_index {
        0 | 15 => &[],
        1 | 14 => &[(0, 3)],
        2 | 13 => &[(0, 1)],
        3 | 12 => &[(3, 1)],
        4 | 11 => &[(1, 2)],
        5 if center_positive => &[(0, 1), (2, 3)],
        5 => &[(0, 3), (1, 2)],
        6 | 9 => &[(0, 2)],
        7 | 8 => &[(2, 3)],
        10 if center_positive => &[(0, 3), (1, 2)],
        10 => &[(0, 1), (2, 3)],
        _ => &[],
    }
}

impl Project for DecisionBoundaryPlot {
    fn project(&self, ctx: &mut ProjectionCtx) {
        self.project_with_points(ctx, &self.points);
    }
}

impl Bounded for DecisionBoundaryPlot {
    fn local_bounds(&self) -> Bounds {
        Bounds::new(
            vec2(self.x_range.0, self.y_range.0),
            vec2(self.x_range.1, self.y_range.1),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marching_squares_tracks_a_straight_zero_contour() {
        let segments =
            marching_squares_segments((-1.0, 1.0), (-1.0, 1.0), 16, |point| point.x).unwrap();

        assert_eq!(segments.len(), 16);
        assert!(segments.iter().all(|(start, end)| {
            start.x.abs() < 1e-6 && end.x.abs() < 1e-6 && start.distance(*end) <= 0.1251
        }));
    }

    #[test]
    fn marching_squares_keeps_disconnected_contours_separate() {
        fn two_circles(point: Vec2) -> f32 {
            let left = (point - vec2(-1.0, 0.0)).length_squared() - 0.25;
            let right = (point - vec2(1.0, 0.0)).length_squared() - 0.25;
            left * right
        }

        let steps = 80;
        let segments =
            marching_squares_segments((-2.0, 2.0), (-1.5, 1.5), steps, two_circles).unwrap();
        let max_cell_diagonal = vec2(4.0 / steps as f32, 3.0 / steps as f32).length();

        assert!(!segments.is_empty());
        assert!(
            segments
                .iter()
                .all(|(start, end)| start.distance(*end) <= max_cell_diagonal + 1e-5)
        );
        assert!(
            segments
                .iter()
                .any(|(start, end)| ((start.x + end.x) * 0.5) < -0.5)
        );
        assert!(
            segments
                .iter()
                .any(|(start, end)| ((start.x + end.x) * 0.5) > 0.5)
        );
    }

    #[test]
    fn ambiguous_saddles_use_the_center_sign_consistently() {
        assert_eq!(cell_edge_pairs(5, true), &[(0, 1), (2, 3)]);
        assert_eq!(cell_edge_pairs(5, false), &[(0, 3), (1, 2)]);
        assert_eq!(cell_edge_pairs(10, true), &[(0, 3), (1, 2)]);
        assert_eq!(cell_edge_pairs(10, false), &[(0, 1), (2, 3)]);
    }
}
