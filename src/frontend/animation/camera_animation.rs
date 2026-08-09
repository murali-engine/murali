use crate::engine::camera::Projection;
use crate::engine::scene::Scene;
use crate::frontend::animation::{Animation, Ease};
use glam::Vec3;

/// Camera animation kinds targeting the semantic state of the viewpoint.
#[derive(Debug, Clone)]
pub enum CameraAnimKind {
    FrameTo {
        position: Vec3,
        target: Vec3,
    },
    MoveTo {
        to: Vec3,
    },
    LookAt {
        target: Vec3,
    },
    /// Orthographic zoom factor
    ZoomTo {
        zoom: f32,
    },
    /// Perspective field-of-view (radians)
    FovTo {
        fov_y_rad: f32,
    },
}

/// A concrete Animation implementation that mutates the Scene's camera.
pub struct CameraAnimate {
    pub kind: CameraAnimKind,
    pub ease: Ease,

    // Captured start states for interpolation
    from_pos: Option<Vec3>,
    from_target: Option<Vec3>,
    from_width: Option<f32>,
    from_height: Option<f32>,
    from_fov: Option<f32>,
}

impl CameraAnimate {
    pub fn new_frame(position: Vec3, target: Vec3, ease: Ease) -> Self {
        Self {
            kind: CameraAnimKind::FrameTo { position, target },
            ease,
            from_pos: None,
            from_target: None,
            from_width: None,
            from_height: None,
            from_fov: None,
        }
    }

    pub fn new_move(to: Vec3, ease: Ease) -> Self {
        Self {
            kind: CameraAnimKind::MoveTo { to },
            ease,
            from_pos: None,
            from_target: None,
            from_width: None,
            from_height: None,
            from_fov: None,
        }
    }

    pub fn new_lookat(target: Vec3, ease: Ease) -> Self {
        Self {
            kind: CameraAnimKind::LookAt { target },
            ease,
            from_pos: None,
            from_target: None,
            from_width: None,
            from_height: None,
            from_fov: None,
        }
    }

    pub fn new_zoom(zoom: f32, ease: Ease) -> Self {
        Self {
            kind: CameraAnimKind::ZoomTo { zoom },
            ease,
            from_pos: None,
            from_target: None,
            from_width: None,
            from_height: None,
            from_fov: None,
        }
    }

    pub fn new_fov(fov_y_rad: f32, ease: Ease) -> Self {
        Self {
            kind: CameraAnimKind::FovTo { fov_y_rad },
            ease,
            from_pos: None,
            from_target: None,
            from_width: None,
            from_height: None,
            from_fov: None,
        }
    }
}

impl Animation for CameraAnimate {
    fn on_start(&mut self, scene: &mut Scene) {
        let cam = &scene.camera;
        self.from_pos = Some(cam.position);
        self.from_target = Some(cam.target);

        match cam.projection {
            Projection::Orthographic { width, height, .. } => {
                self.from_width = Some(width);
                self.from_height = Some(height);
            }
            Projection::Perspective { fov_y_rad, .. } => {
                self.from_fov = Some(fov_y_rad);
            }
        }
    }

    fn apply_at(&mut self, scene: &mut Scene, t: f32) {
        // t is normalized (0.0 to 1.0) provided by the Timeline
        let k = self.ease.eval(t);
        let cam = scene.camera_mut();

        match self.kind {
            CameraAnimKind::FrameTo { position, target } => {
                let from_pos = self.from_pos.unwrap_or(cam.position);
                let from_target = self.from_target.unwrap_or(cam.target);
                cam.position = from_pos.lerp(position, k);
                cam.target = from_target.lerp(target, k);
            }
            CameraAnimKind::MoveTo { to } => {
                let from = self.from_pos.unwrap_or(cam.position);
                cam.position = from.lerp(to, k);
            }
            CameraAnimKind::LookAt { target } => {
                let from = self.from_target.unwrap_or(cam.target);
                cam.target = from.lerp(target, k);
            }
            CameraAnimKind::ZoomTo { zoom } => {
                if let Projection::Orthographic {
                    width,
                    height,
                    near,
                    far,
                } = cam.projection
                {
                    let fw = self.from_width.unwrap_or(width);
                    let fh = self.from_height.unwrap_or(height);
                    let zoom = zoom.max(0.001);
                    cam.projection = Projection::Orthographic {
                        width: fw + (fw / zoom - fw) * k,
                        height: fh + (fh / zoom - fh) * k,
                        near,
                        far,
                    };
                }
            }
            CameraAnimKind::FovTo { fov_y_rad } => {
                if let Projection::Perspective {
                    aspect, near, far, ..
                } = cam.projection
                {
                    let from = self.from_fov.unwrap_or(fov_y_rad);
                    cam.projection = Projection::Perspective {
                        fov_y_rad: from + (fov_y_rad - from) * k,
                        aspect,
                        near,
                        far,
                    };
                }
            }
        }
    }

    fn reapplies_terminal_state(&self) -> bool {
        true
    }
}
