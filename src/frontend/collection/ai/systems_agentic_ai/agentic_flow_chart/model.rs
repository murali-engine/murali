use glam::{Vec2, Vec4};
use std::collections::HashSet;

use super::edge::FlowEdge;
use super::node::FlowNode;
use super::types::{EdgeAnimationStyle, FlowChartDirection, NodeAnimationStyle};
use crate::frontend::TattvaId;

/// Core data model for the flowchart
#[derive(Debug, Clone)]
pub struct FlowChartModel {
    pub nodes: Vec<FlowNode>,
    pub edges: Vec<FlowEdge>,
    pub flow_path: Vec<usize>,
    pub direction: FlowChartDirection,
    /// Label tattva IDs for each node (for WriteText animation)
    pub label_ids: Vec<Option<TattvaId>>,
    /// Active content nodes (for conditional rendering)
    pub active_content_nodes: HashSet<usize>,
}

impl FlowChartModel {
    pub fn new(nodes: Vec<FlowNode>) -> Self {
        let label_ids = vec![None; nodes.len()];
        Self {
            nodes,
            edges: Vec::new(),
            flow_path: Vec::new(),
            direction: FlowChartDirection::Horizontal,
            label_ids,
            active_content_nodes: HashSet::new(),
        }
    }

    pub fn with_edges(mut self, edges: Vec<FlowEdge>) -> Self {
        self.edges = edges;
        self
    }

    pub fn with_flow_path(mut self, flow_path: Vec<usize>) -> Self {
        self.flow_path = flow_path;
        self
    }

    pub fn with_direction(mut self, direction: FlowChartDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn with_label_ids(mut self, label_ids: Vec<Option<TattvaId>>) -> Self {
        self.label_ids = label_ids;
        self
    }

    pub fn set_label_id(&mut self, node_index: usize, label_id: TattvaId) {
        if let Some(slot) = self.label_ids.get_mut(node_index) {
            *slot = Some(label_id);
        }
    }

    pub fn with_active_content_nodes(mut self, nodes: impl IntoIterator<Item = usize>) -> Self {
        self.active_content_nodes = nodes.into_iter().collect();
        self
    }

    pub fn activate_content_node(&mut self, node_index: usize) {
        self.active_content_nodes.insert(node_index);
    }

    pub fn deactivate_content_node(&mut self, node_index: usize) {
        self.active_content_nodes.remove(&node_index);
    }
}

/// Style configuration for the flowchart
#[derive(Debug, Clone)]
pub struct FlowChartStyle {
    pub default_node_size: Vec2,
    pub text_height: f32,
    pub text_padding: Vec2,
    pub node_gap: f32,
    pub lane_gap: f32,
    pub edge_thickness: f32,
    pub edge_color: Vec4,
    pub active_edge_color: Vec4,
    pub pulse_color: Vec4,
    pub pulse_radius: f32,
    pub arrow_size: f32,
    pub indicate_color: Vec4,
    pub indicate_scale: f32,
    pub indicate_window: f32,
}

impl Default for FlowChartStyle {
    fn default() -> Self {
        Self {
            default_node_size: Vec2::new(1.8, 0.8),
            text_height: 0.28,
            text_padding: Vec2::new(0.28, 0.18),
            node_gap: 0.8,
            lane_gap: 1.2,
            edge_thickness: 0.055,
            edge_color: Vec4::new(0.56, 0.63, 0.74, 1.0),
            active_edge_color: Vec4::new(0.98, 0.80, 0.30, 1.0),
            pulse_color: Vec4::new(1.0, 0.96, 0.84, 1.0),
            pulse_radius: 0.12,
            arrow_size: 0.18,
            indicate_color: Vec4::new(0.98, 0.80, 0.30, 1.0),
            indicate_scale: 0.12,
            indicate_window: 0.15,
        }
    }
}

impl FlowChartStyle {
    pub fn with_text_height(mut self, text_height: f32) -> Self {
        self.text_height = text_height;
        self
    }

    pub fn with_default_node_size(mut self, size: Vec2) -> Self {
        self.default_node_size = size;
        self
    }

    pub fn with_node_gap(mut self, gap: f32) -> Self {
        self.node_gap = gap;
        self
    }

    pub fn with_lane_gap(mut self, gap: f32) -> Self {
        self.lane_gap = gap;
        self
    }

    pub fn with_edge_color(mut self, color: Vec4) -> Self {
        self.edge_color = color;
        self
    }

    pub fn with_active_edge_color(mut self, color: Vec4) -> Self {
        self.active_edge_color = color;
        self
    }

    pub fn with_pulse_color(mut self, color: Vec4) -> Self {
        self.pulse_color = color;
        self
    }

    pub fn with_pulse_radius(mut self, radius: f32) -> Self {
        self.pulse_radius = radius;
        self
    }

    pub fn with_indicate_color(mut self, color: Vec4) -> Self {
        self.indicate_color = color;
        self
    }

    pub fn with_indicate_scale(mut self, scale: f32) -> Self {
        self.indicate_scale = scale;
        self
    }
}

/// Animation state and configuration
#[derive(Debug, Clone)]
pub struct FlowChartAnimation {
    pub progress: f32,
    pub reveal_progress: f32,
    pub node_animation_style: NodeAnimationStyle,
    pub edge_animation_style: EdgeAnimationStyle,
    pub node_reveal_window: f32,
    pub node_reveal_delay: f32,
    pub edge_reveal_window: f32,
    pub progressive_edges: bool,
}

impl Default for FlowChartAnimation {
    fn default() -> Self {
        Self {
            progress: 0.0,
            reveal_progress: 0.0,
            node_animation_style: NodeAnimationStyle::Instant,
            edge_animation_style: EdgeAnimationStyle::Instant,
            node_reveal_window: 0.2,
            node_reveal_delay: 0.0,
            edge_reveal_window: 0.15,
            progressive_edges: false,
        }
    }
}

impl FlowChartAnimation {
    pub fn with_reveal_progress(mut self, progress: f32) -> Self {
        self.reveal_progress = progress;
        self
    }

    pub fn with_progress(mut self, progress: f32) -> Self {
        self.progress = progress;
        self
    }

    pub fn with_node_animation_style(mut self, style: NodeAnimationStyle) -> Self {
        self.node_animation_style = style;
        self
    }

    pub fn with_edge_animation_style(mut self, style: EdgeAnimationStyle) -> Self {
        self.edge_animation_style = style;
        self
    }

    pub fn with_node_reveal_window(mut self, window: f32) -> Self {
        self.node_reveal_window = window;
        self
    }

    pub fn with_node_reveal_delay(mut self, delay: f32) -> Self {
        self.node_reveal_delay = delay;
        self
    }

    pub fn with_edge_reveal_window(mut self, window: f32) -> Self {
        self.edge_reveal_window = window;
        self
    }

    pub fn with_progressive_edges(mut self, progressive: bool) -> Self {
        self.progressive_edges = progressive;
        self
    }
}
