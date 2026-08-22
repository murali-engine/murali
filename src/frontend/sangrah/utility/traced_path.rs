use crate::frontend::DirtyFlags;
use crate::frontend::TattvaId;
use crate::frontend::layout::Bounded;
use crate::frontend::layout::Bounds;
use crate::frontend::props::SharedProps;
use crate::frontend::tattva_trait::TattvaTrait;
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};
/// TracedPath: Tracks and visualizes the path of a point on a moving object
/// Similar to Manim's TracedPath animation
use glam::{Vec2, Vec3, Vec4};

/// A traced path that records the trajectory of a point on a moving object
pub struct TracedPath {
    /// The ID of the object being traced
    pub tracked_object_id: TattvaId,
    /// Function to get the point position relative to the tracked object
    /// Takes the object's world position and rotation, returns the traced point's world position
    pub point_fn: Box<dyn Fn(Vec3, glam::Quat) -> Vec3 + Send + Sync>,
    /// Recorded path points
    pub path_points: Vec<Vec3>,
    /// Maximum number of points to keep (for memory efficiency)
    pub max_points: usize,
    /// Color of the traced path
    pub color: Vec4,
    /// Thickness of the traced line
    pub thickness: f32,
    /// Whether to record new points
    pub is_recording: bool,
    /// Last recorded position (to avoid duplicate points)
    last_recorded_pos: Option<Vec3>,
    /// Minimum distance between recorded points
    pub min_distance: f32,
    /// Properties for the tattva trait
    props: SharedProps,
    /// Dirty flags
    dirty: DirtyFlags,
    /// ID
    id: TattvaId,
}

impl TracedPath {
    /// Create a new traced path that tracks a point on an object
    ///
    /// # Arguments
    /// * `tracked_object_id` - The ID of the object to track
    /// * `point_fn` - Function that returns the traced point position given the object's position and rotation
    /// * `color` - Color of the traced path
    /// * `thickness` - Thickness of the traced line
    pub fn new<F>(tracked_object_id: TattvaId, point_fn: F, color: Vec4, thickness: f32) -> Self
    where
        F: Fn(Vec3, glam::Quat) -> Vec3 + Send + Sync + 'static,
    {
        Self {
            tracked_object_id,
            point_fn: Box::new(point_fn),
            path_points: Vec::new(),
            max_points: 10000,
            color,
            thickness,
            is_recording: true,
            last_recorded_pos: None,
            min_distance: 0.01,
            props: SharedProps::default(),
            dirty: DirtyFlags::ALL,
            id: 0,
        }
    }

    /// Set the maximum number of points to keep
    pub fn with_max_points(mut self, max_points: usize) -> Self {
        self.max_points = max_points;
        self
    }

    /// Set the minimum distance between recorded points
    pub fn with_min_distance(mut self, min_distance: f32) -> Self {
        self.min_distance = min_distance;
        self
    }

    /// Start recording the path
    pub fn start_recording(&mut self) {
        self.is_recording = true;
    }

    /// Stop recording the path
    pub fn stop_recording(&mut self) {
        self.is_recording = false;
    }

    /// Clear all recorded points
    pub fn clear(&mut self) {
        self.path_points.clear();
        self.last_recorded_pos = None;
    }

    /// Record a new point if conditions are met
    pub fn record_point(&mut self, point: Vec3) {
        if !self.is_recording {
            return;
        }

        // Check minimum distance
        if let Some(last_pos) = self.last_recorded_pos {
            if (point - last_pos).length() < self.min_distance {
                return;
            }
        }

        self.path_points.push(point);
        self.last_recorded_pos = Some(point);

        // Maintain max points limit
        if self.path_points.len() > self.max_points {
            self.path_points.remove(0);
        }
    }

    /// Get the current path as a vector of 2D points (for rendering)
    pub fn get_path_2d(&self) -> Vec<Vec2> {
        self.path_points
            .iter()
            .map(|p| Vec2::new(p.x, p.y))
            .collect()
    }

    /// Get the number of recorded points
    pub fn point_count(&self) -> usize {
        self.path_points.len()
    }
}

impl Bounded for TracedPath {
    fn local_bounds(&self) -> Bounds {
        if self.path_points.is_empty() {
            return Bounds::new(Vec2::ZERO, Vec2::ZERO);
        }

        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for point in &self.path_points {
            min_x = min_x.min(point.x);
            max_x = max_x.max(point.x);
            min_y = min_y.min(point.y);
            max_y = max_y.max(point.y);
        }

        Bounds::new(Vec2::new(min_x, min_y), Vec2::new(max_x, max_y))
    }
}

impl Project for TracedPath {
    fn project(&self, ctx: &mut ProjectionCtx) {
        if self.path_points.len() < 2 {
            return;
        }

        // Draw line segments connecting the path points
        for i in 0..self.path_points.len() - 1 {
            let start = self.path_points[i];
            let end = self.path_points[i + 1];

            // Emit a line primitive
            ctx.emit(RenderPrimitive::Line {
                start,
                end,
                thickness: self.thickness,
                color: self.color,
                dash_length: 0.0,
                gap_length: 0.0,
                dash_offset: 0.0,
            });
        }
    }
}

impl TattvaTrait for TracedPath {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn props(&self) -> &SharedProps {
        &self.props
    }

    fn local_bounds(&self) -> Bounds {
        Bounded::local_bounds(self)
    }

    fn dirty_flags(&self) -> DirtyFlags {
        self.dirty
    }

    fn mark_dirty(&mut self, flags: DirtyFlags) {
        self.dirty |= flags;
    }

    fn clear_dirty(&mut self, flags: DirtyFlags) {
        self.dirty = self.dirty.without(flags);
    }

    fn clear_all_dirty(&mut self) {
        self.dirty = DirtyFlags::NONE;
    }

    fn set_id(&mut self, id: TattvaId) {
        self.id = id;
    }

    fn id(&self) -> TattvaId {
        self.id
    }

    fn project(&self, ctx: &mut ProjectionCtx) {
        Project::project(self, ctx);
    }
}
