use crate::backend::renderer::mesh::MeshInstance;
use crate::engine::scene::SharedProps;
use crate::frontend::TattvaId;
use glam::{Vec2, Vec3, Vec4};
use std::sync::Arc;

/// Wraps a GPU-uploaded mesh for the renderer.
pub struct MeshComponent(pub Arc<MeshInstance>);

/// Stable authored order retained independently of ECS entity allocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderOrderComponent {
    pub tattva_id: TattvaId,
    pub primitive_index: u32,
}

/// Data for the instanced line renderer.
pub struct LineComponent {
    pub start: Vec3,
    pub end: Vec3,
    pub thickness: f32,
    pub dash_length: f32,
    pub gap_length: f32,
    pub dash_offset: f32,
}

/// Stores a color for the shader to use.
pub struct ColorComponent(pub Vec4);

/// Metadata for Text rendering (used by the Text Pipeline).
pub struct TextComponent {
    pub content: String,
    pub height: f32,
}

/// Metadata for LaTeX rendering.
pub struct LatexComponent {
    pub source: String,
    pub height: f32,
}

/// Metadata for Typst rendering.
pub struct TypstComponent {
    pub source: String,
    pub height: f32,
}

pub struct QuadComponent {
    pub size: Vec2,
    pub texture_id: Option<u64>,
}

// We use the Sangh's shared props as a component directly.
// hecs allows any type that is Send + Sync + 'static.
pub type TransformComponent = SharedProps;
