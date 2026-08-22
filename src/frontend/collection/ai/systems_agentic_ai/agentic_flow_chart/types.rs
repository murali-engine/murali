use glam::{Vec2, Vec3};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowChartDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowNodeShape {
    Rectangle,
    Rounded,
    Pill,
    Diamond,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowNodePlacement {
    RightOfPrevious,
    LeftOfPrevious,
    AbovePrevious,
    BelowPrevious,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeStep {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowNodeContentVisibility {
    Always,
    ActiveOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeAnimationStyle {
    /// Nodes appear instantly when revealed
    Instant,
    /// Nodes draw themselves like a write effect
    Write,
    /// Nodes drop from above with bounce
    Drop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EdgeAnimationStyle {
    /// Edges appear instantly when both source and target are revealed
    #[default]
    Instant,
    /// Edges draw progressively like a line drawing
    Write,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlowNodeArrival {
    pub node_index: usize,
    pub visit_index: usize,
    pub time: f32,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct NodeLayout {
    pub center: Vec3,
    pub size: Vec2,
}
