use glam::{Vec2, Vec4, vec2};

use crate::engine::scene::Scene;
use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Project, ProjectionCtx};

/// Controls how a child scene's local clock behaves after it starts.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SceneViewPlayback {
    /// Keep advancing local time. Timeline animations hold their final values,
    /// while frame-dependent updaters continue to run.
    Continuous,
    /// Stop advancing when the child timeline reaches its end.
    Once,
    /// Repeat a seekable interval of the child scene.
    Loop { duration: f32 },
    /// Keep the child at its current local time.
    Paused,
}

impl Default for SceneViewPlayback {
    fn default() -> Self {
        Self::Continuous
    }
}

/// A normal Murali scene presented as one drawable object inside another scene.
/// Multiple sibling views are supported; recursive SceneView compositing is not
/// part of the initial implementation.
pub struct SceneView {
    pub(crate) scene: Box<Scene>,
    pub(crate) size: Vec2,
    pub(crate) background: Option<Vec4>,
    pub(crate) corner_radius: f32,
    pub(crate) border_width: f32,
    pub(crate) border_color: Vec4,
    pub(crate) playback: SceneViewPlayback,
    pub(crate) start_time: f32,
    pub(crate) local_time_offset: f32,
    pub(crate) time_scale: f32,
    pub(crate) resolution: Option<(u32, u32)>,
}

impl SceneView {
    pub fn new(scene: Scene) -> Self {
        let (width, height) = scene.frame().logical_size();
        Self {
            scene: Box::new(scene),
            size: vec2(width, height),
            background: None,
            corner_radius: 0.0,
            border_width: 0.0,
            border_color: Vec4::ZERO,
            playback: SceneViewPlayback::Continuous,
            start_time: 0.0,
            local_time_offset: 0.0,
            time_scale: 1.0,
            resolution: None,
        }
    }

    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size.abs().max(Vec2::splat(f32::EPSILON));
        self
    }

    pub fn background(mut self, color: Vec4) -> Self {
        self.background = Some(color);
        self
    }

    pub fn transparent_background(mut self) -> Self {
        self.background = None;
        self
    }

    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius.max(0.0);
        self
    }

    pub fn border(mut self, width: f32, color: Vec4) -> Self {
        self.border_width = width.max(0.0);
        self.border_color = color;
        self
    }

    pub fn playback(mut self, playback: SceneViewPlayback) -> Self {
        self.playback = playback;
        self
    }

    /// Parent-scene time at which this view's local clock starts at zero.
    pub fn start_at(mut self, parent_time: f32) -> Self {
        self.start_time = parent_time.max(0.0);
        self
    }

    pub fn local_time_offset(mut self, offset: f32) -> Self {
        self.local_time_offset = offset.max(0.0);
        self
    }

    pub fn time_scale(mut self, scale: f32) -> Self {
        self.time_scale = scale.max(0.0);
        self
    }

    /// Overrides the automatically derived offscreen texture resolution.
    pub fn resolution(mut self, width: u32, height: u32) -> Self {
        self.resolution = Some((width.max(1), height.max(1)));
        self
    }

    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    pub fn scene_mut(&mut self) -> &mut Scene {
        &mut self.scene
    }

    pub fn local_time(&self) -> f32 {
        self.scene.scene_time
    }

    pub fn playback_mode(&self) -> SceneViewPlayback {
        self.playback
    }

    pub fn set_playback(&mut self, playback: SceneViewPlayback) {
        self.playback = playback;
    }

    pub fn set_time_scale(&mut self, scale: f32) {
        self.time_scale = scale.max(0.0);
    }

    pub fn restart_at(
        &mut self,
        parent_time: f32,
    ) -> Result<(), crate::engine::timeline::SeekError> {
        self.start_time = parent_time.max(0.0);
        self.local_time_offset = 0.0;
        self.scene.seek_to(0.0)
    }

    pub fn view_size(&self) -> Vec2 {
        self.size
    }

    pub(crate) fn inferred_parent_end_time(&self) -> Option<f32> {
        if matches!(
            self.playback,
            SceneViewPlayback::Loop { .. } | SceneViewPlayback::Paused
        ) || self.time_scale <= f32::EPSILON
        {
            return None;
        }
        let child_end = self.scene.timeline.as_ref()?.end_time();
        let remaining_local = (child_end - self.local_time_offset).max(0.0);
        Some(self.start_time + remaining_local / self.time_scale)
    }

    pub(crate) fn update_from_parent(
        &mut self,
        parent_time: f32,
    ) -> Result<(), crate::engine::timeline::SeekError> {
        let target_time = self.target_local_time(parent_time);
        let result = self.drive_to(target_time);
        self.scene.enforce_camera_frame_aspect();
        result
    }

    pub(crate) fn seek_from_parent(
        &mut self,
        parent_time: f32,
    ) -> Result<(), crate::engine::timeline::SeekError> {
        let target_time = self.target_local_time(parent_time);
        if self.playback == SceneViewPlayback::Paused {
            return Ok(());
        }
        let result = self.scene.seek_to(target_time);
        self.scene.enforce_camera_frame_aspect();
        result
    }

    fn target_local_time(&self, parent_time: f32) -> f32 {
        if self.playback == SceneViewPlayback::Paused {
            return self.scene.scene_time;
        }

        let elapsed = (parent_time - self.start_time).max(0.0) * self.time_scale;
        let local_time = self.local_time_offset + elapsed;
        match self.playback {
            SceneViewPlayback::Continuous | SceneViewPlayback::Paused => local_time,
            SceneViewPlayback::Once => {
                let end = self
                    .scene
                    .timeline
                    .as_ref()
                    .map_or(local_time, |timeline| timeline.end_time());
                local_time.min(end)
            }
            SceneViewPlayback::Loop { duration } if duration > f32::EPSILON => {
                local_time.rem_euclid(duration)
            }
            SceneViewPlayback::Loop { .. } => 0.0,
        }
    }

    fn drive_to(&mut self, target_time: f32) -> Result<(), crate::engine::timeline::SeekError> {
        if self.playback == SceneViewPlayback::Paused {
            return Ok(());
        }
        let delta = target_time - self.scene.scene_time;
        if delta < 0.0 {
            self.scene.seek_to(target_time)
        } else {
            self.scene.update(delta)
        }
    }
}

/// Non-rendering proxy whose props and bounds let a SceneView participate in
/// the existing layout and animation APIs.
#[derive(Debug, Clone)]
pub(crate) struct SceneViewProxy {
    size: Vec2,
}

impl SceneViewProxy {
    pub(crate) fn new(size: Vec2) -> Self {
        Self { size }
    }
}

impl Project for SceneViewProxy {
    fn project(&self, _ctx: &mut ProjectionCtx) {}
}

impl Bounded for SceneViewProxy {
    fn local_bounds(&self) -> Bounds {
        Bounds::from_center_size(Vec2::ZERO, self.size)
    }
}

#[cfg(test)]
mod tests {
    use super::{SceneView, SceneViewPlayback};
    use crate::engine::scene::Scene;
    use crate::engine::timeline::Timeline;
    use glam::{Vec3, vec2};

    #[test]
    fn local_clock_starts_at_zero_when_parent_reaches_start_time() {
        let mut view = SceneView::new(Scene::new()).start_at(2.0);

        view.update_from_parent(1.0).unwrap();
        assert_eq!(view.local_time(), 0.0);

        view.update_from_parent(3.25).unwrap();
        assert!((view.local_time() - 1.25).abs() < 1e-6);
    }

    #[test]
    fn time_scale_and_offset_map_parent_time_to_local_time() {
        let mut view = SceneView::new(Scene::new())
            .start_at(2.0)
            .local_time_offset(0.5)
            .time_scale(0.25);

        view.update_from_parent(6.0).unwrap();

        assert!((view.local_time() - 1.5).abs() < 1e-6);
    }

    #[test]
    fn loop_wraps_the_child_clock_independently() {
        let mut view =
            SceneView::new(Scene::new()).playback(SceneViewPlayback::Loop { duration: 2.0 });

        view.update_from_parent(1.5).unwrap();
        assert!((view.local_time() - 1.5).abs() < 1e-6);

        view.update_from_parent(2.5).unwrap();
        assert!((view.local_time() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn parent_animation_targets_the_scene_view_as_one_tattva() {
        let mut parent = Scene::new();
        let id = parent.add_scene_view(
            SceneView::new(Scene::new()).size(vec2(4.0, 2.0)),
            Vec3::ZERO,
        );
        let mut timeline = Timeline::new();
        timeline
            .animate(id)
            .at(0.0)
            .for_duration(1.0)
            .move_to(Vec3::new(3.0, 1.0, 0.0))
            .spawn();
        parent.play(timeline).unwrap();

        parent.update(1.0).unwrap();

        let bounds = parent.world_bounds(id).unwrap();
        assert!((bounds.center() - vec2(3.0, 1.0)).length() < 1e-6);
        assert!((parent.scene_view(id).unwrap().local_time() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn removing_the_proxy_removes_the_owned_child_scene() {
        let mut parent = Scene::new();
        let id = parent.add_scene_view(SceneView::new(Scene::new()), Vec3::ZERO);

        parent.remove_tattva(id);

        assert!(parent.scene_view(id).is_none());
    }
}
