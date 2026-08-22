use glam::Vec3;

use super::edge::FlowEdge;
use super::node::FlowNode;
use super::shapes::sample_polyline;
use super::types::{FlowChartDirection, NodeLayout};

/// Computed animation state for a single frame
/// This is the output of AnimationEngine::compute()
#[derive(Debug, Clone)]
pub struct FlowAnimationState {
    /// Reveal threshold for each node (0.0 to 1.0)
    pub node_thresholds: Vec<f32>,
    /// Reveal threshold for each edge (0.0 to 1.0)
    pub edge_thresholds: Vec<f32>,
    /// Current active hop (index, progress within hop)
    pub active_hop: Option<(usize, f32)>,
    /// Current pulse position in world space
    pub pulse_position: Option<Vec3>,
}

/// Animation engine - computes all animation state for a frame
pub struct AnimationEngine;

impl AnimationEngine {
    /// Compute animation state for the current frame
    pub fn compute(
        nodes: &[FlowNode],
        edges: &[FlowEdge],
        flow_path: &[usize],
        progress: f32,
        reveal_progress: f32,
        progressive_edges: bool,
        layouts: &[NodeLayout],
        direction: FlowChartDirection,
        lane_gap: f32,
        node_gap: f32,
        default_node_size: glam::Vec2,
        edge_route_fn: impl Fn(usize, usize) -> Option<Vec<Vec3>>,
    ) -> FlowAnimationState {
        let (node_thresholds, edge_thresholds) =
            Self::compute_reveal_thresholds(nodes, edges, progressive_edges);

        let active_hop = Self::compute_active_hop(flow_path, progress);

        let pulse_position =
            Self::compute_pulse_position(flow_path, layouts, active_hop, edge_route_fn);

        FlowAnimationState {
            node_thresholds,
            edge_thresholds,
            active_hop,
            pulse_position,
        }
    }

    /// Compute reveal thresholds for nodes and edges (public for external use)
    pub fn compute_reveal_thresholds_pub(
        nodes: &[FlowNode],
        edges: &[FlowEdge],
        progressive_edges: bool,
    ) -> (Vec<f32>, Vec<f32>) {
        Self::compute_reveal_thresholds(nodes, edges, progressive_edges)
    }

    /// Compute reveal thresholds for nodes and edges
    fn compute_reveal_thresholds(
        nodes: &[FlowNode],
        edges: &[FlowEdge],
        progressive_edges: bool,
    ) -> (Vec<f32>, Vec<f32>) {
        let n_nodes = nodes.len();
        let n_edges = edges.len();
        let total = n_nodes + n_edges;
        if total == 0 {
            return (Vec::new(), Vec::new());
        }

        // Helper to map raw component indices to virtual "reveal steps"
        // If interleaved, sequence is: Node 0, Edge 0, Node 1, Edge 1...
        let get_v_idx = |raw_idx: usize| -> usize {
            if !progressive_edges {
                return raw_idx;
            }
            if raw_idx < n_nodes {
                raw_idx * 2
            } else {
                (raw_idx - n_nodes) * 2 + 1
            }
        };

        // Collect all explicit assignments using virtual indices
        let mut fixed: Vec<(usize, f32)> = Vec::new();
        for (i, node) in nodes.iter().enumerate() {
            if let Some(at) = node.reveal_at {
                fixed.push((get_v_idx(i), at));
            }
        }
        for (i, edge) in edges.iter().enumerate() {
            if let Some(at) = edge.reveal_at {
                fixed.push((get_v_idx(i + n_nodes), at));
            }
        }

        // Use the interleaved distribution logic if nothing is fixed
        if fixed.is_empty() {
            let mut node_thresholds = vec![0.0; n_nodes];
            let mut edge_thresholds = vec![0.0; n_edges];
            if progressive_edges {
                for i in 0..n_nodes {
                    node_thresholds[i] = (i * 2) as f32 / total as f32;
                }
                for i in 0..n_edges {
                    edge_thresholds[i] = (i * 2 + 1) as f32 / total as f32;
                }
            } else {
                for i in 0..n_nodes {
                    node_thresholds[i] = i as f32 / total as f32;
                }
                for i in 0..n_edges {
                    edge_thresholds[i] = (i + n_nodes) as f32 / total as f32;
                }
            }
            return (node_thresholds, edge_thresholds);
        }

        // Robust Interpolation Branch using virtual indices
        fixed.sort_by_key(|a| a.0);

        // Ensure boundaries are covered in virtual space
        if fixed.first().map(|f| f.0).unwrap_or(0) > 0 {
            fixed.insert(0, (0, 0.0));
        }
        if fixed.last().map(|f| f.0).unwrap_or(0) < total - 1 {
            fixed.push((total - 1, 1.0));
        }
        fixed.dedup_by_key(|a| a.0);

        let mut v_thresholds = vec![0.0; total];
        if fixed.len() == 1 {
            let (_, val) = fixed[0];
            for i in 0..total {
                v_thresholds[i] = val;
            }
        } else {
            for window in fixed.windows(2) {
                let (idx_a, t_a) = window[0];
                let (idx_b, t_b) = window[1];

                v_thresholds[idx_a] = t_a;
                v_thresholds[idx_b] = t_b;

                let count = idx_b - idx_a;
                if count > 1 {
                    let t_range = t_b - t_a;
                    let t_step = t_range / count as f32;
                    for i in 1..count {
                        v_thresholds[idx_a + i] = t_a + t_step * i as f32;
                    }
                }
            }
        }

        // Map back from virtual thresholds to component thresholds
        let mut node_thresholds = vec![0.0; n_nodes];
        let mut edge_thresholds = vec![0.0; n_edges];
        for i in 0..n_nodes {
            node_thresholds[i] = v_thresholds[get_v_idx(i)];
        }
        for i in 0..n_edges {
            edge_thresholds[i] = v_thresholds[get_v_idx(i + n_nodes)];
        }
        (node_thresholds, edge_thresholds)
    }

    /// Compute active hop state (which edge is being traversed)
    fn compute_active_hop(flow_path: &[usize], progress: f32) -> Option<(usize, f32)> {
        let hop_count = flow_path.len().checked_sub(1)?;
        if hop_count == 0 {
            return None;
        }

        let scaled = progress.clamp(0.0, 1.0) * hop_count as f32;
        let hop_idx = scaled.floor() as usize;
        if hop_idx >= hop_count {
            return Some((hop_count - 1, 1.0));
        }
        Some((hop_idx, scaled - hop_idx as f32))
    }

    /// Compute pulse position along the flow path
    fn compute_pulse_position(
        flow_path: &[usize],
        layouts: &[NodeLayout],
        active_hop: Option<(usize, f32)>,
        edge_route_fn: impl Fn(usize, usize) -> Option<Vec<Vec3>>,
    ) -> Option<Vec3> {
        if flow_path.is_empty() {
            return None;
        }
        if flow_path.len() == 1 {
            return layouts.get(flow_path[0]).map(|l| l.center);
        }

        let (hop_idx, hop_t) = active_hop?;
        let route = edge_route_fn(flow_path[hop_idx], flow_path[hop_idx + 1])?;
        sample_polyline(&route, hop_t)
    }

    /// Get write progress for a node (0.0 to 1.0)
    pub fn node_write_progress(
        node_index: usize,
        reveal_progress: f32,
        node_thresholds: &[f32],
        node_reveal_delay: f32,
        node_reveal_window: f32,
    ) -> f32 {
        let threshold = node_thresholds.get(node_index).copied().unwrap_or(0.0);
        let delayed_threshold = threshold + node_reveal_delay;

        if reveal_progress < delayed_threshold {
            return 0.0;
        }

        // Calculate progress within the reveal window
        (reveal_progress - delayed_threshold) / node_reveal_window
    }

    /// Get indication intensity for a node (0.0 to 1.0)
    pub fn node_indicate_intensity(
        node_index: usize,
        flow_path: &[usize],
        progress: f32,
        reveal_progress: f32,
        node_thresholds: &[f32],
        indicate_window: f32,
    ) -> f32 {
        if flow_path.is_empty() {
            return 0.0;
        }

        let node_threshold = node_thresholds.get(node_index).copied().unwrap_or(0.0);

        // Only show highlight if the node has been revealed
        if reveal_progress < node_threshold {
            return 0.0;
        }

        let hop_count = flow_path.len().saturating_sub(1) as f32;
        let mut best: f32 = 0.0;
        for (arrival_idx, path_node) in flow_path.iter().copied().enumerate() {
            if path_node != node_index {
                continue;
            }
            let arrival_t = if hop_count <= 0.0 {
                0.0
            } else {
                arrival_idx as f32 / hop_count
            };
            let delta = ((progress.clamp(0.0, 1.0) - arrival_t).abs() / indicate_window.max(1e-4))
                .clamp(0.0, 1.0);
            let intensity = (1.0 - delta).sin().max(0.0);
            best = best.max(intensity);
        }
        best
    }

    /// Check if an edge should be drawn in progressive mode
    pub fn should_draw_edge_progressive(
        edge: &FlowEdge,
        reveal_progress: f32,
        node_thresholds: &[f32],
        progressive_edges: bool,
    ) -> bool {
        if !progressive_edges {
            return true; // Draw all edges if not in progressive mode
        }

        let from_threshold = node_thresholds.get(edge.from).copied().unwrap_or(0.0);
        let to_threshold = node_thresholds.get(edge.to).copied().unwrap_or(0.0);

        // Draw edge if both nodes are revealed
        reveal_progress >= from_threshold && reveal_progress >= to_threshold
    }
}
