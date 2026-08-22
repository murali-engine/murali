use glam::{Vec2, Vec3, Vec4, vec2};

use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};

#[derive(Clone, Copy)]
pub struct ParametricCurve3D {
    pub t_range: (f32, f32),
    pub samples: usize,
    pub thickness: f32,
    pub color: Vec4,
    pub f: fn(f32) -> Vec3,
}

impl ParametricCurve3D {
    pub fn new(t_range: (f32, f32), f: fn(f32) -> Vec3) -> Self {
        Self {
            t_range,
            samples: 128,
            thickness: 0.03,
            color: Vec4::new(0.98, 0.74, 0.28, 1.0),
            f,
        }
    }

    pub fn with_samples(mut self, samples: usize) -> Self {
        self.samples = samples.max(2);
        self
    }

    pub fn sample_points(&self) -> Vec<Vec3> {
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

impl Project for ParametricCurve3D {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let pts = self.sample_points();
        for pair in pts.windows(2) {
            let a = pair[0];
            let b = pair[1];
            ctx.emit(RenderPrimitive::Line {
                start: a,
                end: b,
                thickness: self.thickness,
                color: self.color,
                dash_length: 0.0,
                gap_length: 0.0,
                dash_offset: 0.0,
            });
        }
    }
}

impl Bounded for ParametricCurve3D {
    fn local_bounds(&self) -> Bounds {
        let pts = self.sample_points();
        let mut min_xy = Vec2::splat(f32::INFINITY);
        let mut max_xy = Vec2::splat(f32::NEG_INFINITY);
        let mut min_z = f32::INFINITY;
        let mut max_z = f32::NEG_INFINITY;

        for p in pts {
            min_xy.x = min_xy.x.min(p.x);
            min_xy.y = min_xy.y.min(p.y);
            max_xy.x = max_xy.x.max(p.x);
            max_xy.y = max_xy.y.max(p.y);
            min_z = min_z.min(p.z);
            max_z = max_z.max(p.z);
        }

        let z_pad = max_z.abs().max(min_z.abs()) * 0.15;
        Bounds::new(min_xy - vec2(z_pad, z_pad), max_xy + vec2(z_pad, z_pad))
    }
}
