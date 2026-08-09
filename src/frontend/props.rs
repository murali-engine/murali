// src/frontend/props.rs

use glam::Mat4;
use glam::{Quat, Vec3};
use parking_lot::RwLock;
use std::sync::Arc;

pub type SharedProps = Arc<RwLock<DrawableProps>>;

/// Conventional 2D painter-order layers. Custom `i32` values are also valid.
pub mod layers {
    pub const BACKGROUND: i32 = -1000;
    pub const CONTENT: i32 = 0;
    pub const OVERLAY: i32 = 1000;
    pub const UI: i32 = 2000;
}

/// Controls whether a drawable participates in world depth or renders as an overlay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DepthMode {
    #[default]
    World,
    Overlay,
}

/// Runtime visual state of a drawable.
/// This is authoritative for rendering and animation.
#[derive(Debug, Clone)]
pub struct DrawableProps {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub visible: bool,
    pub opacity: f32,
    /// Painter-order layer for 2D scenes. Higher layers are drawn later.
    pub layer: i32,
    pub depth_mode: DepthMode,
    pub tag: Option<String>,
}

impl Default for DrawableProps {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            visible: true,
            opacity: 1.0,
            layer: layers::CONTENT,
            depth_mode: DepthMode::World,
            tag: None,
        }
    }
}

/// Builder-style helpers
impl DrawableProps {
    pub fn identity() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
            visible: true,
            opacity: 1.0,
            layer: layers::CONTENT,
            depth_mode: DepthMode::World,
            tag: None,
        }
    }

    pub fn model_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    pub fn at(mut self, pos: Vec3) -> Self {
        self.position = pos;
        self
    }

    pub fn scale_uniform(mut self, s: f32) -> Self {
        self.scale = Vec3::splat(s);
        self
    }

    pub fn scale(mut self, v: Vec3) -> Self {
        self.scale = v;
        self
    }

    pub fn rotate(mut self, q: Quat) -> Self {
        self.rotation = q;
        self
    }

    pub fn hide(mut self) -> Self {
        self.visible = false;
        self
    }

    pub fn show(mut self) -> Self {
        self.visible = true;
        self
    }

    pub fn opacity(mut self, v: f32) -> Self {
        self.opacity = v.clamp(0.0, 1.0);
        self
    }

    pub fn layer(mut self, layer: i32) -> Self {
        self.layer = layer;
        self
    }

    pub fn depth_mode(mut self, depth_mode: DepthMode) -> Self {
        self.depth_mode = depth_mode;
        self
    }

    pub fn write(shared: &SharedProps) -> parking_lot::RwLockWriteGuard<'_, Self> {
        shared.write()
    }

    pub fn read(shared: &SharedProps) -> parking_lot::RwLockReadGuard<'_, Self> {
        shared.read()
    }
}

#[cfg(test)]
mod tests {
    use super::{DepthMode, DrawableProps, layers};

    #[test]
    fn drawable_layer_defaults_to_content_and_supports_custom_values() {
        assert_eq!(DrawableProps::default().layer, layers::CONTENT);
        assert_eq!(DrawableProps::default().layer(42).layer, 42);
        assert_eq!(DrawableProps::default().depth_mode, DepthMode::World);
    }
}
