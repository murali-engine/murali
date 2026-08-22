use glam::{Vec2, Vec4, vec3};

use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};

use super::EPSILON;
use super::vector::VectorArrow2D;

#[derive(Debug, Clone)]
/// Experimental API: this type is part of the evolving linear algebra visual toolkit.
pub struct ProjectionShadow {
    pub vector: Vec2,
    pub onto: Vec2,
    pub vector_color: Vec4,
    pub projection_color: Vec4,
    pub residual_color: Vec4,
    pub guide_color: Vec4,
    pub thickness: f32,
    pub show_original: bool,
    pub show_residual: bool,
}

impl ProjectionShadow {
    pub fn new(vector: Vec2, onto: Vec2) -> Self {
        Self {
            vector,
            onto,
            vector_color: Vec4::new(0.34, 0.78, 0.95, 1.0),
            projection_color: Vec4::new(0.42, 0.82, 0.48, 1.0),
            residual_color: Vec4::new(0.95, 0.36, 0.34, 1.0),
            guide_color: Vec4::new(0.78, 0.82, 0.88, 0.55),
            thickness: 0.035,
            show_original: true,
            show_residual: true,
        }
    }

    pub fn projection(&self) -> Vec2 {
        let denom = self.onto.length_squared();
        if denom <= EPSILON {
            Vec2::ZERO
        } else {
            self.onto * (self.vector.dot(self.onto) / denom)
        }
    }

    pub fn residual(&self) -> Vec2 {
        self.vector - self.projection()
    }

    pub fn with_original(mut self, show: bool) -> Self {
        self.show_original = show;
        self
    }

    pub fn with_residual(mut self, show: bool) -> Self {
        self.show_residual = show;
        self
    }
}

impl Project for ProjectionShadow {
    fn project(&self, ctx: &mut ProjectionCtx) {
        if self.onto.length_squared() <= EPSILON {
            return;
        }

        let projection = self.projection();
        if self.show_original {
            VectorArrow2D::from_origin(self.vector)
                .with_color(self.vector_color)
                .with_thickness(self.thickness)
                .project(ctx);
        }

        VectorArrow2D::from_origin(projection)
            .with_color(self.projection_color)
            .with_thickness(self.thickness)
            .project(ctx);

        ctx.emit(RenderPrimitive::Line {
            start: vec3(self.vector.x, self.vector.y, 0.0),
            end: vec3(projection.x, projection.y, 0.0),
            thickness: self.thickness * 0.55,
            color: self.guide_color,
            dash_length: 0.12,
            gap_length: 0.08,
            dash_offset: 0.0,
        });

        if self.show_residual {
            VectorArrow2D::new(projection, self.vector)
                .with_color(self.residual_color)
                .with_thickness(self.thickness * 0.8)
                .project(ctx);
        }
    }
}

impl Bounded for ProjectionShadow {
    fn local_bounds(&self) -> Bounds {
        let projection = self.projection();
        Bounds::new(
            Vec2::ZERO.min(self.vector).min(projection) - Vec2::splat(0.25),
            Vec2::ZERO.max(self.vector).max(projection) + Vec2::splat(0.25),
        )
    }
}
