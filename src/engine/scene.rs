//src/engine/scene.rs

pub use crate::frontend::props::{DrawableProps, SharedProps};

use glam::{Mat4, Quat, Vec2, Vec3, Vec4, vec2, vec3};
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

use crate::engine::camera::Camera;
use crate::engine::frame::Frame;
use crate::engine::scene_view::{SceneView, SceneViewProxy};
use crate::engine::timeline::{SeekError, Timeline};
use crate::frontend::layout::{Anchor, Bounds, Direction, anchor_for_direction, opposite_anchor};
use crate::frontend::props::DepthMode;
use crate::frontend::updater::UpdaterManager;
use crate::frontend::{DirtyFlags, IntoTattva, TattvaId, tattva_trait::TattvaTrait};
use crate::resource::texture::TextureImage;
use crate::validation::ValidationError;

#[derive(Debug, Clone, Default)]
pub struct ScreenshotCapture {
    pub times: Vec<f32>,
    pub names: Option<Vec<Option<PathBuf>>>,
}

impl ScreenshotCapture {
    pub fn new<I>(times: I) -> Self
    where
        I: IntoIterator<Item = f32>,
    {
        Self {
            times: times.into_iter().collect(),
            names: None,
        }
    }

    pub fn with_names<I, P>(mut self, names: I) -> Self
    where
        I: IntoIterator<Item = Option<P>>,
        P: Into<PathBuf>,
    {
        self.names = Some(names.into_iter().map(|name| name.map(Into::into)).collect());
        self
    }

    fn sort_by_time(&mut self) {
        let Some(names) = self.names.take() else {
            self.times
                .sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            return;
        };

        if names.len() != self.times.len() {
            self.names = Some(names);
            return;
        }

        let mut entries: Vec<_> = self.times.drain(..).zip(names).collect();
        entries.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let (times, names) = entries.into_iter().unzip();
        self.times = times;
        self.names = Some(names);
    }
}

#[derive(Debug, Clone)]
pub struct GifCapture {
    pub name: String,
    pub times: Vec<f32>,
}

impl GifCapture {
    pub fn new<I>(name: impl Into<String>, times: I) -> Self
    where
        I: IntoIterator<Item = f32>,
    {
        let mut times: Vec<f32> = times.into_iter().collect();
        times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        Self {
            name: name.into(),
            times,
        }
    }
}

/// The Scene represents the authoritative Frontend state.
pub struct Scene {
    /// Authoritative Tattvas (Source of Truth)
    /// We use TattvaTrait to allow different types (Circle, Latex, etc.) in one list.
    pub(crate) tattvas: HashMap<TattvaId, Box<dyn TattvaTrait>>,

    /// Time & Animation
    pub scene_time: f32,
    pub timeline: Option<Timeline>,
    pub screenshot_captures: Vec<ScreenshotCapture>,
    pub gif_captures: Vec<GifCapture>,

    /// Updaters - callbacks that run every frame
    pub updaters: UpdaterManager,

    /// Global State
    pub camera: Camera,
    frame: Frame,
    background: Option<Vec4>,
    pub global_model: Mat4,

    /// Independently animated child scenes presented as parent-scene objects.
    pub(crate) scene_views: HashMap<TattvaId, SceneView>,

    /// Identity bookkeeping
    next_tattva_id: TattvaId,
    removed_tattva_ids: Vec<TattvaId>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            tattvas: HashMap::new(),
            scene_time: 0.0,
            timeline: None,
            screenshot_captures: Vec::new(),
            gif_captures: Vec::new(),
            updaters: UpdaterManager::new(),
            camera: Camera::for_frame(Frame::default()),
            frame: Frame::default(),
            background: None,
            global_model: Mat4::IDENTITY,
            scene_views: HashMap::new(),
            next_tattva_id: 1,
            removed_tattva_ids: Vec::new(),
        }
    }

    /// Selects the scene's logical composition frame.
    ///
    /// Call this before frame-relative layout operations such as `to_edge`.
    pub fn with_frame(mut self, frame: Frame) -> Self {
        self.set_frame(frame);
        self
    }

    pub fn set_frame(&mut self, frame: Frame) {
        self.frame = frame;
        self.camera.set_frame(frame);
    }

    pub fn frame(&self) -> Frame {
        self.frame
    }

    /// Selects the scene's explicit clear/background color.
    pub fn with_background(mut self, color: Vec4) -> Self {
        self.set_background(color);
        self
    }

    pub fn set_background(&mut self, color: Vec4) {
        self.background = Some(color);
    }

    pub fn clear_background(&mut self) {
        self.background = None;
    }

    pub fn background(&self) -> Option<Vec4> {
        self.background
    }

    /// Adds a Tattva to the scene and returns its stable ID.
    pub fn add<T: TattvaTrait + 'static>(&mut self, mut tattva: T) -> TattvaId {
        let id = self.next_tattva_id;
        self.next_tattva_id += 1;

        // Tattvas need to know their own ID for animation targeting
        // (Assuming a .set_id() helper on the trait)
        tattva.set_id(id);

        self.tattvas.insert(id, Box::new(tattva));
        id
    }

    /// Adds a concrete state object to the scene at a given position.
    pub fn add_tattva<T>(&mut self, state: T, position: Vec3) -> TattvaId
    where
        T: crate::projection::Project + crate::frontend::layout::Bounded + Send + Sync + 'static,
    {
        let tattva = state.into_tattva();
        {
            let mut props = DrawableProps::write(&tattva.props);
            props.position = position;
        }
        self.add(tattva)
    }

    pub fn add_vector_latex(
        &mut self,
        equation: crate::frontend::collection::maths::notation::equation::VectorLatexEquation,
    ) -> crate::frontend::collection::maths::notation::equation::VectorEquationHandle {
        equation.add_to_scene(self)
    }

    pub fn add_vector_typst(
        &mut self,
        equation: crate::frontend::collection::maths::notation::equation::VectorTypstEquation,
    ) -> crate::frontend::collection::maths::notation::equation::VectorEquationHandle {
        equation.add_to_scene(self)
    }

    pub fn add_vector_formula_latex(
        &mut self,
        equation: crate::frontend::collection::maths::notation::equation::VectorLatexEquation,
    ) -> crate::frontend::collection::maths::notation::equation::VectorEquationHandle {
        self.add_vector_latex(equation)
    }

    pub fn add_vector_formula_typst(
        &mut self,
        equation: crate::frontend::collection::maths::notation::equation::VectorTypstEquation,
    ) -> crate::frontend::collection::maths::notation::equation::VectorEquationHandle {
        self.add_vector_typst(equation)
    }

    /// Hides a tattva completely from rendering until another animation or mutation reveals it.
    pub fn hide_tattva(&mut self, id: TattvaId) {
        self.hide(id);
    }

    pub fn add_screenshot_capture(&mut self, mut capture: ScreenshotCapture) {
        capture.sort_by_time();
        self.screenshot_captures.push(capture);
    }

    pub fn capture_screenshots<I>(&mut self, times: I)
    where
        I: IntoIterator<Item = f32>,
    {
        self.add_screenshot_capture(ScreenshotCapture::new(times));
    }

    pub fn capture_screenshots_named<I, P>(&mut self, names: I)
    where
        I: IntoIterator<Item = (f32, Option<P>)>,
        P: Into<PathBuf>,
    {
        let entries: Vec<(f32, Option<PathBuf>)> = names
            .into_iter()
            .map(|(time, name)| (time, name.map(Into::into)))
            .collect();
        let times = entries.iter().map(|(time, _)| *time).collect::<Vec<_>>();
        let names = entries
            .into_iter()
            .map(|(_, name)| name)
            .collect::<Vec<_>>();
        self.add_screenshot_capture(ScreenshotCapture::new(times).with_names(names));
    }

    pub fn add_gif_capture(&mut self, capture: GifCapture) {
        self.gif_captures.push(capture);
    }

    pub fn capture_gif<I>(&mut self, name: impl Into<String>, times: I)
    where
        I: IntoIterator<Item = f32>,
    {
        self.add_gif_capture(GifCapture::new(name, times));
    }

    /// Installs or replaces the scene's global timeline.
    pub fn play(&mut self, mut timeline: Timeline) -> Result<(), ValidationError> {
        self.enforce_camera_frame_aspect();
        timeline.validate()?;
        timeline.validate_for_scene(self)?;
        timeline.prepare(self);
        self.timeline = Some(timeline);
        Ok(())
    }

    pub fn add_textured_surface<F>(
        &mut self,
        u_range: (f32, f32),
        v_range: (f32, f32),
        f: F,
        texture: TextureImage,
        position: Vec3,
    ) -> TattvaId
    where
        F: Fn(f32, f32) -> Vec3 + Send + Sync + 'static,
    {
        let surface =
            crate::frontend::collection::maths::calculus::parametric_surface::ParametricSurface::new(
                u_range, v_range, f,
            )
            .with_texture(texture);
        self.add_tattva(surface, position)
    }

    pub fn add_textured_surface_with_path(
        &mut self,
        surface: crate::frontend::collection::maths::calculus::parametric_surface::ParametricSurface,
        texture_path: impl AsRef<Path>,
        position: Vec3,
    ) -> anyhow::Result<TattvaId> {
        let surface = surface.with_texture_path(texture_path)?;
        Ok(self.add_tattva(surface, position))
    }

    /// Adds an independently animated child scene and returns its parent-facing ID.
    pub fn add_scene_view(&mut self, view: SceneView, position: Vec3) -> TattvaId {
        let size = view.view_size();
        let id = self.add_tattva(SceneViewProxy::new(size), position);
        self.scene_views.insert(id, view);
        id
    }

    pub fn scene_view(&self, id: TattvaId) -> Option<&SceneView> {
        self.scene_views.get(&id)
    }

    pub fn scene_view_mut(&mut self, id: TattvaId) -> Option<&mut SceneView> {
        self.scene_views.get_mut(&id)
    }

    /// Retrieves a Tattva for internal mutation with explicit dirty-flag handling.
    pub(crate) fn get_tattva_any_mut(
        &mut self,
        id: TattvaId,
    ) -> Option<&mut (dyn TattvaTrait + '_)> {
        match self.tattvas.get_mut(&id) {
            Some(b) => Some(b.as_mut()),
            None => None,
        }
    }

    pub fn get_tattva_any(&self, id: TattvaId) -> Option<&(dyn TattvaTrait + '_)> {
        match self.tattvas.get(&id) {
            Some(b) => Some(b.as_ref()),
            None => None,
        }
    }

    pub fn get_tattva_typed<T: 'static>(
        &self,
        id: TattvaId,
    ) -> Option<&crate::frontend::Tattva<T>> {
        self.get_tattva_any(id)?
            .as_any()
            .downcast_ref::<crate::frontend::Tattva<T>>()
    }

    pub fn get_tattva_typed_mut<T: 'static>(
        &mut self,
        id: TattvaId,
    ) -> Option<&mut crate::frontend::Tattva<T>> {
        let tattva = self
            .tattvas
            .get_mut(&id)?
            .as_any_mut()
            .downcast_mut::<crate::frontend::Tattva<T>>()?;
        tattva.mark_dirty(DirtyFlags::ALL);
        Some(tattva)
    }

    /// Primary update loop for the frontend.
    pub fn update(&mut self, dt: f32) -> Result<(), SeekError> {
        if dt < 0.0 {
            return self.seek_to(self.scene_time + dt);
        }
        self.scene_time += dt;

        // We temporarily move the timeline out to avoid borrow checker conflicts with `self`.
        if let Some(mut timeline) = self.timeline.take() {
            let result = timeline.update(self.scene_time, self);
            self.timeline = Some(timeline);
            result?;
        }

        // 2. Update traced paths
        self.update_traced_paths();

        // 3. Run all updaters
        let updaters = std::mem::take(&mut self.updaters);
        updaters.update_all(self, dt);
        self.updaters = updaters;

        for view in self.scene_views.values_mut() {
            view.update_from_parent(self.scene_time)?;
        }
        Ok(())
    }

    /// Reconstructs seekable timeline state at an absolute time.
    pub fn seek_to(&mut self, scene_time: f32) -> Result<(), SeekError> {
        let scene_time = if scene_time.is_finite() {
            scene_time.max(0.0)
        } else {
            0.0
        };
        if !self.updaters.is_empty() {
            return Err(SeekError::FrameDependentUpdaters {
                count: self.updaters.len(),
            });
        }

        let traced_path_count = self.traced_path_count();
        if traced_path_count > 0 {
            return Err(SeekError::HistoryDependentTracedPaths {
                count: traced_path_count,
            });
        }

        if let Some(mut timeline) = self.timeline.take() {
            let result = timeline.seek_to(scene_time, self);
            self.timeline = Some(timeline);
            result?;
        }
        for view in self.scene_views.values_mut() {
            view.seek_from_parent(scene_time)?;
        }
        self.scene_time = scene_time;
        Ok(())
    }

    fn traced_path_count(&self) -> usize {
        use crate::frontend::Tattva;
        use crate::frontend::collection::utility::TracedPath;

        self.tattvas
            .values()
            .filter(|tattva| {
                tattva.as_any().is::<TracedPath>() || tattva.as_any().is::<Tattva<TracedPath>>()
            })
            .count()
    }

    /// Update all traced paths in the scene
    fn update_traced_paths(&mut self) {
        use crate::frontend::Tattva;
        use crate::frontend::collection::utility::TracedPath;

        // Collect IDs of traced paths and their tracked objects
        let mut traced_paths: Vec<(TattvaId, TattvaId)> = Vec::new();

        for (id, tattva) in self.tattvas.iter() {
            // TracedPath is wrapped in Tattva<TracedPath>
            if let Some(wrapped) = tattva.as_any().downcast_ref::<Tattva<TracedPath>>() {
                traced_paths.push((*id, wrapped.state.tracked_object_id));
            }
        }

        // Update each traced path
        for (traced_path_id, tracked_object_id) in traced_paths {
            // Get the tracked object's position and rotation
            if let Some(tracked_obj) = self.get_tattva_any(tracked_object_id) {
                let props = DrawableProps::read(tracked_obj.props());
                let obj_pos = props.position;
                let obj_rot = props.rotation;
                drop(props);

                // Get the traced path and record the point
                if let Some(tattva) = self.tattvas.get_mut(&traced_path_id) {
                    if let Some(wrapped) = tattva.as_any_mut().downcast_mut::<Tattva<TracedPath>>()
                    {
                        // Compute the traced point position using the point function
                        let traced_point = (wrapped.state.point_fn)(obj_pos, obj_rot);
                        let old_count = wrapped.state.path_points.len();
                        wrapped.state.record_point(traced_point);
                        let new_count = wrapped.state.path_points.len();

                        // If a new point was added, mark the tattva as dirty
                        if new_count > old_count {
                            wrapped.mark_dirty(DirtyFlags::GEOMETRY);
                        }
                    }
                }
            }
        }
    }

    /// Returns an iterator over all Tattvas for the Sync Boundary to process.
    pub(crate) fn tattvas_iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (&TattvaId, &mut Box<dyn TattvaTrait>)> {
        self.tattvas.iter_mut()
    }

    pub fn local_bounds(&self, id: TattvaId) -> Option<Bounds> {
        self.get_tattva_any(id).map(|t| t.local_bounds())
    }

    pub fn world_bounds(&self, id: TattvaId) -> Option<Bounds> {
        let tattva = self.get_tattva_any(id)?;
        let local = tattva.local_bounds();
        let props = DrawableProps::read(tattva.props());
        Some(local.transform_3d(props.model_matrix()))
    }

    pub fn anchor_position(&self, id: TattvaId, anchor: Anchor) -> Option<Vec2> {
        self.world_bounds(id).map(|b| b.anchor(anchor))
    }

    pub fn position(&self, id: TattvaId) -> Option<Vec3> {
        self.get_tattva_any(id)
            .map(|tattva| DrawableProps::read(tattva.props()).position)
    }

    pub fn set_position_2d(&mut self, id: TattvaId, position: Vec2) {
        if let Some(tattva) = self.get_tattva_any_mut(id) {
            let mut props = DrawableProps::write(tattva.props());
            props.position = vec3(position.x, position.y, props.position.z);
            drop(props);
            tattva.mark_dirty(DirtyFlags::TRANSFORM);
        }
    }

    pub fn set_position_3d(&mut self, id: TattvaId, position: Vec3) {
        if let Some(tattva) = self.get_tattva_any_mut(id) {
            let mut props = DrawableProps::write(tattva.props());
            props.position = position;
            drop(props);
            tattva.mark_dirty(DirtyFlags::TRANSFORM);
        }
    }

    pub fn set_scale(&mut self, id: TattvaId, scale: Vec3) {
        if let Some(tattva) = self.get_tattva_any_mut(id) {
            let mut props = DrawableProps::write(tattva.props());
            props.scale = scale;
            drop(props);
            tattva.mark_dirty(DirtyFlags::TRANSFORM);
        }
    }

    pub fn set_rotation(&mut self, id: TattvaId, rotation: Quat) {
        if let Some(tattva) = self.get_tattva_any_mut(id) {
            let mut props = DrawableProps::write(tattva.props());
            props.rotation = rotation;
            drop(props);
            tattva.mark_dirty(DirtyFlags::TRANSFORM);
        }
    }

    pub fn set_opacity(&mut self, id: TattvaId, opacity: f32) {
        if let Some(tattva) = self.get_tattva_any_mut(id) {
            let mut props = DrawableProps::write(tattva.props());
            props.opacity = opacity.clamp(0.0, 1.0);
            props.visible = props.opacity > 0.001;
            drop(props);
            tattva.mark_dirty(DirtyFlags::STYLE | DirtyFlags::VISIBILITY);
        }
    }

    /// Sets the painter-order layer used by 2D rendering.
    /// Higher layers are drawn later and therefore appear on top.
    pub fn set_layer(&mut self, id: TattvaId, layer: i32) {
        if let Some(tattva) = self.get_tattva_any_mut(id) {
            let mut props = DrawableProps::write(tattva.props());
            props.layer = layer;
            drop(props);
            tattva.mark_dirty(DirtyFlags::STYLE);
        }
    }

    /// Selects world-depth or always-on-top overlay rendering for perspective scenes.
    pub fn set_depth_mode(&mut self, id: TattvaId, depth_mode: DepthMode) {
        if let Some(tattva) = self.get_tattva_any_mut(id) {
            let mut props = DrawableProps::write(tattva.props());
            props.depth_mode = depth_mode;
            drop(props);
            tattva.mark_dirty(DirtyFlags::STYLE);
        }
    }

    pub fn show(&mut self, id: TattvaId) {
        self.set_opacity(id, 1.0);
    }

    pub fn hide(&mut self, id: TattvaId) {
        self.set_opacity(id, 0.0);
    }

    pub fn align_to(&mut self, moving: TattvaId, target: TattvaId, anchor: Anchor) {
        let Some(moving_anchor) = self.anchor_position(moving, anchor) else {
            return;
        };
        let Some(target_anchor) = self.anchor_position(target, anchor) else {
            return;
        };
        let delta = target_anchor - moving_anchor;
        if let Some(tattva) = self.get_tattva_any(moving) {
            let position = DrawableProps::read(tattva.props()).position.truncate();
            let new_position = match anchor {
                Anchor::Up | Anchor::Down => glam::vec2(position.x, position.y + delta.y),
                Anchor::Left | Anchor::Right => glam::vec2(position.x + delta.x, position.y),
                _ => position + delta,
            };
            self.set_position_2d(moving, new_position);
        }
    }

    pub fn next_to(
        &mut self,
        moving: TattvaId,
        target: TattvaId,
        direction: Direction,
        padding: f32,
    ) {
        let Some(target_bounds) = self.world_bounds(target) else {
            return;
        };
        let Some(moving_bounds) = self.world_bounds(moving) else {
            return;
        };

        let moving_anchor = opposite_anchor(direction);
        let target_anchor = anchor_for_direction(direction);
        let target_point = target_bounds.anchor(target_anchor);
        let moving_point = moving_bounds.anchor(moving_anchor);

        let offset = match direction {
            Direction::Up => vec2(0.0, padding),
            Direction::Down => vec2(0.0, -padding),
            Direction::Left => vec2(-padding, 0.0),
            Direction::Right => vec2(padding, 0.0),
        };

        let delta = target_point + offset - moving_point;
        if let Some(tattva) = self.get_tattva_any(moving) {
            let position = DrawableProps::read(tattva.props()).position.truncate();
            self.set_position_2d(moving, position + delta);
        }
    }

    pub fn to_edge(&mut self, id: TattvaId, direction: Direction, margin: f32) {
        let Some(moving_bounds) = self.world_bounds(id) else {
            return;
        };
        let Some(frame) = self.frame_bounds() else {
            return;
        };
        let target_anchor = anchor_for_direction(direction);
        let moving_anchor = target_anchor;
        let edge_point = frame.anchor(target_anchor);
        let margin_offset = match direction {
            Direction::Up => vec2(0.0, -margin),
            Direction::Down => vec2(0.0, margin),
            Direction::Left => vec2(margin, 0.0),
            Direction::Right => vec2(-margin, 0.0),
        };
        let delta = edge_point + margin_offset - moving_bounds.anchor(moving_anchor);
        if let Some(tattva) = self.get_tattva_any(id) {
            let position = DrawableProps::read(tattva.props()).position.truncate();
            self.set_position_2d(id, position + delta);
        }
    }

    /// Returns the camera frame projected onto Murali's 2D layout plane (`z = 0`).
    pub fn frame_bounds(&self) -> Option<Bounds> {
        let mut camera = self.camera;
        camera.set_aspect_ratio(self.frame.aspect_ratio());
        camera.frame_bounds_at_z(0.0)
    }

    pub fn clear(&mut self) {
        self.removed_tattva_ids.extend(self.tattvas.keys().copied());
        self.removed_tattva_ids.sort_unstable();
        self.removed_tattva_ids.dedup();
        self.tattvas.clear();
        self.timeline = None;
        self.screenshot_captures.clear();
        self.gif_captures.clear();
        self.updaters.clear();
        self.scene_views.clear();
        self.scene_time = 0.0;
        self.next_tattva_id = 1;
        self.camera = Camera::for_frame(self.frame);
    }

    // camera
    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    pub(crate) fn enforce_camera_frame_aspect(&mut self) {
        self.camera.set_aspect_ratio(self.frame.aspect_ratio());
    }

    /// Replaces the implementation of an existing Tattva.
    /// This is used for shape morphing where we swap types (e.g., Circle -> Path).
    pub fn replace_tattva(&mut self, id: TattvaId, mut tattva: Box<dyn TattvaTrait>) {
        tattva.set_id(id);
        tattva.mark_dirty(DirtyFlags::ALL);
        self.tattvas.insert(id, tattva);
    }

    /// Removes a tattva from the scene and records its ID so the backend can
    /// despawn any cached render entities tied to it.
    pub fn remove_tattva(&mut self, id: TattvaId) -> Option<Box<dyn TattvaTrait>> {
        self.scene_views.remove(&id);
        let removed = self.tattvas.remove(&id);
        if removed.is_some() {
            self.removed_tattva_ids.push(id);
        }
        removed
    }

    pub fn take_removed_tattva_ids(&mut self) -> Vec<TattvaId> {
        std::mem::take(&mut self.removed_tattva_ids)
    }

    /// Add an updater callback for a tattva
    /// The callback will be called every frame with (scene, tattva_id, dt)
    /// Returns an index that can be used to remove the updater later
    pub fn add_updater<F>(&mut self, tattva_id: TattvaId, callback: F) -> usize
    where
        F: Fn(&mut Scene, TattvaId, f32) + Send + Sync + 'static,
    {
        self.updaters.add_updater(tattva_id, callback)
    }

    /// Remove an updater by its index
    pub fn remove_updater(&mut self, index: usize) {
        self.updaters.remove_updater(index);
    }

    /// Remove all updaters for a specific tattva
    pub fn remove_updaters_for(&mut self, tattva_id: TattvaId) {
        self.updaters.remove_updaters_for_tattva(tattva_id);
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::camera::Projection;
    use crate::frontend::collection::primitives::square::Square;
    use glam::Vec4;

    #[test]
    fn clear_queues_every_tattva_for_backend_removal_before_reusing_ids() {
        let mut scene = Scene::new();
        let first = scene.add_tattva(Square::new(1.0, Vec4::ONE), Vec3::ZERO);
        let second = scene.add_tattva(Square::new(1.0, Vec4::ONE), Vec3::ZERO);
        scene.remove_tattva(first);

        scene.clear();

        assert!(scene.tattvas.is_empty());
        assert_eq!(scene.take_removed_tattva_ids(), vec![first, second]);
        assert_eq!(
            scene.add_tattva(Square::new(1.0, Vec4::ONE), Vec3::ZERO),
            first
        );
    }

    #[test]
    fn portrait_frame_is_available_to_layout_before_rendering() {
        let mut scene = Scene::new().with_frame(Frame::portrait());
        let id = scene.add_tattva(Square::new(1.0, Vec4::ONE), Vec3::ZERO);

        scene.to_edge(id, Direction::Up, 0.5);

        let frame = scene.frame_bounds().unwrap();
        let bounds = scene.world_bounds(id).unwrap();
        assert!((frame.width() - 9.0).abs() < 1e-5);
        assert!((frame.height() - 16.0).abs() < 1e-5);
        assert!((frame.max.y - bounds.max.y - 0.5).abs() < 1e-5);
    }

    #[test]
    fn clear_preserves_the_scene_frame() {
        let mut scene = Scene::new().with_frame(Frame::square());
        scene.camera_mut().set_view_width(5.0);

        scene.clear();

        assert_eq!(scene.frame(), Frame::square());
        let Projection::Orthographic { width, height, .. } = scene.camera().projection else {
            panic!("expected orthographic camera");
        };
        assert_eq!((width, height), (16.0, 16.0));
    }

    #[test]
    fn scene_frame_remains_authoritative_after_direct_projection_mutation() {
        let mut scene = Scene::new().with_frame(Frame::square());
        scene.camera.projection = Projection::Orthographic {
            width: 10.0,
            height: 5.0,
            near: -100.0,
            far: 100.0,
        };

        let bounds = scene.frame_bounds().unwrap();
        assert!((bounds.width() - 10.0).abs() < 1e-5);
        assert!((bounds.height() - 10.0).abs() < 1e-5);

        scene.enforce_camera_frame_aspect();
        let Projection::Orthographic { width, height, .. } = scene.camera.projection else {
            panic!("expected orthographic camera");
        };
        assert_eq!((width, height), (10.0, 10.0));
    }

    #[test]
    fn replace_tattva_preserves_scene_identity_and_forces_backend_rebuild() {
        let mut scene = Scene::new();
        let id = scene.add_tattva(Square::new(1.0, Vec4::ONE), Vec3::ZERO);
        let mut replacement =
            crate::frontend::Tattva::new(id + 100, Square::new(2.0, Vec4::new(0.2, 0.4, 0.6, 1.0)));
        replacement.clear_all_dirty();

        scene.replace_tattva(id, Box::new(replacement));

        let replacement = scene.get_tattva_any(id).unwrap();
        assert_eq!(replacement.id(), id);
        assert_eq!(replacement.dirty_flags(), DirtyFlags::ALL);
    }

    #[test]
    fn typed_mutable_access_automatically_marks_state_dirty() {
        let mut scene = Scene::new();
        let id = scene.add_tattva(Square::new(1.0, Vec4::ONE), Vec3::ZERO);
        scene.tattvas.get_mut(&id).unwrap().clear_all_dirty();

        scene.get_tattva_typed_mut::<Square>(id).unwrap().state.size = 2.0;

        assert_eq!(
            scene.get_tattva_any(id).unwrap().dirty_flags(),
            DirtyFlags::ALL
        );
    }

    #[test]
    fn failed_typed_mutable_access_does_not_mark_state_dirty() {
        use crate::frontend::collection::primitives::circle::Circle;

        let mut scene = Scene::new();
        let id = scene.add_tattva(Square::new(1.0, Vec4::ONE), Vec3::ZERO);
        scene.tattvas.get_mut(&id).unwrap().clear_all_dirty();

        assert!(scene.get_tattva_typed_mut::<Circle>(id).is_none());
        assert_eq!(
            scene.get_tattva_any(id).unwrap().dirty_flags(),
            DirtyFlags::NONE
        );
    }

    #[test]
    fn world_bounds_include_signed_scale_and_rotation() {
        let mut scene = Scene::new();
        let id = scene.add_tattva(Square::new(2.0, Vec4::ONE), Vec3::new(3.0, 4.0, 0.0));
        scene.set_scale(id, Vec3::new(-2.0, 1.0, 1.0));
        scene.set_rotation(id, Quat::from_rotation_z(std::f32::consts::FRAC_PI_2));

        let bounds = scene.world_bounds(id).unwrap();
        assert!(bounds.min.abs_diff_eq(vec2(2.0, 2.0), 1e-5));
        assert!(bounds.max.abs_diff_eq(vec2(4.0, 6.0), 1e-5));
    }

    #[test]
    fn next_to_preserves_padding_for_transformed_objects() {
        let mut scene = Scene::new();
        let target = scene.add_tattva(Square::new(2.0, Vec4::ONE), Vec3::ZERO);
        let moving = scene.add_tattva(Square::new(2.0, Vec4::ONE), Vec3::ZERO);
        scene.set_scale(moving, Vec3::new(-2.0, 1.0, 1.0));
        scene.set_rotation(moving, Quat::from_rotation_z(std::f32::consts::FRAC_PI_2));

        scene.next_to(moving, target, Direction::Right, 0.5);

        let target_bounds = scene.world_bounds(target).unwrap();
        let moving_bounds = scene.world_bounds(moving).unwrap();
        assert!((moving_bounds.min.x - target_bounds.max.x - 0.5).abs() < 1e-5);
    }

    #[test]
    fn perspective_to_edge_uses_the_layout_plane_frame() {
        let mut scene = Scene::new();
        scene.camera.projection = crate::engine::camera::Projection::Perspective {
            fov_y_rad: std::f32::consts::FRAC_PI_2,
            aspect: 2.0,
            near: 0.1,
            far: 100.0,
        };
        let id = scene.add_tattva(Square::new(2.0, Vec4::ONE), Vec3::ZERO);

        scene.to_edge(id, Direction::Up, 0.5);

        let frame = scene.frame_bounds().unwrap();
        let bounds = scene.world_bounds(id).unwrap();
        assert!((frame.max.y - bounds.max.y - 0.5).abs() < 1e-3);
    }

    #[test]
    fn seeking_with_frame_dependent_updaters_returns_an_error() {
        let mut scene = Scene::new();
        let id = scene.add_tattva(Square::new(1.0, Vec4::ONE), Vec3::ZERO);
        scene.add_updater(id, |_scene, _id, _dt| {});

        assert_eq!(
            scene.seek_to(1.0).unwrap_err(),
            SeekError::FrameDependentUpdaters { count: 1 }
        );
        assert_eq!(scene.scene_time, 0.0);
    }

    #[test]
    fn seeking_with_traced_path_history_returns_an_error() {
        use crate::frontend::collection::utility::TracedPath;

        let mut scene = Scene::new();
        let tracked = scene.add_tattva(Square::new(1.0, Vec4::ONE), Vec3::ZERO);
        scene.add_tattva(
            TracedPath::new(tracked, |position, _rotation| position, Vec4::ONE, 0.02),
            Vec3::ZERO,
        );

        assert_eq!(
            scene.seek_to(1.0).unwrap_err(),
            SeekError::HistoryDependentTracedPaths { count: 1 }
        );
        assert_eq!(scene.scene_time, 0.0);
    }

    #[test]
    fn named_screenshot_sorting_keeps_times_and_names_together() {
        let mut scene = Scene::new();
        scene.capture_screenshots_named([(2.0, Some("second.png")), (1.0, Some("first.png"))]);

        let capture = &scene.screenshot_captures[0];
        assert_eq!(capture.times, vec![1.0, 2.0]);
        assert_eq!(
            capture.names.as_ref().unwrap(),
            &vec![
                Some(PathBuf::from("first.png")),
                Some(PathBuf::from("second.png")),
            ]
        );
    }

    #[test]
    fn added_screenshot_capture_normalizes_chained_names() {
        let mut scene = Scene::new();
        scene.add_screenshot_capture(
            ScreenshotCapture::new([2.0, 1.0]).with_names([Some("second.png"), Some("first.png")]),
        );

        let capture = &scene.screenshot_captures[0];
        assert_eq!(capture.times, vec![1.0, 2.0]);
        assert_eq!(
            capture.names.as_ref().unwrap(),
            &vec![
                Some(PathBuf::from("first.png")),
                Some(PathBuf::from("second.png")),
            ]
        );
    }
}
