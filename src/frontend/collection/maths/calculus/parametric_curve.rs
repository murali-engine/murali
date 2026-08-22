use glam::{Vec2, Vec3, Vec4};

use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};

#[derive(Clone, Copy)]
pub struct ParametricCurve {
    pub t_range: (f32, f32),
    pub samples: usize,
    pub thickness: f32,
    pub color: Vec4,
    pub f: fn(f32) -> Vec2,
}

impl ParametricCurve {
    pub fn new(t_range: (f32, f32), f: fn(f32) -> Vec2) -> Self {
        Self {
            t_range,
            samples: 128,
            thickness: 0.03,
            color: Vec4::new(0.95, 0.57, 0.25, 1.0),
            f,
        }
    }

    pub fn with_samples(mut self, samples: usize) -> Self {
        self.samples = samples.max(2);
        self
    }

    pub fn sample_points(&self) -> Vec<Vec2> {
        let mut pts = Vec::with_capacity(self.samples.max(2));
        let samples = self.samples.max(2);
        let step = (self.t_range.1 - self.t_range.0) / (samples - 1) as f32;
        for i in 0..samples {
            let t = self.t_range.0 + i as f32 * step;
            pts.push((self.f)(t));
        }
        pts
    }
}

impl Project for ParametricCurve {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let pts = self.sample_points();
        for pair in pts.windows(2) {
            let a = pair[0];
            let b = pair[1];
            ctx.emit(RenderPrimitive::Line {
                start: Vec3::new(a.x, a.y, 0.0),
                end: Vec3::new(b.x, b.y, 0.0),
                thickness: self.thickness,
                color: self.color,
                dash_length: 0.0,
                gap_length: 0.0,
                dash_offset: 0.0,
            });
        }
    }
}

impl Bounded for ParametricCurve {
    fn local_bounds(&self) -> Bounds {
        let pts = self.sample_points();
        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = Vec2::splat(f32::NEG_INFINITY);
        for p in pts {
            min.x = min.x.min(p.x);
            min.y = min.y.min(p.y);
            max.x = max.x.max(p.x);
            max.y = max.y.max(p.y);
        }
        Bounds::new(min, max)
    }
}
