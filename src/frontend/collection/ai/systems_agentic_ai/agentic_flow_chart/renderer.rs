use std::collections::HashSet;

use glam::{Vec2, Vec3, Vec4};

use crate::backend::renderer::vertex::{mesh::MeshVertex, text::TextVertex};
use crate::projection::mesh::MeshData;
use crate::projection::{Mesh, ProjectionCtx, RenderPrimitive};

use super::animation::{AnimationEngine, FlowAnimationState};
use super::edge::FlowEdge;
use super::node::FlowNode;
use super::shapes::{partial_polyline, shape_mesh, shape_outline};
use super::types::*;

/// Rendering context - decoupled from AgenticFlowChart
pub struct RenderContext<'a> {
    pub nodes: &'a [FlowNode],
    pub edges: &'a [FlowEdge],
    pub flow_path: &'a [usize],
    pub label_ids: &'a [Option<crate::frontend::TattvaId>],
    pub active_content_nodes: &'a HashSet<usize>,

    // Style
    pub edge_color: Vec4,
    pub active_edge_color: Vec4,
    pub edge_thickness: f32,
    pub arrow_size: f32,
    pub pulse_radius: f32,
    pub pulse_color: Vec4,
    pub indicate_color: Vec4,
    pub indicate_scale: f32,
    pub text_height: f32,

    // Animation config
    pub node_animation_style: NodeAnimationStyle,
    pub edge_animation_style: EdgeAnimationStyle,
    pub reveal_progress: f32,
    pub progress: f32,
    pub node_reveal_delay: f32,
    pub node_reveal_window: f32,
    pub edge_reveal_window: f32,
    pub progressive_edges: bool,
}

/// Flow chart renderer - pure rendering logic
pub struct FlowRenderer;

impl FlowRenderer {
    /// Main render entry point
    pub fn render(
        ctx: &mut ProjectionCtx,
        render_ctx: &RenderContext,
        layouts: &[NodeLayout],
        anim_state: &FlowAnimationState,
        edge_route_fn: impl Fn(usize, usize) -> Option<Vec<Vec3>>,
    ) {
        // 1. Draw static edges
        Self::render_static_edges(ctx, render_ctx, layouts, anim_state, &edge_route_fn);

        // 2. Draw flow path edges
        Self::render_flow_edges(ctx, render_ctx, layouts, anim_state, &edge_route_fn);

        // 3. Draw nodes
        Self::render_nodes(ctx, render_ctx, layouts, anim_state);

        // 4. Draw indication highlights
        Self::render_indications(ctx, render_ctx, layouts, anim_state);

        // 5. Draw node content (text labels)
        Self::render_node_content(ctx, render_ctx, layouts, anim_state);

        // 6. Draw pulse
        Self::render_pulse(ctx, render_ctx, anim_state);
    }

    fn render_static_edges(
        ctx: &mut ProjectionCtx,
        render_ctx: &RenderContext,
        layouts: &[NodeLayout],
        anim_state: &FlowAnimationState,
        edge_route_fn: &impl Fn(usize, usize) -> Option<Vec<Vec3>>,
    ) {
        // Draw ALL edges during reveal phase using their original index for threshold lookup.
        // Flow-path edges will be drawn again (highlighted) by render_flow_edges during propagation.
        for (idx, edge) in render_ctx.edges.iter().enumerate() {
            let threshold = anim_state.edge_thresholds.get(idx).copied().unwrap_or(0.0);
            if render_ctx.reveal_progress < threshold {
                continue;
            }

            if !AnimationEngine::should_draw_edge_progressive(
                edge,
                render_ctx.reveal_progress,
                &anim_state.node_thresholds,
                render_ctx.progressive_edges,
            ) {
                continue;
            }

            if let Some(route) = edge_route_fn(edge.from, edge.to) {
                let window = edge.reveal_window.unwrap_or(render_ctx.edge_reveal_window);
                let (draw_route, draw_color) = match render_ctx.edge_animation_style {
                    EdgeAnimationStyle::Write => {
                        let edge_progress =
                            ((render_ctx.reveal_progress - threshold) / window).clamp(0.0, 1.0);
                        if edge_progress <= 0.001 {
                            continue;
                        }
                        (
                            partial_polyline(&route, edge_progress),
                            render_ctx.edge_color,
                        )
                    }
                    EdgeAnimationStyle::Instant => (route, render_ctx.edge_color),
                };

                emit_polyline(ctx, &draw_route, render_ctx.edge_thickness, draw_color);

                let show_arrow = match render_ctx.edge_animation_style {
                    EdgeAnimationStyle::Write => {
                        let edge_progress =
                            ((render_ctx.reveal_progress - threshold) / window).clamp(0.0, 1.0);
                        edge_progress > 0.95
                    }
                    EdgeAnimationStyle::Instant => true,
                };

                if show_arrow {
                    emit_arrowhead(
                        ctx,
                        &draw_route,
                        render_ctx.arrow_size,
                        render_ctx.edge_thickness,
                        draw_color,
                    );
                }
            }
        }
    }

    fn render_flow_edges(
        ctx: &mut ProjectionCtx,
        render_ctx: &RenderContext,
        layouts: &[NodeLayout],
        anim_state: &FlowAnimationState,
        edge_route_fn: &impl Fn(usize, usize) -> Option<Vec<Vec3>>,
    ) {
        if render_ctx.flow_path.len() < 2 {
            return;
        }

        let progress = render_ctx.progress.clamp(0.0, 1.0);
        let hop_count = render_ctx.flow_path.len() - 1;
        let scaled = progress * hop_count as f32;
        let current_hop = scaled.floor() as usize;
        let current_t = if current_hop >= hop_count {
            1.0
        } else {
            scaled - current_hop as f32
        };

        for hop in 0..hop_count {
            let from_idx = render_ctx.flow_path[hop];
            let to_idx = render_ctx.flow_path[hop + 1];
            let edge_idx = render_ctx
                .edges
                .iter()
                .position(|e| e.from == from_idx && e.to == to_idx);

            if let Some(e_idx) = edge_idx {
                let threshold = anim_state
                    .edge_thresholds
                    .get(e_idx)
                    .copied()
                    .unwrap_or(0.0);
                if render_ctx.reveal_progress < threshold {
                    continue;
                }
            }

            let Some(route) = edge_route_fn(from_idx, to_idx) else {
                continue;
            };

            if hop < current_hop {
                emit_polyline(
                    ctx,
                    &route,
                    render_ctx.edge_thickness * 1.15,
                    render_ctx.active_edge_color,
                );
                emit_arrowhead(
                    ctx,
                    &route,
                    render_ctx.arrow_size,
                    render_ctx.edge_thickness * 1.15,
                    render_ctx.active_edge_color,
                );
            } else if hop == current_hop && progress < 1.0 {
                let partial = partial_polyline(&route, current_t);
                emit_polyline(
                    ctx,
                    &partial,
                    render_ctx.edge_thickness * 1.15,
                    render_ctx.active_edge_color,
                );
                emit_arrowhead(
                    ctx,
                    &partial,
                    render_ctx.arrow_size,
                    render_ctx.edge_thickness * 1.15,
                    render_ctx.active_edge_color,
                );
            } else if hop == current_hop && progress >= 1.0 {
                emit_polyline(
                    ctx,
                    &route,
                    render_ctx.edge_thickness * 1.15,
                    render_ctx.active_edge_color,
                );
                emit_arrowhead(
                    ctx,
                    &route,
                    render_ctx.arrow_size,
                    render_ctx.edge_thickness * 1.15,
                    render_ctx.active_edge_color,
                );
            }
        }
    }

    fn render_nodes(
        ctx: &mut ProjectionCtx,
        render_ctx: &RenderContext,
        layouts: &[NodeLayout],
        anim_state: &FlowAnimationState,
    ) {
        let v_node_indices: Vec<usize> = (0..render_ctx.nodes.len()).collect();

        for (idx, node) in render_ctx.nodes.iter().enumerate() {
            let v_idx = v_node_indices[idx];
            let threshold = anim_state
                .node_thresholds
                .get(v_idx)
                .copied()
                .unwrap_or(0.0);

            let draw_threshold = match render_ctx.node_animation_style {
                NodeAnimationStyle::Write => threshold + render_ctx.node_reveal_delay,
                _ => threshold,
            };

            if render_ctx.reveal_progress < draw_threshold {
                continue;
            }

            if let Some(layout) = layouts.get(idx).copied() {
                let intensity = AnimationEngine::node_indicate_intensity(
                    idx,
                    render_ctx.flow_path,
                    render_ctx.progress,
                    render_ctx.reveal_progress,
                    &anim_state.node_thresholds,
                    0.18, // indicate_window - should be in render_ctx
                );
                let node_scale = 1.0 + render_ctx.indicate_scale * intensity * 0.82;
                Self::draw_node(
                    ctx,
                    render_ctx,
                    idx,
                    node,
                    layout,
                    node_scale,
                    node.stroke_color,
                    render_ctx.edge_thickness * 0.9,
                    &anim_state.node_thresholds,
                );
            }
        }
    }

    fn draw_node(
        ctx: &mut ProjectionCtx,
        render_ctx: &RenderContext,
        node_index: usize,
        node: &FlowNode,
        layout: NodeLayout,
        scale: f32,
        stroke_color: Vec4,
        stroke_thickness: f32,
        node_thresholds: &[f32],
    ) {
        let size = layout.size * scale;
        let outline = shape_outline(node.shape, size);

        match render_ctx.node_animation_style {
            NodeAnimationStyle::Write => {
                let write_progress = AnimationEngine::node_write_progress(
                    node_index,
                    render_ctx.reveal_progress,
                    node_thresholds,
                    render_ctx.node_reveal_delay,
                    render_ctx.node_reveal_window,
                );

                if write_progress <= 0.0 {
                    return;
                }

                let outline_3d = translate_points(&outline, layout.center);

                let partial_outline = if write_progress < 1.0 {
                    let partial = partial_polyline(&outline_3d, write_progress);
                    emit_polyline(ctx, &partial, stroke_thickness, stroke_color);
                    partial
                } else {
                    emit_closed_polyline(ctx, &outline_3d, stroke_thickness, stroke_color);
                    outline_3d.clone()
                };

                let has_external_label = render_ctx
                    .label_ids
                    .get(node_index)
                    .and_then(|id| id.as_ref())
                    .is_some();

                if !has_external_label && partial_outline.len() >= 3 {
                    use lyon_tessellation as lyon;
                    use lyon_tessellation::path::Path as LyonPath;
                    use lyon_tessellation::{FillOptions, FillTessellator, VertexBuffers};

                    let mut builder = LyonPath::builder();

                    if let Some(&first_pt) = partial_outline.first() {
                        builder.begin(lyon::math::point(first_pt.x, first_pt.y));

                        for &pt in &partial_outline[1..] {
                            builder.line_to(lyon::math::point(pt.x, pt.y));
                        }

                        builder.end(true);
                    }

                    let lpath = builder.build();
                    let mut tessellator = FillTessellator::new();
                    let mut geometry: VertexBuffers<lyon::math::Point, u16> = VertexBuffers::new();

                    let res = tessellator.tessellate_path(
                        &lpath,
                        &FillOptions::default(),
                        &mut lyon::geometry_builder::simple_builder(&mut geometry),
                    );

                    if res.is_ok() {
                        let vertices: Vec<MeshVertex> = geometry
                            .vertices
                            .iter()
                            .map(|v| MeshVertex {
                                position: [v.x, v.y, 0.0],
                                color: node.fill_color.into(),
                            })
                            .collect();

                        let indices = geometry.indices.into_iter().map(u32::from).collect();
                        let mesh = Mesh::from_tessellation(vertices, indices);
                        ctx.emit(RenderPrimitive::Mesh(mesh));
                    }
                }
            }
            NodeAnimationStyle::Drop => {
                let mesh = shape_mesh(node.shape, size, node.fill_color)
                    .as_ref()
                    .clone()
                    .translated(layout.center);
                ctx.emit(RenderPrimitive::Mesh(mesh));

                let outline_3d = translate_points(&outline, layout.center);
                emit_closed_polyline(ctx, &outline_3d, stroke_thickness, stroke_color);
            }
            NodeAnimationStyle::Instant => {
                let mesh = shape_mesh(node.shape, size, node.fill_color)
                    .as_ref()
                    .clone()
                    .translated(layout.center);
                ctx.emit(RenderPrimitive::Mesh(mesh));

                let outline_3d = translate_points(&outline, layout.center);
                emit_closed_polyline(ctx, &outline_3d, stroke_thickness, stroke_color);
            }
        }
    }

    fn render_indications(
        ctx: &mut ProjectionCtx,
        render_ctx: &RenderContext,
        layouts: &[NodeLayout],
        anim_state: &FlowAnimationState,
    ) {
        for (idx, node) in render_ctx.nodes.iter().enumerate() {
            let intensity = AnimationEngine::node_indicate_intensity(
                idx,
                render_ctx.flow_path,
                render_ctx.progress,
                render_ctx.reveal_progress,
                &anim_state.node_thresholds,
                0.18, // indicate_window
            );
            if intensity <= 0.01 {
                continue;
            }
            if let Some(layout) = layouts.get(idx).copied() {
                let mut indicate_color = render_ctx.indicate_color;
                indicate_color.w *= 0.35 + intensity * 0.55;
                let scale = 1.0 + render_ctx.indicate_scale * intensity;
                let outline = shape_outline(node.shape, layout.size * scale);
                let outline_3d = translate_points(&outline, layout.center);
                emit_closed_polyline(
                    ctx,
                    &outline_3d,
                    render_ctx.edge_thickness * 1.2,
                    indicate_color,
                );
            }
        }
    }

    fn render_node_content(
        ctx: &mut ProjectionCtx,
        render_ctx: &RenderContext,
        layouts: &[NodeLayout],
        anim_state: &FlowAnimationState,
    ) {
        for (idx, node) in render_ctx.nodes.iter().enumerate() {
            if let Some(layout) = layouts.get(idx).copied() {
                let intensity = AnimationEngine::node_indicate_intensity(
                    idx,
                    render_ctx.flow_path,
                    render_ctx.progress,
                    render_ctx.reveal_progress,
                    &anim_state.node_thresholds,
                    0.18,
                );
                let scale = 1.0 + render_ctx.indicate_scale * intensity * 0.82;
                let content_is_visible = match node.content_visibility {
                    FlowNodeContentVisibility::Always => true,
                    FlowNodeContentVisibility::ActiveOnly => {
                        render_ctx.active_content_nodes.contains(&idx)
                    }
                };

                if content_is_visible {
                    if let Some(content) = &node.embedded_content {
                        let content_size = Vec2::new(
                            (layout.size.x - node.content_padding.x * 2.0).max(0.01),
                            (layout.size.y - node.content_padding.y * 2.0).max(0.01),
                        ) * scale;
                        project_content_in_node(ctx, content.as_ref(), layout.center, content_size);
                    }
                }

                let should_render_text = render_ctx
                    .label_ids
                    .get(idx)
                    .and_then(|id| id.as_ref())
                    .is_none();

                if should_render_text {
                    let (text_content, text_color) = match render_ctx.node_animation_style {
                        NodeAnimationStyle::Write => {
                            let write_progress = AnimationEngine::node_write_progress(
                                idx,
                                render_ctx.reveal_progress,
                                &anim_state.node_thresholds,
                                render_ctx.node_reveal_delay,
                                render_ctx.node_reveal_window,
                            );

                            let total_chars = node.label.chars().count();
                            let chars_to_show =
                                (total_chars as f32 * write_progress).ceil() as usize;
                            let revealed_text: String =
                                node.label.chars().take(chars_to_show).collect();

                            (revealed_text, node.text_color)
                        }
                        _ => (node.label.clone(), node.text_color),
                    };

                    if !text_content.is_empty() {
                        ctx.emit(RenderPrimitive::Text {
                            content: text_content,
                            height: render_ctx.text_height * scale.min(1.08),
                            color: text_color,
                            font_name: None,
                            offset: layout.center,
                            rotation: 0.0,
                        });
                    }
                }
            }
        }
    }

    fn render_pulse(
        ctx: &mut ProjectionCtx,
        render_ctx: &RenderContext,
        anim_state: &FlowAnimationState,
    ) {
        if let Some(pulse_pos) = anim_state.pulse_position {
            let mesh = Mesh::circle(render_ctx.pulse_radius, 28, render_ctx.pulse_color)
                .as_ref()
                .clone()
                .translated(pulse_pos);
            ctx.emit(RenderPrimitive::Mesh(mesh));
        }
    }
}

fn project_content_in_node(
    ctx: &mut ProjectionCtx,
    content: &dyn super::node::FlowNodeContent,
    node_center: Vec3,
    target_size: Vec2,
) {
    let source_bounds = content.local_bounds();
    let source_size = Vec2::new(
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
    Vec3::new(
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

// Helper functions for rendering primitives

fn translate_points(points: &[Vec2], offset: Vec3) -> Vec<Vec3> {
    points
        .iter()
        .map(|point| Vec3::new(point.x + offset.x, point.y + offset.y, offset.z))
        .collect()
}

fn emit_polyline(ctx: &mut ProjectionCtx, points: &[Vec3], thickness: f32, color: Vec4) {
    if points.len() < 2 {
        return;
    }
    for segment in points.windows(2) {
        let start = segment[0];
        let end = segment[1];
        if start.distance_squared(end) <= 1e-6 {
            continue;
        }
        ctx.emit(RenderPrimitive::Line {
            start,
            end,
            thickness,
            color,
            dash_length: 0.0,
            gap_length: 0.0,
            dash_offset: 0.0,
        });
    }
}

fn emit_closed_polyline(ctx: &mut ProjectionCtx, points: &[Vec3], thickness: f32, color: Vec4) {
    if points.len() < 2 {
        return;
    }
    emit_polyline(ctx, points, thickness, color);
    if let (Some(&start), Some(&end)) = (points.first(), points.last()) {
        if start.distance_squared(end) > 1e-6 {
            ctx.emit(RenderPrimitive::Line {
                start: end,
                end: start,
                thickness,
                color,
                dash_length: 0.0,
                gap_length: 0.0,
                dash_offset: 0.0,
            });
        }
    }
}

fn emit_arrowhead(
    ctx: &mut ProjectionCtx,
    route: &[Vec3],
    arrow_size: f32,
    thickness: f32,
    color: Vec4,
) {
    if route.len() < 2 {
        return;
    }
    let end = route[route.len() - 1];
    let prev = route[route.len() - 2];
    let dir = (end - prev).normalize();
    let perp = Vec3::new(-dir.y, dir.x, 0.0);

    let tip = end;
    let base = end - dir * arrow_size;
    let left = base + perp * arrow_size * 0.5;
    let right = base - perp * arrow_size * 0.5;

    let arrow_points = vec![left, tip, right];
    emit_polyline(ctx, &arrow_points, thickness, color);
}
