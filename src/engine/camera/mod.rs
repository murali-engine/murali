// src/camera/mod.rs
//! Camera definition for Murali.
//!
//! Design (LOCKED):
//! ----------------
//! - Single 3D camera model (always Vec3, Mat4).
//! - Orthographic projection by default (math-first).
//! - Perspective projection is opt-in.
//! - Camera is PURE state: no input, no movement logic.
//! - 2D scenes are constrained 3D scenes with explicit painter-order layers.

pub mod controller;
use crate::engine::frame::Frame;
use glam::{Mat4, Vec3};

/// Canonical world-space constants.
/// These define Murali's default coordinate canvas.
pub const DEFAULT_VIEW_WIDTH: f32 = 16.0;
pub const ASPECT_RATIO: f32 = 16.0 / 9.0;
pub const DEFAULT_VIEW_HEIGHT: f32 = DEFAULT_VIEW_WIDTH / ASPECT_RATIO;

/// Projection mode for the camera.
#[derive(Debug, Copy, Clone)]
pub enum Projection {
    /// Orthographic projection (default for math scenes)
    Orthographic {
        /// Visible width in world units
        width: f32,
        /// Visible height in world units
        height: f32,
        /// Near clipping plane
        near: f32,
        /// Far clipping plane
        far: f32,
    },

    /// Perspective projection (for true 3D scenes)
    Perspective {
        /// Vertical field of view (radians)
        fov_y_rad: f32,
        /// Aspect ratio (width / height)
        aspect: f32,
        /// Near clipping plane
        near: f32,
        /// Far clipping plane
        far: f32,
    },
}

impl Projection {
    /// Compute the projection matrix.
    pub fn matrix(&self) -> Mat4 {
        match *self {
            Projection::Orthographic {
                width,
                height,
                near,
                far,
            } => glam::camera::rh::proj::directx::orthographic(
                -width / 2.0,
                width / 2.0,
                -height / 2.0,
                height / 2.0,
                near,
                far,
            ),

            Projection::Perspective {
                fov_y_rad,
                aspect,
                near,
                far,
            } => glam::camera::rh::proj::directx::perspective(fov_y_rad, aspect, near, far),
        }
    }
}

/// Camera describing view + projection.
/// Owned by `Scene`.
#[derive(Debug, Copy, Clone)]
pub struct Camera {
    /// Camera position in world space
    pub position: Vec3,

    /// Point the camera is looking at
    pub target: Vec3,

    /// Up direction (usually +Y)
    pub up: Vec3,

    /// Projection mode
    pub projection: Projection,
}

impl Camera {
    pub fn for_frame(frame: Frame) -> Self {
        let mut camera = Self::default();
        camera.set_frame(frame);
        camera
    }

    /// Applies a scene frame while preserving the current projection mode.
    pub fn set_frame(&mut self, frame: Frame) {
        let (frame_width, frame_height) = frame.logical_size();
        match &mut self.projection {
            Projection::Orthographic { width, height, .. } => {
                *width = frame_width;
                *height = frame_height;
            }
            Projection::Perspective { aspect, .. } => {
                *aspect = frame.aspect_ratio();
            }
        }
    }

    /// Reconciles the current projection with a scene-owned aspect ratio.
    /// Orthographic cameras retain their current visible width.
    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        let aspect_ratio = aspect_ratio.max(0.001);
        match &mut self.projection {
            Projection::Orthographic { width, height, .. } => {
                *height = *width / aspect_ratio;
            }
            Projection::Perspective { aspect, .. } => {
                *aspect = aspect_ratio;
            }
        }
    }

    /// View matrix (world → view)
    pub fn view_matrix(&self) -> Mat4 {
        glam::camera::rh::view::look_at_mat4(self.position, self.target, self.up)
    }

    /// Projection matrix (view → clip)
    pub fn projection_matrix(&self) -> Mat4 {
        self.projection.matrix()
    }

    /// Combined view-projection matrix (world → clip)
    pub fn view_proj_matrix(&self) -> Mat4 {
        self.projection_matrix() * self.view_matrix()
    }

    /// Intersects the four camera-corner rays with a world-space Z plane.
    ///
    /// Returns `None` when the plane is parallel to any corner ray or the
    /// camera matrices cannot produce finite intersections.
    pub fn frame_bounds_at_z(&self, plane_z: f32) -> Option<crate::frontend::layout::Bounds> {
        use crate::frontend::layout::Bounds;

        let inverse = self.view_proj_matrix().inverse();
        if !inverse.is_finite() {
            return None;
        }

        let mut intersections = Vec::with_capacity(4);
        for (x, y) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
            let near = inverse.project_point3(Vec3::new(x, y, -1.0));
            let far = inverse.project_point3(Vec3::new(x, y, 1.0));
            let ray = far - near;
            if ray.z.abs() <= f32::EPSILON {
                return None;
            }
            let point = near + ray * ((plane_z - near.z) / ray.z);
            if !point.is_finite() {
                return None;
            }
            intersections.push(point.truncate());
        }

        let first = intersections[0];
        Some(
            intersections[1..]
                .iter()
                .fold(Bounds::new(first, first), |bounds, point| {
                    Bounds::new(bounds.min.min(*point), bounds.max.max(*point))
                }),
        )
    }

    /// Convenience: forward direction (normalized)
    pub fn forward(&self) -> Vec3 {
        (self.target - self.position).normalize()
    }

    /// Convenience: right direction (normalized)
    pub fn right(&self) -> Vec3 {
        self.forward().cross(self.up).normalize()
    }

    /// Returns the visible world width (orthographic only).
    /// For perspective cameras, returns 0.0.
    pub fn view_width(&self) -> f32 {
        match self.projection {
            Projection::Orthographic { width, .. } => width,
            Projection::Perspective { .. } => 0.0,
        }
    }

    /// Sets the visible world width while preserving the current aspect ratio.
    /// Smaller values zoom in (objects appear larger).
    /// Larger values zoom out (more world is visible).
    /// No-op for perspective cameras.
    pub fn set_view_width(&mut self, width: f32) {
        if let Projection::Orthographic {
            width: w,
            height: h,
            ..
        } = &mut self.projection
        {
            let current_aspect = *w / *h;
            let aspect = if current_aspect.is_finite() && current_aspect > 0.0 {
                current_aspect
            } else {
                ASPECT_RATIO
            };
            *w = width.max(0.01);
            *h = *w / aspect;
        }
    }

    /// Zoom in by a factor — objects appear `factor` times larger.
    /// Equivalent to `set_view_width(view_width() / factor)`.
    pub fn zoom_in(&mut self, factor: f32) {
        let w = self.view_width();
        self.set_view_width(w / factor.max(0.001));
    }

    /// Zoom out by a factor — objects appear `factor` times smaller.
    /// Equivalent to `set_view_width(view_width() * factor)`.
    pub fn zoom_out(&mut self, factor: f32) {
        let w = self.view_width();
        self.set_view_width(w * factor.max(0.001));
    }
}

/// Canonical default camera:
/// - Orthographic
/// - 16:9 world
/// - Looking down -Z
/// - Origin centered
impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 10.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            projection: Projection::Orthographic {
                width: DEFAULT_VIEW_WIDTH,
                height: DEFAULT_VIEW_HEIGHT,
                near: -100.0,
                far: 100.0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perspective_frame_bounds_intersect_the_layout_plane() {
        let camera = Camera {
            position: Vec3::new(0.0, 0.0, 10.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            projection: Projection::Perspective {
                fov_y_rad: std::f32::consts::FRAC_PI_2,
                aspect: 2.0,
                near: 0.1,
                far: 100.0,
            },
        };

        let bounds = camera.frame_bounds_at_z(0.0).unwrap();
        assert!(bounds.min.abs_diff_eq(glam::vec2(-20.0, -10.0), 1e-3));
        assert!(bounds.max.abs_diff_eq(glam::vec2(20.0, 10.0), 1e-3));
    }

    #[test]
    fn orthographic_frame_bounds_follow_camera_pan() {
        let camera = Camera {
            position: Vec3::new(3.0, -2.0, 10.0),
            target: Vec3::new(3.0, -2.0, 0.0),
            ..Camera::default()
        };

        let bounds = camera.frame_bounds_at_z(0.0).unwrap();
        assert!(bounds.center().abs_diff_eq(glam::vec2(3.0, -2.0), 1e-5));
        assert!((bounds.width() - DEFAULT_VIEW_WIDTH).abs() < 1e-5);
        assert!((bounds.height() - DEFAULT_VIEW_HEIGHT).abs() < 1e-5);
    }

    #[test]
    fn set_view_width_preserves_portrait_and_square_aspects() {
        let mut portrait = Camera::for_frame(Frame::portrait());
        portrait.set_view_width(4.5);
        let Projection::Orthographic { width, height, .. } = portrait.projection else {
            panic!("expected orthographic camera");
        };
        assert!((width - 4.5).abs() < 1e-6);
        assert!((height - 8.0).abs() < 1e-6);

        let mut square = Camera::for_frame(Frame::square());
        square.set_view_width(10.0);
        let Projection::Orthographic { width, height, .. } = square.projection else {
            panic!("expected orthographic camera");
        };
        assert!((width - 10.0).abs() < 1e-6);
        assert!((height - 10.0).abs() < 1e-6);
    }

    #[test]
    fn frame_bounds_reject_a_parallel_layout_plane() {
        let camera = Camera {
            position: Vec3::new(10.0, 0.0, 0.0),
            target: Vec3::ZERO,
            ..Camera::default()
        };

        assert!(camera.frame_bounds_at_z(0.0).is_none());
    }
}
