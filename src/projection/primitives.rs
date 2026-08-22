use crate::projection::mesh::Mesh;
use glam::{Vec3, Vec4};
use std::sync::Arc;

/// CPU-side render primitives produced by the Projection layer.
/// These are later materialized into GPU resources by the Backend.
/// CPU-side render primitives produced by the Projection layer.
///
/// These are intermediate representations of visual elements after they have
/// been projected into 3D space but before they are materialized into
/// GPU-specific resources (like vertex buffers or texture handles).
pub enum RenderPrimitive {
    /// A pre-computed 3D triangle mesh.
    ///
    /// Meshes are wrapped in an [`Arc`] for efficient reuse across frames and
    /// multiple instances (e.g., in
    /// [`NeuralNetworkDiagram`](crate::frontend::collection::ai::deep_learning::neural_network_diagram::NeuralNetworkDiagram)).
    Mesh(Arc<Mesh>),

    /// A 3D line segment with fixed thickness.
    ///
    /// Lines can be rendered as solid colors or dashed patterns. The backend
    /// typically materializes these into instanced quads or specialized line-meshes.
    Line {
        /// Start position in 3D world space.
        start: Vec3,
        /// End position in 3D world space.
        end: Vec3,
        /// Thickness of the line in world units.
        thickness: f32,
        /// Base color (subject to opacity stack).
        color: Vec4,
        /// Length of a single dash (0.0 for solid lines).
        dash_length: f32,
        /// Gap between dashes.
        gap_length: f32,
        /// Offset for the dash pattern (for 'marching ants' effects).
        dash_offset: f32,
    },

    /// A text string rendered at a specific world position.
    ///
    /// Character layout and glyph generation are handled by the text resource system
    /// in the Projection context during materialization.
    Text {
        /// String content to render.
        content: String,
        /// Font height in world units.
        height: f32,
        /// Text color.
        color: Vec4,
        /// Optional registered font key.
        font_name: Option<String>,
        /// Bottom-left or center anchor position in 3D space.
        offset: Vec3,
        /// Rotation in radians around Z-axis (for vertical text).
        rotation: f32,
    },

    /// A LaTeX mathematical formula.
    ///
    /// The formula is typically converted into a [`crate::frontend::collection::primitives::path::Path`]
    /// or pre-rendered texture depending on the Backend implementation.
    Latex {
        /// LaTeX source string (e.g., `e^{i\pi} + 1 = 0`).
        source: String,
        /// Vertical height of the rendered formula.
        height: f32,
        /// Fill color for the math symbols.
        color: Vec4,
        /// World position for the formula center.
        offset: Vec3,
    },

    /// A Typst document fragment.
    ///
    /// Typst allows for complex layout and high-fidelity typesetting
    /// within the 3D scene.
    Typst {
        /// Typst source content.
        source: String,
        /// Vertical scale factor (font height or total height depending on normalize).
        height: f32,
        /// Text color (acts as tint if tint is true).
        color: Vec4,
        /// World position anchor.
        offset: Vec3,
        /// If true, the height is normalized to match typographic height.
        /// If false, height is proportional to the Typst font scale.
        normalize: bool,
        /// If true, RGB is forced to 255 so the frontend color can tint the glyphs.
        /// If false, original SVG colors (syntax highlighting) are preserved.
        tint: bool,
    },
}

impl RenderPrimitive {
    /// Returns a new primitive translated by the given offset.
    pub fn translated(self, offset: Vec3) -> Self {
        match self {
            RenderPrimitive::Mesh(mesh) => RenderPrimitive::Mesh(mesh.as_ref().translated(offset)),
            RenderPrimitive::Line {
                start,
                end,
                thickness,
                color,
                dash_length,
                gap_length,
                dash_offset,
            } => RenderPrimitive::Line {
                start: start + offset,
                end: end + offset,
                thickness,
                color,
                dash_length,
                gap_length,
                dash_offset,
            },
            RenderPrimitive::Text {
                content,
                height,
                color,
                font_name,
                offset: old_offset,
                rotation,
            } => RenderPrimitive::Text {
                content,
                height,
                color,
                font_name,
                offset: old_offset + offset,
                rotation,
            },
            RenderPrimitive::Latex {
                source,
                height,
                color,
                offset: old_offset,
            } => RenderPrimitive::Latex {
                source,
                height,
                color,
                offset: old_offset + offset,
            },
            RenderPrimitive::Typst {
                source,
                height,
                color,
                offset: old_offset,
                normalize,
                tint,
            } => RenderPrimitive::Typst {
                source,
                height,
                color,
                offset: old_offset + offset,
                normalize,
                tint,
            },
        }
    }

    /// returns a new primitive with updated opacity.
    ///
    /// The `alpha` value is multiplied with the existing color's alpha channel.
    pub fn with_opacity(self, alpha: f32) -> Self {
        let f = |c: Vec4| Vec4::new(c.x, c.y, c.z, c.w * alpha);
        match self {
            RenderPrimitive::Mesh(mesh) => RenderPrimitive::Mesh(mesh.with_opacity(alpha)),
            RenderPrimitive::Line {
                start,
                end,
                thickness,
                color,
                dash_length,
                gap_length,
                dash_offset,
            } => RenderPrimitive::Line {
                start,
                end,
                thickness,
                color: f(color),
                dash_length,
                gap_length,
                dash_offset,
            },
            RenderPrimitive::Text {
                content,
                height,
                color,
                font_name,
                offset,
                rotation,
            } => RenderPrimitive::Text {
                content,
                height,
                color: f(color),
                font_name,
                offset,
                rotation,
            },
            RenderPrimitive::Latex {
                source,
                height,
                color,
                offset,
            } => RenderPrimitive::Latex {
                source,
                height,
                color: f(color),
                offset,
            },
            RenderPrimitive::Typst {
                source,
                height,
                color,
                offset,
                normalize,
                tint,
            } => RenderPrimitive::Typst {
                source,
                height,
                color: f(color),
                offset,
                normalize,
                tint,
            },
        }
    }

    /// Returns a new primitive scaled by the given factor.
    pub fn scaled(self, scale: f32) -> Self {
        match self {
            RenderPrimitive::Mesh(mesh) => RenderPrimitive::Mesh(mesh.as_ref().scaled(scale)),
            RenderPrimitive::Line {
                start,
                end,
                thickness,
                color,
                dash_length,
                gap_length,
                dash_offset,
            } => RenderPrimitive::Line {
                start: start * scale,
                end: end * scale,
                thickness: thickness * scale,
                color,
                dash_length: dash_length * scale,
                gap_length: gap_length * scale,
                dash_offset: dash_offset * scale,
            },
            RenderPrimitive::Text {
                content,
                height,
                color,
                font_name,
                offset,
                rotation,
            } => RenderPrimitive::Text {
                content,
                height: height * scale,
                color,
                font_name,
                offset: offset * scale,
                rotation,
            },
            RenderPrimitive::Latex {
                source,
                height,
                color,
                offset,
            } => RenderPrimitive::Latex {
                source,
                height: height * scale,
                color,
                offset: offset * scale,
            },
            RenderPrimitive::Typst {
                source,
                height,
                color,
                offset,
                normalize,
                tint,
            } => RenderPrimitive::Typst {
                source,
                height: height * scale,
                color,
                offset: offset * scale,
                normalize,
                tint,
            },
        }
    }
}
