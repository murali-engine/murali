use glam::{Vec2, Vec3, vec3};

use super::node::FlowNode;
use super::types::{FlowChartDirection, FlowNodePlacement, NodeLayout};

/// Calculate the resolved size for a node (either custom or default)
pub(super) fn resolved_node_size(
    node: &FlowNode,
    default_size: Vec2,
    text_height: f32,
    text_padding: Vec2,
) -> Vec2 {
    if let Some(size) = node.size {
        return size;
    }

    if node.embedded_content.is_some() {
        return default_size;
    }

    if node.label.is_empty() {
        return default_size;
    }

    let char_count = node.label.chars().count();
    let estimated_width = (char_count as f32 * text_height * 0.55).max(default_size.x * 0.5);
    Vec2::new(
        estimated_width + text_padding.x * 2.0,
        text_height + text_padding.y * 2.0,
    )
}

/// Calculate layouts for all nodes based on direction and placement
pub(super) fn calculate_node_layouts(
    nodes: &[FlowNode],
    direction: FlowChartDirection,
    node_gap: f32,
    default_size: Vec2,
    text_height: f32,
    text_padding: Vec2,
) -> Vec<NodeLayout> {
    if nodes.is_empty() {
        return Vec::new();
    }

    let sizes: Vec<Vec2> = nodes
        .iter()
        .map(|node| resolved_node_size(node, default_size, text_height, text_padding))
        .collect();

    let has_custom_placements = nodes.iter().skip(1).any(|node| node.placement.is_some());

    if has_custom_placements {
        return calculate_custom_placement_layouts(nodes, &sizes, direction, node_gap);
    }

    calculate_linear_layouts(&sizes, direction, node_gap)
}

/// Calculate layouts with custom placements
fn calculate_custom_placement_layouts(
    nodes: &[FlowNode],
    sizes: &[Vec2],
    direction: FlowChartDirection,
    node_gap: f32,
) -> Vec<NodeLayout> {
    let mut layouts = Vec::with_capacity(nodes.len());
    layouts.push(NodeLayout {
        center: Vec3::ZERO,
        size: sizes[0],
    });

    for idx in 1..nodes.len() {
        let previous = layouts[idx - 1];
        let size = sizes[idx];
        let placement = nodes[idx].placement.unwrap_or(match direction {
            FlowChartDirection::Horizontal => FlowNodePlacement::RightOfPrevious,
            FlowChartDirection::Vertical => FlowNodePlacement::BelowPrevious,
        });

        let center = match placement {
            FlowNodePlacement::RightOfPrevious => vec3(
                previous.center.x + previous.size.x * 0.5 + node_gap + size.x * 0.5,
                previous.center.y,
                0.0,
            ),
            FlowNodePlacement::LeftOfPrevious => vec3(
                previous.center.x - previous.size.x * 0.5 - node_gap - size.x * 0.5,
                previous.center.y,
                0.0,
            ),
            FlowNodePlacement::AbovePrevious => vec3(
                previous.center.x,
                previous.center.y + previous.size.y * 0.5 + node_gap + size.y * 0.5,
                0.0,
            ),
            FlowNodePlacement::BelowPrevious => vec3(
                previous.center.x,
                previous.center.y - previous.size.y * 0.5 - node_gap - size.y * 0.5,
                0.0,
            ),
        };
        layouts.push(NodeLayout { center, size });
    }

    // Center the entire layout
    let (min, max) = layout_extents(&layouts);
    let center = (min + max) * 0.5;
    for layout in &mut layouts {
        layout.center.x -= center.x;
        layout.center.y -= center.y;
    }

    layouts
}

/// Calculate linear layouts (horizontal or vertical)
fn calculate_linear_layouts(
    sizes: &[Vec2],
    direction: FlowChartDirection,
    node_gap: f32,
) -> Vec<NodeLayout> {
    let total_primary: f32 = sizes
        .iter()
        .map(|size| match direction {
            FlowChartDirection::Horizontal => size.x,
            FlowChartDirection::Vertical => size.y,
        })
        .sum::<f32>()
        + node_gap * sizes.len().saturating_sub(1) as f32;

    let mut cursor = -total_primary * 0.5;
    let mut layouts = Vec::with_capacity(sizes.len());

    for &size in sizes {
        let center = match direction {
            FlowChartDirection::Horizontal => {
                let x = cursor + size.x * 0.5;
                cursor += size.x + node_gap;
                vec3(x, 0.0, 0.0)
            }
            FlowChartDirection::Vertical => {
                let y = total_primary * 0.5 - (cursor.abs() + size.y * 0.5);
                cursor += size.y + node_gap;
                vec3(0.0, y, 0.0)
            }
        };
        layouts.push(NodeLayout { center, size });
    }

    // Fix vertical layout positioning
    if direction == FlowChartDirection::Vertical {
        let mut cursor = total_primary * 0.5;
        for layout in &mut layouts {
            layout.center.y = cursor - layout.size.y * 0.5;
            cursor -= layout.size.y + node_gap;
        }
    }

    layouts
}

/// Calculate the bounding box of all layouts
pub(super) fn layout_extents(layouts: &[NodeLayout]) -> (Vec2, Vec2) {
    let mut min = Vec2::splat(f32::INFINITY);
    let mut max = Vec2::splat(f32::NEG_INFINITY);

    for layout in layouts {
        let half = layout.size * 0.5;
        min = min.min(layout.center.truncate() - half);
        max = max.max(layout.center.truncate() + half);
    }

    (min, max)
}
