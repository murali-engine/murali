use glam::Vec4;

use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};

/// Frontend Typst object.
/// Used for rich document layout and advanced typesetting.
///
/// The actual compilation and rasterization are handled by
/// the Typst compiler within the Resource layer.
#[derive(Debug, Clone)]
pub struct Typst {
    pub source: String,
    pub world_height: f32,
    pub color: Vec4, // RGBA for rich text blending
}

impl Typst {
    /// Creates a new Typst Tattva from raw Typst markup.
    pub fn new(source: impl Into<String>, world_height: f32) -> Self {
        Self {
            source: source.into(),
            world_height,
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
        }
    }

    /// Builder-style color setter
    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }
}

impl Project for Typst {
    fn project(&self, ctx: &mut ProjectionCtx) {
        // Emit the Typst primitive.
        // The Sync Boundary will use src/resource/typst/compiler.rs
        // and cache the resulting texture to ensure performance.
        ctx.emit(RenderPrimitive::Typst {
            source: self.source.clone(),
            height: self.world_height,
            color: self.color,
            offset: glam::Vec3::ZERO,
            normalize: true,
            tint: true,
        });
    }
}

impl Bounded for Typst {
    fn local_bounds(&self) -> Bounds {
        let width = self.source.chars().count() as f32 * self.world_height * 0.55;
        Bounds::from_center_size(
            glam::Vec2::ZERO,
            glam::vec2(width.max(self.world_height), self.world_height),
        )
    }
}
