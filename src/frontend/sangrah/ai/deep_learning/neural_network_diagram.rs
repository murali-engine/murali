use crate::frontend::animation::indicate::Indicate;
use crate::frontend::layout::{Bounded, Bounds};
use crate::frontend::sangrah::ai::ml_components::signal_flow::SignalFlow;
use crate::projection::{Mesh, Project, ProjectionCtx, RenderPrimitive};
use glam::{Vec2, Vec3, Vec4, vec2};
use std::collections::HashSet;
use thiserror::Error;

const MAX_INDICATION_PATHS: usize = 64;

#[derive(Debug, Clone)]
pub struct NeuralNetworkDiagram {
    pub layers: Vec<usize>,
    pub layer_spacing: f32,
    pub node_spacing: f32,
    pub node_radius: f32,
    pub node_color: Vec4,
    pub edge_color: Vec4,
    pub edge_thickness: f32,
    pub layer_labels: Option<Vec<String>>,
    pub activation: ActivationFunc,
    pub inactive_nodes: HashSet<(usize, usize)>,
    pub inactive_node_color: Vec4,
    pub inactive_edge_color: Vec4,
    pub indication_style: IndicationStyle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IndicationStyle {
    /// Fires exactly once over the duration of the indication (t: 0.0 -> 1.0)
    Single,
    /// Loops the signal N times over the duration of the indication
    Loop(usize),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActivationFunc {
    None,
    ReLU,
    Sigmoid,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum NeuralNetworkDiagramError {
    #[error("a neural-network diagram requires at least one layer")]
    NoLayers,
    #[error("neural-network layer {layer_index} must contain at least one node")]
    EmptyLayer { layer_index: usize },
}

impl NeuralNetworkDiagram {
    pub fn new(layers: Vec<usize>) -> Self {
        Self::try_new(layers).unwrap_or_else(|error| panic!("invalid neural network: {error}"))
    }

    pub fn try_new(layers: Vec<usize>) -> Result<Self, NeuralNetworkDiagramError> {
        if layers.is_empty() {
            return Err(NeuralNetworkDiagramError::NoLayers);
        }
        if let Some(layer_index) = layers.iter().position(|&count| count == 0) {
            return Err(NeuralNetworkDiagramError::EmptyLayer { layer_index });
        }

        Ok(Self {
            layers,
            layer_spacing: 1.8,
            node_spacing: 0.8,
            node_radius: 0.12,
            node_color: Vec4::new(0.36, 0.77, 0.98, 1.0),
            edge_color: Vec4::new(0.48, 0.56, 0.68, 1.0),
            edge_thickness: 0.015,
            layer_labels: None,
            activation: ActivationFunc::None,
            inactive_nodes: HashSet::new(),
            inactive_node_color: Vec4::new(0.26, 0.30, 0.36, 0.95),
            inactive_edge_color: Vec4::new(0.28, 0.32, 0.38, 0.45),
            indication_style: IndicationStyle::Loop(3),
        })
    }

    pub fn with_layer_spacing(mut self, spacing: f32) -> Self {
        self.layer_spacing = spacing;
        self
    }

    pub fn with_node_spacing(mut self, spacing: f32) -> Self {
        self.node_spacing = spacing;
        self
    }

    pub fn with_node_radius(mut self, radius: f32) -> Self {
        self.node_radius = radius;
        self
    }

    pub fn with_node_color(mut self, color: Vec4) -> Self {
        self.node_color = color;
        self
    }

    pub fn with_edge_color(mut self, color: Vec4) -> Self {
        self.edge_color = color;
        self
    }

    pub fn with_edge_thickness(mut self, thickness: f32) -> Self {
        self.edge_thickness = thickness;
        self
    }

    pub fn with_inactive_node_color(mut self, color: Vec4) -> Self {
        self.inactive_node_color = color;
        self
    }

    pub fn with_inactive_edge_color(mut self, color: Vec4) -> Self {
        self.inactive_edge_color = color;
        self
    }

    pub fn with_indication_style(mut self, style: IndicationStyle) -> Self {
        self.indication_style = style;
        self
    }

    pub fn with_labels(mut self, labels: Vec<impl Into<String>>) -> Self {
        self.layer_labels = Some(labels.into_iter().map(|s| s.into()).collect());
        self
    }

    pub fn with_activation(mut self, func: ActivationFunc) -> Self {
        self.activation = func;
        self
    }

    pub fn with_deactivated_nodes(
        mut self,
        nodes: impl IntoIterator<Item = (usize, usize)>,
    ) -> Self {
        self.inactive_nodes = nodes.into_iter().collect();
        self
    }

    pub fn deactivate_node(mut self, layer_idx: usize, node_idx: usize) -> Self {
        self.inactive_nodes.insert((layer_idx, node_idx));
        self
    }

    pub fn is_node_active(&self, layer_idx: usize, node_idx: usize) -> bool {
        self.layers
            .get(layer_idx)
            .map(|count| node_idx < *count && !self.inactive_nodes.contains(&(layer_idx, node_idx)))
            .unwrap_or(false)
    }

    pub fn is_edge_active(
        &self,
        from_layer_idx: usize,
        from_node_idx: usize,
        to_layer_idx: usize,
        to_node_idx: usize,
    ) -> bool {
        self.is_node_active(from_layer_idx, from_node_idx)
            && to_layer_idx == from_layer_idx + 1
            && self.is_node_active(to_layer_idx, to_node_idx)
    }

    fn layer_x(&self, idx: usize) -> f32 {
        let width = (self.layers.len().saturating_sub(1) as f32) * self.layer_spacing;
        -width * 0.5 + idx as f32 * self.layer_spacing
    }

    fn node_y(&self, count: usize, idx: usize) -> f32 {
        let height = (count.saturating_sub(1) as f32) * self.node_spacing;
        height * 0.5 - idx as f32 * self.node_spacing
    }

    pub fn node_position(&self, layer_idx: usize, node_idx: usize) -> Option<Vec3> {
        let count = *self.layers.get(layer_idx)?;
        if node_idx >= count {
            return None;
        }
        Some(Vec3::new(
            self.layer_x(layer_idx),
            self.node_y(count, node_idx),
            0.0,
        ))
    }

    pub fn path_points(&self, node_indices_per_layer: &[usize]) -> Option<Vec<Vec3>> {
        if node_indices_per_layer.len() != self.layers.len() || node_indices_per_layer.is_empty() {
            return None;
        }

        let mut points = Vec::with_capacity(node_indices_per_layer.len());
        for (layer_idx, node_idx) in node_indices_per_layer.iter().copied().enumerate() {
            if !self.is_node_active(layer_idx, node_idx) {
                return None;
            }
            points.push(self.node_position(layer_idx, node_idx)?);
        }
        Some(points)
    }

    pub fn all_path_points(&self) -> Vec<Vec<Vec3>> {
        if self.layers.is_empty() {
            return Vec::new();
        }

        let mut all_paths = Vec::new();
        let mut node_choices = Vec::with_capacity(self.layers.len());
        self.collect_paths(0, &mut node_choices, &mut all_paths);
        all_paths
    }

    /// Returns a deterministic, bounded set of active routes for indication effects.
    pub fn indication_path_points(&self, max_paths: usize) -> Vec<Vec<Vec3>> {
        if max_paths == 0 || self.layers.is_empty() {
            return Vec::new();
        }

        let active_nodes: Vec<Vec<usize>> = self
            .layers
            .iter()
            .enumerate()
            .map(|(layer_idx, &count)| {
                (0..count)
                    .filter(|&node_idx| self.is_node_active(layer_idx, node_idx))
                    .collect()
            })
            .collect();
        if active_nodes.iter().any(Vec::is_empty) {
            return Vec::new();
        }

        let path_count = active_nodes
            .iter()
            .map(Vec::len)
            .max()
            .unwrap_or(0)
            .min(max_paths);
        (0..path_count)
            .map(|path_index| {
                active_nodes
                    .iter()
                    .enumerate()
                    .map(|(layer_idx, choices)| {
                        let choice_index = path_index * choices.len() / path_count;
                        self.node_position(layer_idx, choices[choice_index])
                            .unwrap()
                    })
                    .collect()
            })
            .collect()
    }

    fn collect_paths(
        &self,
        layer_idx: usize,
        node_choices: &mut Vec<usize>,
        all_paths: &mut Vec<Vec<Vec3>>,
    ) {
        let Some(&count) = self.layers.get(layer_idx) else {
            return;
        };

        for node_idx in 0..count {
            if !self.is_node_active(layer_idx, node_idx) {
                continue;
            }

            node_choices.push(node_idx);
            if layer_idx + 1 == self.layers.len() {
                if let Some(path) = self.partial_path_points(node_choices) {
                    all_paths.push(path);
                }
            } else {
                self.collect_paths(layer_idx + 1, node_choices, all_paths);
            }
            node_choices.pop();
        }
    }

    fn partial_path_points(&self, node_indices_per_layer: &[usize]) -> Option<Vec<Vec3>> {
        if node_indices_per_layer.is_empty() || node_indices_per_layer.len() > self.layers.len() {
            return None;
        }

        let mut points = Vec::with_capacity(node_indices_per_layer.len());
        for (layer_idx, node_idx) in node_indices_per_layer.iter().copied().enumerate() {
            if !self.is_node_active(layer_idx, node_idx) {
                return None;
            }
            points.push(self.node_position(layer_idx, node_idx)?);
        }
        Some(points)
    }

    fn project_activation(&self, ctx: &mut ProjectionCtx, x: f32, y: f32) {
        let size = 0.15;
        let color = Vec4::new(0.96, 0.74, 0.26, 1.0);
        let thickness = 0.02;

        match self.activation {
            ActivationFunc::ReLU => {
                // Draw L shape
                ctx.emit(RenderPrimitive::Line {
                    start: Vec3::new(x - size, y, 0.0),
                    end: Vec3::new(x, y, 0.0),
                    thickness,
                    color,
                    dash_length: 0.0,
                    gap_length: 0.0,
                    dash_offset: 0.0,
                });
                ctx.emit(RenderPrimitive::Line {
                    start: Vec3::new(x, y, 0.0),
                    end: Vec3::new(x + size, y + size, 0.0),
                    thickness,
                    color,
                    dash_length: 0.0,
                    gap_length: 0.0,
                    dash_offset: 0.0,
                });
            }
            ActivationFunc::Sigmoid => {
                // S curve approximation
                let steps = 8;
                for i in 0..steps {
                    let t0 = i as f32 / steps as f32;
                    let t1 = (i + 1) as f32 / steps as f32;
                    let x0 = x - size + t0 * 2.0 * size;
                    let x1 = x - size + t1 * 2.0 * size;
                    let y0 = y + (size * 2.0 / (1.0 + (-((t0 - 0.5) * 6.0)).exp())) - size;
                    let y1 = y + (size * 2.0 / (1.0 + (-((t1 - 0.5) * 6.0)).exp())) - size;

                    ctx.emit(RenderPrimitive::Line {
                        start: Vec3::new(x0, y0, 0.0),
                        end: Vec3::new(x1, y1, 0.0),
                        thickness,
                        color,
                        dash_length: 0.0,
                        gap_length: 0.0,
                        dash_offset: 0.0,
                    });
                }
            }
            ActivationFunc::None => {}
        }
    }
}

impl Project for NeuralNetworkDiagram {
    fn project(&self, ctx: &mut ProjectionCtx) {
        for layer_idx in 0..self.layers.len().saturating_sub(1) {
            let from_count = self.layers[layer_idx];
            let to_count = self.layers[layer_idx + 1];
            let x0 = self.layer_x(layer_idx);
            let x1 = self.layer_x(layer_idx + 1);
            for i in 0..from_count {
                for j in 0..to_count {
                    let active = self.is_edge_active(layer_idx, i, layer_idx + 1, j);
                    ctx.emit(RenderPrimitive::Line {
                        start: Vec3::new(x0, self.node_y(from_count, i), 0.0),
                        end: Vec3::new(x1, self.node_y(to_count, j), 0.0),
                        thickness: self.edge_thickness,
                        color: if active {
                            self.edge_color
                        } else {
                            self.inactive_edge_color
                        },
                        dash_length: 0.0,
                        gap_length: 0.0,
                        dash_offset: 0.0,
                    });
                }
            }
        }

        for (layer_idx, count) in self.layers.iter().copied().enumerate() {
            let x = self.layer_x(layer_idx);
            for node_idx in 0..count {
                let node_color = if self.is_node_active(layer_idx, node_idx) {
                    self.node_color
                } else {
                    self.inactive_node_color
                };
                let mesh = Mesh::circle(self.node_radius, 24, node_color)
                    .as_ref()
                    .translated(Vec3::new(x, self.node_y(count, node_idx), 0.0));
                ctx.emit(RenderPrimitive::Mesh(mesh));
            }

            // Draw layer label if present
            if let Some(labels) = &self.layer_labels {
                if let Some(label) = labels.get(layer_idx) {
                    let y_offset = self.node_y(count, 0) + self.node_spacing * 0.7;
                    ctx.emit(RenderPrimitive::Text {
                        content: label.clone(),
                        height: 0.22,
                        color: Vec4::new(0.9, 0.9, 0.9, 1.0),
                        font_name: None,
                        offset: Vec3::new(x, y_offset, 0.0),
                        rotation: 0.0,
                    });
                }
            }

            // Draw activation icon if not None (only for hidden/output layers)
            if layer_idx > 0 && count > 0 && self.activation != ActivationFunc::None {
                let y_offset = self.node_y(count, count - 1) - self.node_spacing * 0.7;
                self.project_activation(ctx, x, y_offset);
            }
        }
    }
}

impl Bounded for NeuralNetworkDiagram {
    fn local_bounds(&self) -> Bounds {
        let width = (self.layers.len().saturating_sub(1) as f32) * self.layer_spacing
            + self.node_radius * 2.0;
        let max_nodes = self.layers.iter().copied().max().unwrap_or(1);
        let height =
            (max_nodes.saturating_sub(1) as f32) * self.node_spacing + self.node_radius * 2.0;
        Bounds::from_center_size(
            Vec2::ZERO,
            vec2(
                width.max(self.node_radius * 2.0),
                height.max(self.node_radius * 2.0),
            ),
        )
    }
}

impl Indicate for NeuralNetworkDiagram {
    fn project_indicated(&self, ctx: &mut ProjectionCtx, t: f32) {
        // Base pulse for the whole diagram
        let scale = 1.0 + 0.15 * t;
        ctx.with_scale(scale, |ctx| {
            self.project(ctx);

            // Internal firing logic
            let loop_progress = match self.indication_style {
                IndicationStyle::Single => t,
                IndicationStyle::Loop(n) => (t * n as f32).fract(),
            };

            let paths = self.indication_path_points(MAX_INDICATION_PATHS);
            if !paths.is_empty() {
                SignalFlow::from_paths(paths)
                    .with_progress(loop_progress)
                    .with_edge_color(Vec4::new(0.98, 0.76, 0.30, 0.95))
                    .with_pulse_color(Vec4::new(1.0, 0.96, 0.82, 1.0))
                    .project(ctx);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::{NeuralNetworkDiagram, NeuralNetworkDiagramError};

    #[test]
    fn constructors_reject_missing_and_zero_sized_layers() {
        assert_eq!(
            NeuralNetworkDiagram::try_new(vec![]).unwrap_err(),
            NeuralNetworkDiagramError::NoLayers
        );
        assert_eq!(
            NeuralNetworkDiagram::try_new(vec![3, 0, 2]).unwrap_err(),
            NeuralNetworkDiagramError::EmptyLayer { layer_index: 1 }
        );
    }

    #[test]
    fn edge_activity_requires_both_endpoint_nodes() {
        let diagram = NeuralNetworkDiagram::new(vec![2, 2]).deactivate_node(1, 1);

        assert!(diagram.is_edge_active(0, 0, 1, 0));
        assert!(!diagram.is_edge_active(0, 0, 1, 1));
        assert!(!diagram.is_edge_active(0, 0, 2, 0));
    }

    #[test]
    fn indication_paths_are_active_and_bounded() {
        let diagram = NeuralNetworkDiagram::new(vec![100, 100, 100]).deactivate_node(1, 50);
        let paths = diagram.indication_path_points(64);

        assert_eq!(paths.len(), 64);
        assert!(paths.iter().all(|path| path.len() == 3));
        let inactive_position = diagram.node_position(1, 50).unwrap();
        assert!(
            paths
                .iter()
                .all(|path| !path.iter().any(|point| *point == inactive_position))
        );
    }

    #[test]
    fn all_path_points_only_returns_complete_active_routes() {
        let diagram = NeuralNetworkDiagram::new(vec![2, 2, 1]).deactivate_node(1, 1);
        let paths = diagram.all_path_points();

        assert_eq!(paths.len(), 2);
        assert!(paths.iter().all(|path| path.len() == 3));
        let inactive_position = diagram.node_position(1, 1).unwrap();
        assert!(
            paths
                .iter()
                .all(|path| !path.iter().any(|point| *point == inactive_position))
        );
    }
}
