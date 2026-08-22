use glam::{Vec2, Vec3, Vec4, vec2, vec3};
use std::collections::HashSet;
use std::sync::Arc;

use crate::backend::renderer::vertex::{mesh::MeshVertex, text::TextVertex};
use crate::frontend::TattvaId;
use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::mesh::MeshData;
use crate::projection::{Mesh, Project, ProjectionCtx, RenderPrimitive};

// Import from our submodules
use super::animation::{AnimationEngine, FlowAnimationState};
use super::edge::FlowEdge;
use super::layout::{calculate_node_layouts, layout_extents};
use super::node::{FlowNode, FlowNodeContent, ProjectedFlowNodeContent};
use super::renderer::{FlowRenderer, RenderContext};
use super::routing::EdgeRouter;
use super::shapes::{
    dedup_points, next_level_val, partial_polyline, polyline_length, sample_polyline, shape_mesh,
    shape_outline,
};
use super::types::*;

#[derive(Debug, Clone)]
pub struct AgenticFlowChart {
    pub nodes: Vec<FlowNode>,
    pub edges: Vec<FlowEdge>,
    pub flow_path: Vec<usize>,
    pub progress: f32,
    pub direction: FlowChartDirection,
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
    pub reveal_progress: f32,
    pub active_content_nodes: HashSet<usize>,
    /// Animation style for node appearance
    pub node_animation_style: NodeAnimationStyle,
    /// Animation style for edge appearance
    pub edge_animation_style: EdgeAnimationStyle,
    /// Progress window for node reveal (0.0 to 1.0)
    pub node_reveal_window: f32,
    /// Delay before node box reveal starts (0.0 to 1.0, relative to window or threshold)
    pub node_reveal_delay: f32,
    /// Progress window for edge reveal (0.0 to 1.0)
    pub edge_reveal_window: f32,
    /// Whether to draw edges progressively as nodes appear
    pub progressive_edges: bool,
    /// Label tattva IDs for each node (for WriteText animation)
    pub label_ids: Vec<Option<TattvaId>>,
}

impl AgenticFlowChart {
    pub fn new(nodes: Vec<FlowNode>) -> Self {
        let label_ids = vec![None; nodes.len()];
        Self {
            nodes,
            edges: Vec::new(),
            flow_path: Vec::new(),
            progress: 0.0,
            direction: FlowChartDirection::Horizontal,
            default_node_size: vec2(2.2, 1.0),
            text_height: 0.22,
            text_padding: vec2(0.35, 0.22),
            node_gap: 1.0,
            lane_gap: 0.9,
            edge_thickness: 0.04,
            edge_color: Vec4::new(0.42, 0.49, 0.59, 1.0),
            active_edge_color: Vec4::new(0.98, 0.78, 0.28, 1.0),
            pulse_color: Vec4::new(1.0, 0.96, 0.82, 1.0),
            pulse_radius: 0.12,
            arrow_size: 0.16,
            indicate_color: Vec4::new(1.0, 0.90, 0.42, 1.0),
            indicate_scale: 0.12,
            indicate_window: 0.14,
            reveal_progress: 1.0,
            active_content_nodes: HashSet::new(),
            node_animation_style: NodeAnimationStyle::Instant,
            edge_animation_style: EdgeAnimationStyle::default(),
            node_reveal_window: 0.15,
            node_reveal_delay: 0.0,
            edge_reveal_window: 0.1,
            progressive_edges: false,
            label_ids,
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

    pub fn with_reveal_progress(mut self, progress: f32) -> Self {
        self.reveal_progress = progress.clamp(0.0, 1.0);
        self
    }

    pub fn with_progress(mut self, progress: f32) -> Self {
        self.progress = progress.clamp(0.0, 1.0);
        self
    }

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

    /// Set a specific reveal progress for a node
    pub fn with_node_reveal_at(mut self, node_index: usize, progress: f32) -> Self {
        if node_index < self.nodes.len() {
            self.nodes[node_index].reveal_at = Some(progress.clamp(0.0, 1.0));
        }
        self
    }

    /// Set a specific reveal progress for an edge
    pub fn with_edge_reveal_at(mut self, edge_index: usize, progress: f32) -> Self {
        if edge_index < self.edges.len() {
            self.edges[edge_index].reveal_at = Some(progress.clamp(0.0, 1.0));
        }
        self
    }

    /// Attach label tattva IDs for WriteText animation support
    pub fn with_label_ids(mut self, label_ids: Vec<Option<TattvaId>>) -> Self {
        self.label_ids = label_ids;
        self
    }

    /// Set a specific label ID for a node
    pub fn set_label_id(&mut self, node_index: usize, label_id: TattvaId) {
        if node_index < self.label_ids.len() {
            self.label_ids[node_index] = Some(label_id);
        }
    }

    fn connection_pairs(&self) -> Vec<FlowEdge> {
        if !self.edges.is_empty() {
            return self.edges.clone();
        }

        let mut pairs = Vec::new();
        for window in self.flow_path.windows(2) {
            let edge = FlowEdge::new(window[0], window[1]);
            if !pairs.contains(&edge) {
                pairs.push(edge);
            }
        }
        pairs
    }

    fn resolved_node_size(&self, node: &FlowNode) -> Vec2 {
        if let Some(size) = node.size {
            return size;
        }

        let text_layout =
            crate::resource::text::layout::measure_label(&node.label, self.text_height, None);
        let mut size = vec2(
            self.default_node_size
                .x
                .max(text_layout.width + self.text_padding.x * 2.0),
            self.default_node_size
                .y
                .max(text_layout.height + self.text_padding.y * 2.0),
        );

        if let Some(content) = &node.embedded_content {
            let content_size = content.local_bounds().size();
            size.x = size.x.max(content_size.x + node.content_padding.x * 2.0);
            size.y = size.y.max(content_size.y + node.content_padding.y * 2.0);
        }

        size
    }

    fn node_layouts(&self) -> Vec<NodeLayout> {
        if self.nodes.is_empty() {
            return Vec::new();
        }

        let sizes: Vec<Vec2> = self
            .nodes
            .iter()
            .map(|node| self.resolved_node_size(node))
            .collect();

        let has_custom_placements = self
            .nodes
            .iter()
            .skip(1)
            .any(|node| node.placement.is_some());

        if has_custom_placements {
            let mut layouts = Vec::with_capacity(self.nodes.len());
            layouts.push(NodeLayout {
                center: Vec3::ZERO,
                size: sizes[0],
            });

            for idx in 1..self.nodes.len() {
                let previous = layouts[idx - 1];
                let size = sizes[idx];
                let placement = self.nodes[idx].placement.unwrap_or(match self.direction {
                    FlowChartDirection::Horizontal => FlowNodePlacement::RightOfPrevious,
                    FlowChartDirection::Vertical => FlowNodePlacement::BelowPrevious,
                });
                let center = match placement {
                    FlowNodePlacement::RightOfPrevious => Vec3::new(
                        previous.center.x + previous.size.x * 0.5 + self.node_gap + size.x * 0.5,
                        previous.center.y,
                        0.0,
                    ),
                    FlowNodePlacement::LeftOfPrevious => Vec3::new(
                        previous.center.x - previous.size.x * 0.5 - self.node_gap - size.x * 0.5,
                        previous.center.y,
                        0.0,
                    ),
                    FlowNodePlacement::AbovePrevious => Vec3::new(
                        previous.center.x,
                        previous.center.y + previous.size.y * 0.5 + self.node_gap + size.y * 0.5,
                        0.0,
                    ),
                    FlowNodePlacement::BelowPrevious => Vec3::new(
                        previous.center.x,
                        previous.center.y - previous.size.y * 0.5 - self.node_gap - size.y * 0.5,
                        0.0,
                    ),
                };
                layouts.push(NodeLayout { center, size });
            }

            let (min, max) = layout_extents(&layouts);
            let center = (min + max) * 0.5;
            for layout in &mut layouts {
                layout.center.x -= center.x;
                layout.center.y -= center.y;
            }
            return layouts;
        }

        let total_primary: f32 = sizes
            .iter()
            .map(|size| match self.direction {
                FlowChartDirection::Horizontal => size.x,
                FlowChartDirection::Vertical => size.y,
            })
            .sum::<f32>()
            + self.node_gap * self.nodes.len().saturating_sub(1) as f32;

        let mut cursor = -total_primary * 0.5;
        let mut layouts = Vec::with_capacity(self.nodes.len());
        for size in sizes {
            let center = match self.direction {
                FlowChartDirection::Horizontal => {
                    let x = cursor + size.x * 0.5;
                    cursor += size.x + self.node_gap;
                    Vec3::new(x, 0.0, 0.0)
                }
                FlowChartDirection::Vertical => {
                    let y = total_primary * 0.5 - (cursor.abs() + size.y * 0.5);
                    cursor += size.y + self.node_gap;
                    Vec3::new(0.0, y, 0.0)
                }
            };
            layouts.push(NodeLayout { center, size });
        }

        if self.direction == FlowChartDirection::Vertical {
            let mut cursor = total_primary * 0.5;
            for layout in &mut layouts {
                layout.center.y = cursor - layout.size.y * 0.5;
                cursor -= layout.size.y + self.node_gap;
            }
        }

        layouts
    }

    pub fn node_center(&self, node_index: usize) -> Option<Vec3> {
        self.node_layouts()
            .get(node_index)
            .map(|layout| layout.center)
    }

    /// Returns the reveal threshold for each node, matching the animation engine's calculation.
    /// Use this to synchronize external label animations with node reveal timing.
    pub fn node_reveal_thresholds(&self) -> Vec<f32> {
        use super::animation::AnimationEngine;
        let (thresholds, _) = AnimationEngine::compute_reveal_thresholds_pub(
            &self.nodes,
            &self.edges,
            self.progressive_edges,
        );
        thresholds
    }

    pub fn node_arrivals(&self, start_time: f32, duration: f32) -> Vec<FlowNodeArrival> {
        if self.flow_path.is_empty() {
            return Vec::new();
        }

        let hop_count = self.flow_path.len().saturating_sub(1);
        self.flow_path
            .iter()
            .copied()
            .enumerate()
            .map(|(visit_index, node_index)| {
                let progress = if hop_count == 0 {
                    0.0
                } else {
                    visit_index as f32 / hop_count as f32
                };
                FlowNodeArrival {
                    node_index,
                    visit_index,
                    time: start_time + duration.max(0.0) * progress,
                }
            })
            .collect()
    }

    pub fn completion_time(&self, start_time: f32, duration: f32) -> Option<f32> {
        if self.flow_path.is_empty() {
            None
        } else {
            Some(start_time + duration.max(0.0))
        }
    }

    fn edge_definition(&self, from: usize, to: usize) -> Option<&FlowEdge> {
        self.edges
            .iter()
            .find(|edge| edge.from == from && edge.to == to)
    }

    fn edge_route(&self, layouts: &[NodeLayout], from: usize, to: usize) -> Option<Vec<Vec3>> {
        EdgeRouter::route(
            layouts,
            from,
            to,
            self.edge_definition(from, to),
            self.direction,
            self.lane_gap,
            self.node_gap,
            self.default_node_size,
        )
    }

    fn node_content_visible(&self, node_index: usize, node: &FlowNode) -> bool {
        match node.content_visibility {
            FlowNodeContentVisibility::Always => true,
            FlowNodeContentVisibility::ActiveOnly => {
                self.active_content_nodes.contains(&node_index)
            }
        }
    }
}

impl Project for AgenticFlowChart {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let layouts = self.node_layouts();
        if layouts.is_empty() {
            return;
        }

        // Compute animation state once
        let anim_state = AnimationEngine::compute(
            &self.nodes,
            &self.edges,
            &self.flow_path,
            self.progress,
            self.reveal_progress,
            self.progressive_edges,
            &layouts,
            self.direction,
            self.lane_gap,
            self.node_gap,
            self.default_node_size,
            |from, to| self.edge_route(&layouts, from, to),
        );

        // Build render context
        let render_ctx = RenderContext {
            nodes: &self.nodes,
            edges: &self.edges,
            flow_path: &self.flow_path,
            label_ids: &self.label_ids,
            active_content_nodes: &self.active_content_nodes,
            edge_color: self.edge_color,
            active_edge_color: self.active_edge_color,
            edge_thickness: self.edge_thickness,
            arrow_size: self.arrow_size,
            pulse_radius: self.pulse_radius,
            pulse_color: self.pulse_color,
            indicate_color: self.indicate_color,
            indicate_scale: self.indicate_scale,
            text_height: self.text_height,
            node_animation_style: self.node_animation_style,
            edge_animation_style: self.edge_animation_style,
            reveal_progress: self.reveal_progress,
            progress: self.progress,
            node_reveal_delay: self.node_reveal_delay,
            node_reveal_window: self.node_reveal_window,
            edge_reveal_window: self.edge_reveal_window,
            progressive_edges: self.progressive_edges,
        };

        // Render pipeline
        FlowRenderer::render(ctx, &render_ctx, &layouts, &anim_state, |from, to| {
            self.edge_route(&layouts, from, to)
        });
    }
}

impl Bounded for AgenticFlowChart {
    fn local_bounds(&self) -> Bounds {
        let layouts = self.node_layouts();
        if layouts.is_empty() {
            return Bounds::from_center_size(Vec2::ZERO, vec2(0.01, 0.01));
        }

        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = Vec2::splat(f32::NEG_INFINITY);
        let pad = self.pulse_radius
            + self.edge_thickness
            + self.arrow_size
            + self.indicate_scale.max(0.0);

        for layout in &layouts {
            min = min.min(layout.center.truncate() - layout.size * 0.5 - vec2(pad, pad));
            max = max.max(layout.center.truncate() + layout.size * 0.5 + vec2(pad, pad));
        }

        for edge in self.connection_pairs() {
            if let Some(route) = self.edge_route(&layouts, edge.from, edge.to) {
                for point in route {
                    min = min.min(point.truncate() - vec2(pad, pad));
                    max = max.max(point.truncate() + vec2(pad, pad));
                }
            }
        }

        Bounds::new(min, max)
    }
}

fn project_content_in_node(
    ctx: &mut ProjectionCtx,
    content: &dyn FlowNodeContent,
    node_center: Vec3,
    target_size: Vec2,
) {
    let source_bounds = content.local_bounds();
    let source_size = vec2(
        source_bounds.width().max(0.01),
        source_bounds.height().max(0.01),
    );
    let scale = (target_size.x / source_size.x)
        .min(target_size.y / source_size.y)
        .max(0.001);
    let source_center = source_bounds.center();
    let target_center = node_center.truncate();

    let mut subctx = ProjectionCtx::new(ctx.props.clone());
    content.project(&mut subctx);
    ctx.diagnostics.append(&mut subctx.diagnostics);
    for primitive in subctx.primitives {
        emit_transformed_primitive(ctx, primitive, source_center, target_center, scale);
    }
}

fn emit_transformed_primitive(
    ctx: &mut ProjectionCtx,
    primitive: RenderPrimitive,
    source_center: Vec2,
    target_center: Vec2,
    scale: f32,
) {
    match primitive {
        RenderPrimitive::Mesh(mesh) => {
            let transformed = match &mesh.data {
                MeshData::Empty => std::sync::Arc::new(mesh.as_ref().clone()),
                MeshData::Mesh(vertices) => std::sync::Arc::new(Mesh {
                    data: MeshData::Mesh(
                        vertices
                            .iter()
                            .map(|vertex| MeshVertex {
                                position: transform_position(
                                    vertex.position,
                                    source_center,
                                    target_center,
                                    scale,
                                ),
                                color: vertex.color,
                            })
                            .collect(),
                    ),
                    indices: mesh.indices.clone(),
                    texture: mesh.texture.clone(),
                }),
                MeshData::Textured(vertices) => std::sync::Arc::new(Mesh {
                    data: MeshData::Textured(
                        vertices
                            .iter()
                            .map(|vertex| TextVertex {
                                position: transform_position(
                                    vertex.position,
                                    source_center,
                                    target_center,
                                    scale,
                                ),
                                uv: vertex.uv,
                                color: vertex.color,
                            })
                            .collect(),
                    ),
                    indices: mesh.indices.clone(),
                    texture: mesh.texture.clone(),
                }),
                MeshData::Text(vertices) => std::sync::Arc::new(Mesh {
                    data: MeshData::Text(
                        vertices
                            .iter()
                            .map(|vertex| TextVertex {
                                position: transform_position(
                                    vertex.position,
                                    source_center,
                                    target_center,
                                    scale,
                                ),
                                uv: vertex.uv,
                                color: vertex.color,
                            })
                            .collect(),
                    ),
                    indices: mesh.indices.clone(),
                    texture: mesh.texture.clone(),
                }),
            };
            ctx.emit(RenderPrimitive::Mesh(transformed));
        }
        RenderPrimitive::Line {
            start,
            end,
            thickness,
            color,
            dash_length,
            gap_length,
            dash_offset,
        } => {
            ctx.emit(RenderPrimitive::Line {
                start: transform_vec3(start, source_center, target_center, scale),
                end: transform_vec3(end, source_center, target_center, scale),
                thickness: thickness * scale,
                color,
                dash_length: dash_length * scale,
                gap_length: gap_length * scale,
                dash_offset: dash_offset * scale,
            });
        }
        RenderPrimitive::Text {
            content,
            height,
            color,
            font_name,
            offset,
            rotation,
        } => {
            ctx.emit(RenderPrimitive::Text {
                content,
                height: height * scale,
                color,
                font_name,
                offset: transform_vec3(offset, source_center, target_center, scale),
                rotation,
            });
        }
        RenderPrimitive::Latex {
            source,
            height,
            color,
            offset,
        } => {
            ctx.emit(RenderPrimitive::Latex {
                source,
                height: height * scale,
                color,
                offset: transform_vec3(offset, source_center, target_center, scale),
            });
        }
        RenderPrimitive::Typst {
            source,
            height,
            color,
            offset,
            normalize,
            tint,
        } => {
            ctx.emit(RenderPrimitive::Typst {
                source,
                height: height * scale,
                color,
                offset: transform_vec3(offset, source_center, target_center, scale),
                normalize,
                tint,
            });
        }
    }
}

fn transform_vec3(point: Vec3, source_center: Vec2, target_center: Vec2, scale: f32) -> Vec3 {
    vec3(
        (point.x - source_center.x) * scale + target_center.x,
        (point.y - source_center.y) * scale + target_center.y,
        point.z * scale,
    )
}

fn transform_position(
    position: [f32; 3],
    source_center: Vec2,
    target_center: Vec2,
    scale: f32,
) -> [f32; 3] {
    [
        (position[0] - source_center.x) * scale + target_center.x,
        (position[1] - source_center.y) * scale + target_center.y,
        position[2] * scale,
    ]
}
