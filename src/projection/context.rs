// src/projection/context.rs
use crate::engine::scene::SharedProps;
use crate::projection::RenderPrimitive;
use crate::validation::ValidationError;
use glam::Vec3;

/// The Projection Context is the "Collector".
/// Each Tattva is passed this context during the project() phase.
pub struct ProjectionCtx {
    /// Primitives collected during this projection pass
    pub primitives: Vec<RenderPrimitive>,

    /// Structured failures collected during this projection pass.
    pub diagnostics: Vec<ValidationError>,

    /// Shared spatial properties of the parent Tattva
    pub props: SharedProps,

    /// Stack of local translations applied to emitted primitives
    offset_stack: Vec<Vec3>,
    /// Stack of local alpha multipliers
    alpha_stack: Vec<f32>,
    /// Stack of local scale multipliers
    scale_stack: Vec<f32>,
}

impl ProjectionCtx {
    pub fn new(props: SharedProps) -> Self {
        Self {
            primitives: Vec::new(),
            diagnostics: Vec::new(),
            props,
            offset_stack: vec![Vec3::ZERO],
            alpha_stack: vec![1.0],
            scale_stack: vec![1.0],
        }
    }

    /// Reject the current projection pass with a structured diagnostic.
    pub fn report(&mut self, error: ValidationError) {
        self.diagnostics.push(error);
    }

    /// Emit a renderable primitive
    pub fn emit(&mut self, primitive: RenderPrimitive) {
        let offset = self.offset_stack.last().copied().unwrap_or(Vec3::ZERO);
        let alpha = self.alpha_stack.last().copied().unwrap_or(1.0);
        let scale = self.scale_stack.last().copied().unwrap_or(1.0);

        // Apply local transformations before collecting
        let p = primitive
            .scaled(scale)
            .translated(offset)
            .with_opacity(alpha);
        self.primitives.push(p);
    }

    /// Push a local translation offset onto the stack.
    /// All subsequent `emit` calls will be translated by this offset.
    pub fn push_offset(&mut self, offset: Vec3) {
        let current = self.offset_stack.last().copied().unwrap_or(Vec3::ZERO);
        self.offset_stack.push(current + offset);
    }

    pub fn pop_offset(&mut self) {
        if self.offset_stack.len() > 1 {
            self.offset_stack.pop();
        }
    }

    /// Push a local alpha multiplier onto the stack.
    /// All subsequent `emit` calls will have their opacity scaled.
    pub fn push_opacity(&mut self, alpha: f32) {
        let current = self.alpha_stack.last().copied().unwrap_or(1.0);
        self.alpha_stack.push(current * alpha);
    }

    pub fn pop_opacity(&mut self) {
        if self.alpha_stack.len() > 1 {
            self.alpha_stack.pop();
        }
    }

    /// Push a local scale multiplier onto the stack.
    pub fn push_scale(&mut self, scale: f32) {
        let current = self.scale_stack.last().copied().unwrap_or(1.0);
        self.scale_stack.push(current * scale);
    }

    pub fn pop_scale(&mut self) {
        if self.scale_stack.len() > 1 {
            self.scale_stack.pop();
        }
    }

    /// RAII helper for scoped scale
    pub fn with_scale<F: FnOnce(&mut Self)>(&mut self, scale: f32, f: F) {
        self.push_scale(scale);
        f(self);
        self.pop_scale();
    }

    /// RAII helper for scoped offsets
    pub fn with_offset<F: FnOnce(&mut Self)>(&mut self, offset: Vec3, f: F) {
        self.push_offset(offset);
        f(self);
        self.pop_offset();
    }

    /// RAII helper for scoped opacity
    pub fn with_opacity<F: FnOnce(&mut Self)>(&mut self, alpha: f32, f: F) {
        self.push_opacity(alpha);
        f(self);
        self.pop_opacity();
    }
}
