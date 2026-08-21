// src/camera/controller/orbit.rs

use crate::engine::camera::Camera;
use glam::{Vec2, Vec3};

pub struct OrbitCameraController {
    pub yaw: f32,
    pub pitch: f32,
    pub radius: f32,
    pub sensitivity: f32,
}

impl OrbitCameraController {
    pub fn new(radius: f32) -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            radius,
            sensitivity: 0.005,
        }
    }

    pub fn handle_mouse_drag(&mut self, delta: Vec2, cam: &mut Camera) {
        self.yaw += delta.x * self.sensitivity;
        self.pitch = (self.pitch + delta.y * self.sensitivity).clamp(-1.5, 1.5);

        let dir = Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        );

        cam.position = cam.target - dir * self.radius;
    }

    pub fn zoom(&mut self, amount: f32, cam: &mut Camera) {
        let offset = cam.position - cam.target;
        let Some(direction) = offset.try_normalize() else {
            return;
        };
        let factor = (1.0 - amount * 0.1).clamp(0.1, 10.0);
        self.radius = (offset.length() * factor).max(0.05);
        cam.position = cam.target + direction * self.radius;
    }

    pub fn sync_from_camera(&mut self, cam: &Camera) {
        let offset = cam.target - cam.position;
        let Some(v) = offset.try_normalize() else {
            return;
        };
        self.pitch = v.y.asin();
        self.yaw = v.z.atan2(v.x);
        self.radius = offset.length();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scroll_zoom_updates_camera_and_controller_radius() {
        let mut camera = Camera::default();
        camera.position = Vec3::new(0.0, 0.0, 10.0);
        camera.target = Vec3::ZERO;
        let mut controller = OrbitCameraController::new(10.0);

        controller.zoom(2.0, &mut camera);

        assert!((controller.radius - 8.0).abs() < 1e-5);
        assert!((camera.position.z - 8.0).abs() < 1e-5);
    }

    #[test]
    fn synchronized_orbit_does_not_jump_on_first_drag() {
        let mut camera = Camera::default();
        camera.position = Vec3::new(2.0, 1.0, 8.0);
        camera.target = Vec3::new(0.0, 0.5, 0.0);
        let original_position = camera.position;
        let mut controller = OrbitCameraController::new(10.0);

        controller.sync_from_camera(&camera);
        controller.handle_mouse_drag(Vec2::ZERO, &mut camera);

        assert!(camera.position.abs_diff_eq(original_position, 1e-5));
    }
}
