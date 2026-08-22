use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};
use glam::{Vec2, Vec3, Vec4, vec2};

#[derive(Debug, Clone)]
pub struct OptimizationPath2D {
    pub points: Vec<Vec2>,
    pub color: Vec4,
    pub thickness: f32,
    pub show_points: bool,
    pub point_size: f32,
}

impl OptimizationPath2D {
    pub fn new(points: impl Into<Vec<Vec2>>) -> Self {
        Self {
            points: points.into(),
            color: Vec4::ONE,
            thickness: 0.025,
            show_points: true,
            point_size: 0.08,
        }
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }

    pub fn with_thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness;
        self
    }

    pub fn with_point_size(mut self, point_size: f32) -> Self {
        self.point_size = point_size;
        self
    }

    pub fn without_points(mut self) -> Self {
        self.show_points = false;
        self
    }

    pub fn start(&self) -> Option<Vec2> {
        self.points.first().copied()
    }

    pub fn end(&self) -> Option<Vec2> {
        self.points.last().copied()
    }

    pub fn steps(&self) -> usize {
        self.points.len().saturating_sub(1)
    }
}

impl Project for OptimizationPath2D {
    fn project(&self, ctx: &mut ProjectionCtx) {
        for pair in self.points.windows(2) {
            ctx.emit(RenderPrimitive::Line {
                start: Vec3::new(pair[0].x, pair[0].y, 0.0),
                end: Vec3::new(pair[1].x, pair[1].y, 0.0),
                thickness: self.thickness,
                color: self.color,
                dash_length: 0.0,
                gap_length: 0.0,
                dash_offset: 0.0,
            });
        }

        if self.show_points {
            for point in &self.points {
                ctx.emit(RenderPrimitive::Line {
                    start: Vec3::new(point.x - self.point_size, point.y, 0.0),
                    end: Vec3::new(point.x + self.point_size, point.y, 0.0),
                    thickness: self.thickness,
                    color: self.color,
                    dash_length: 0.0,
                    gap_length: 0.0,
                    dash_offset: 0.0,
                });
                ctx.emit(RenderPrimitive::Line {
                    start: Vec3::new(point.x, point.y - self.point_size, 0.0),
                    end: Vec3::new(point.x, point.y + self.point_size, 0.0),
                    thickness: self.thickness,
                    color: self.color,
                    dash_length: 0.0,
                    gap_length: 0.0,
                    dash_offset: 0.0,
                });
            }
        }
    }
}

impl Bounded for OptimizationPath2D {
    fn local_bounds(&self) -> Bounds {
        if self.points.is_empty() {
            return Bounds::new(Vec2::ZERO, Vec2::ZERO);
        }

        let mut min = self.points[0];
        let mut max = self.points[0];

        for point in &self.points {
            min = min.min(*point);
            max = max.max(*point);
        }

        let pad = vec2(self.point_size, self.point_size);
        Bounds::new(min - pad, max + pad)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::vec2;

    #[test]
    fn optimization_path_reports_steps_and_endpoints() {
        let path = OptimizationPath2D::new(vec![vec2(0.0, 1.0), vec2(0.5, 0.4), vec2(0.8, 0.1)]);

        assert_eq!(path.steps(), 2);
        assert_eq!(path.start(), Some(vec2(0.0, 1.0)));
        assert_eq!(path.end(), Some(vec2(0.8, 0.1)));
    }
}
