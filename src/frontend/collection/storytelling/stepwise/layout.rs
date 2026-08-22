use glam::{Vec2, Vec3};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StepwiseDirection {
    #[default]
    Horizontal,
    Vertical,
}

/// Defines the spatial grid for a `Stepwise` storytelling component.
///
/// The layout determines where each node is placed and how the
/// deterministic routing engine calculates coordinates for `Left`, `Right`,
/// `Up`, and `Down` segments.
#[derive(Debug, Clone)]
pub struct StepwiseLayout {
    /// Distance between adjacent node centers.
    pub spacing: f32,
    /// Primary axis of the flow (Horizontal or Vertical).
    pub direction: StepwiseDirection,
}

impl Default for StepwiseLayout {
    fn default() -> Self {
        Self {
            spacing: 1.0,
            direction: StepwiseDirection::Horizontal,
        }
    }
}

impl StepwiseLayout {
    /// Creates a horizontal layout with nodes spaced by `spacing`.
    pub fn horizontal(spacing: f32) -> Self {
        Self {
            spacing,
            direction: StepwiseDirection::Horizontal,
        }
    }

    /// Creates a vertical layout with nodes spaced by `spacing`.
    pub fn vertical(spacing: f32) -> Self {
        Self {
            spacing,
            direction: StepwiseDirection::Vertical,
        }
    }

    /// World-space position for the node at `index`.
    /// Nodes are placed sequentially along the layout axis starting from (0,0,0).
    pub fn position_for(&self, index: usize) -> Vec3 {
        self.position_for_rank(index as isize)
    }

    /// World-space position for an arbitrary rank index.
    ///
    /// This allows the routing engine to calculate points that "go past"
    /// the first or last node (e.g., rank -1 or rank N+1), enabling
    /// clean boundary-clearing loops.
    pub fn position_for_rank(&self, rank: isize) -> Vec3 {
        let offset = rank as f32 * self.spacing;
        match self.direction {
            StepwiseDirection::Horizontal => Vec3::new(offset, 0.0, 0.0),
            StepwiseDirection::Vertical => Vec3::new(0.0, -offset, 0.0),
        }
    }

    /// Linearly interpolated position between two step indices.
    /// Used primarily by the signal flow dot to calculate its position at time `t`.
    pub fn lerp_position(&self, from: usize, to: usize, t: f32) -> Vec2 {
        let a = self.position_for(from);
        let b = self.position_for(to);
        Vec2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
    }
}
