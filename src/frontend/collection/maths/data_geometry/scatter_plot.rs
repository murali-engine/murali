use glam::{Vec2, Vec3, Vec4};

use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};

#[derive(Debug, Clone)]
pub struct ScatterPlot {
    pub points: Vec<Vec2>,
    pub point_radius: f32,
    pub color: Vec4,
    pub thickness: f32,
}

impl ScatterPlot {
    pub fn new(points: Vec<Vec2>) -> Self {
        Self {
            points,
            point_radius: 0.08,
            color: Vec4::new(0.92, 0.31, 0.53, 1.0),
            thickness: 0.025,
        }
    }
}

impl Project for ScatterPlot {
    fn project(&self, ctx: &mut ProjectionCtx) {
        for p in &self.points {
            ctx.emit(RenderPrimitive::Line {
                start: Vec3::new(p.x - self.point_radius, p.y, 0.0),
                end: Vec3::new(p.x + self.point_radius, p.y, 0.0),
                thickness: self.thickness,
                color: self.color,
                dash_length: 0.0,
                gap_length: 0.0,
                dash_offset: 0.0,
            });
            ctx.emit(RenderPrimitive::Line {
                start: Vec3::new(p.x, p.y - self.point_radius, 0.0),
                end: Vec3::new(p.x, p.y + self.point_radius, 0.0),
                thickness: self.thickness,
                color: self.color,
                dash_length: 0.0,
                gap_length: 0.0,
                dash_offset: 0.0,
            });
        }
    }
}

impl Bounded for ScatterPlot {
    fn local_bounds(&self) -> Bounds {
        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = Vec2::splat(f32::NEG_INFINITY);
        for p in &self.points {
            min.x = min.x.min(p.x - self.point_radius);
            min.y = min.y.min(p.y - self.point_radius);
            max.x = max.x.max(p.x + self.point_radius);
            max.y = max.y.max(p.y + self.point_radius);
        }
        Bounds::new(min, max)
    }
}
