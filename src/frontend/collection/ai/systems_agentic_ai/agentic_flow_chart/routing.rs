use glam::{Vec2, Vec3};

use super::edge::FlowEdge;
use super::shapes::{dedup_points, next_level_val, polyline_length};
use super::types::{EdgeStep, FlowChartDirection, NodeLayout};

/// Edge routing engine - calculates paths between nodes
pub struct EdgeRouter;

impl EdgeRouter {
    /// Calculate the route for an edge between two nodes
    pub fn route(
        layouts: &[NodeLayout],
        from: usize,
        to: usize,
        edge: Option<&FlowEdge>,
        direction: FlowChartDirection,
        lane_gap: f32,
        node_gap: f32,
        default_node_size: Vec2,
    ) -> Option<Vec<Vec3>> {
        let start = *layouts.get(from)?;
        let end = *layouts.get(to)?;

        // Use manual routing if steps are provided
        if let Some(edge) = edge {
            if !edge.route_steps.is_empty() {
                return Some(dedup_points(Self::manual_route(
                    layouts,
                    from,
                    to,
                    &edge.route_steps,
                    lane_gap,
                    node_gap,
                    default_node_size,
                )));
            }
        }

        let span = from.abs_diff(to).max(1) as f32;
        let lane_offset = lane_gap * span;
        let stub = 0.28;
        let delta = end.center - start.center;
        let prefer_horizontal = delta.x.abs() >= delta.y.abs();

        let points = match direction {
            FlowChartDirection::Horizontal => Self::route_horizontal(
                start,
                end,
                from,
                to,
                layouts,
                prefer_horizontal,
                delta,
                stub,
                lane_offset,
            ),
            FlowChartDirection::Vertical => Self::route_vertical(
                start,
                end,
                from,
                to,
                layouts,
                prefer_horizontal,
                delta,
                stub,
                lane_offset,
            ),
        };

        Some(dedup_points(points))
    }

    fn route_horizontal(
        start: NodeLayout,
        end: NodeLayout,
        from: usize,
        to: usize,
        layouts: &[NodeLayout],
        prefer_horizontal: bool,
        delta: Vec3,
        stub: f32,
        lane_offset: f32,
    ) -> Vec<Vec3> {
        if to == from + 1 {
            Self::adjacent_route(start, end, prefer_horizontal, delta)
        } else if to > from {
            Self::forward_horizontal_route(start, end, stub, lane_offset)
        } else {
            Self::best_route(
                layouts,
                from,
                to,
                vec![
                    Self::horizontal_loop_route(
                        start,
                        end,
                        start.size.y * 0.5,
                        end.size.y * 0.5,
                        lane_offset,
                        true,
                    ),
                    Self::horizontal_loop_route(
                        start,
                        end,
                        -start.size.y * 0.5,
                        -end.size.y * 0.5,
                        lane_offset,
                        false,
                    ),
                ],
            )
        }
    }

    fn route_vertical(
        start: NodeLayout,
        end: NodeLayout,
        from: usize,
        to: usize,
        layouts: &[NodeLayout],
        prefer_horizontal: bool,
        delta: Vec3,
        stub: f32,
        lane_offset: f32,
    ) -> Vec<Vec3> {
        if to == from + 1 {
            Self::adjacent_route_vertical(start, end, prefer_horizontal, delta)
        } else if to > from {
            Self::forward_vertical_route(start, end, stub, lane_offset)
        } else {
            Self::best_route(
                layouts,
                from,
                to,
                vec![
                    Self::vertical_loop_route(
                        start,
                        end,
                        start.size.x * 0.5,
                        end.size.x * 0.5,
                        lane_offset,
                        true,
                    ),
                    Self::vertical_loop_route(
                        start,
                        end,
                        -start.size.x * 0.5,
                        -end.size.x * 0.5,
                        lane_offset,
                        false,
                    ),
                ],
            )
        }
    }

    fn adjacent_route(
        start: NodeLayout,
        end: NodeLayout,
        prefer_horizontal: bool,
        delta: Vec3,
    ) -> Vec<Vec3> {
        let (start_anchor, end_anchor) = if prefer_horizontal {
            if delta.x >= 0.0 {
                (
                    Vec3::new(start.center.x + start.size.x * 0.5, start.center.y, 0.0),
                    Vec3::new(end.center.x - end.size.x * 0.5, end.center.y, 0.0),
                )
            } else {
                (
                    Vec3::new(start.center.x - start.size.x * 0.5, start.center.y, 0.0),
                    Vec3::new(end.center.x + end.size.x * 0.5, end.center.y, 0.0),
                )
            }
        } else if delta.y >= 0.0 {
            (
                Vec3::new(start.center.x, start.center.y + start.size.y * 0.5, 0.0),
                Vec3::new(end.center.x, end.center.y - end.size.y * 0.5, 0.0),
            )
        } else {
            (
                Vec3::new(start.center.x, start.center.y - start.size.y * 0.5, 0.0),
                Vec3::new(end.center.x, end.center.y + end.size.y * 0.5, 0.0),
            )
        };
        vec![start_anchor, end_anchor]
    }

    fn adjacent_route_vertical(
        start: NodeLayout,
        end: NodeLayout,
        prefer_horizontal: bool,
        delta: Vec3,
    ) -> Vec<Vec3> {
        let (start_anchor, end_anchor) = if prefer_horizontal {
            if delta.x >= 0.0 {
                (
                    Vec3::new(start.center.x + start.size.x * 0.5, start.center.y, 0.0),
                    Vec3::new(end.center.x - end.size.x * 0.5, end.center.y, 0.0),
                )
            } else {
                (
                    Vec3::new(start.center.x - start.size.x * 0.5, start.center.y, 0.0),
                    Vec3::new(end.center.x + end.size.x * 0.5, end.center.y, 0.0),
                )
            }
        } else {
            (
                Vec3::new(start.center.x, start.center.y - start.size.y * 0.5, 0.0),
                Vec3::new(end.center.x, end.center.y + end.size.y * 0.5, 0.0),
            )
        };
        vec![start_anchor, end_anchor]
    }

    fn forward_horizontal_route(
        start: NodeLayout,
        end: NodeLayout,
        stub: f32,
        lane_offset: f32,
    ) -> Vec<Vec3> {
        let start_anchor = Vec3::new(start.center.x + start.size.x * 0.5, start.center.y, 0.0);
        let end_anchor = Vec3::new(end.center.x - end.size.x * 0.5, end.center.y, 0.0);
        let lane_y = lane_offset;
        vec![
            start_anchor,
            Vec3::new(start_anchor.x + stub, start_anchor.y, 0.0),
            Vec3::new(start_anchor.x + stub, lane_y, 0.0),
            Vec3::new(end_anchor.x - stub, lane_y, 0.0),
            Vec3::new(end_anchor.x - stub, end_anchor.y, 0.0),
            end_anchor,
        ]
    }

    fn forward_vertical_route(
        start: NodeLayout,
        end: NodeLayout,
        stub: f32,
        lane_offset: f32,
    ) -> Vec<Vec3> {
        let start_anchor = Vec3::new(start.center.x, start.center.y - start.size.y * 0.5, 0.0);
        let end_anchor = Vec3::new(end.center.x, end.center.y + end.size.y * 0.5, 0.0);
        let lane_x = lane_offset;
        vec![
            start_anchor,
            Vec3::new(start_anchor.x, start_anchor.y - stub, 0.0),
            Vec3::new(lane_x, start_anchor.y - stub, 0.0),
            Vec3::new(lane_x, end_anchor.y + stub, 0.0),
            Vec3::new(end_anchor.x, end_anchor.y + stub, 0.0),
            end_anchor,
        ]
    }

    fn horizontal_loop_route(
        start: NodeLayout,
        end: NodeLayout,
        start_y_offset: f32,
        end_y_offset: f32,
        lane_offset: f32,
        above: bool,
    ) -> Vec<Vec3> {
        let start_anchor = Vec3::new(start.center.x, start.center.y + start_y_offset, 0.0);
        let end_anchor = Vec3::new(end.center.x, end.center.y + end_y_offset, 0.0);
        let lane_y = if above {
            start_anchor.y.max(end_anchor.y) + lane_offset
        } else {
            start_anchor.y.min(end_anchor.y) - lane_offset
        };

        vec![
            start_anchor,
            Vec3::new(start_anchor.x, lane_y, 0.0),
            Vec3::new(end_anchor.x, lane_y, 0.0),
            end_anchor,
        ]
    }

    fn vertical_loop_route(
        start: NodeLayout,
        end: NodeLayout,
        start_x_offset: f32,
        end_x_offset: f32,
        lane_offset: f32,
        right_side: bool,
    ) -> Vec<Vec3> {
        let start_anchor = Vec3::new(start.center.x + start_x_offset, start.center.y, 0.0);
        let end_anchor = Vec3::new(end.center.x + end_x_offset, end.center.y, 0.0);
        let lane_x = if right_side {
            start_anchor.x.max(end_anchor.x) + lane_offset
        } else {
            start_anchor.x.min(end_anchor.x) - lane_offset
        };

        vec![
            start_anchor,
            Vec3::new(lane_x, start_anchor.y, 0.0),
            Vec3::new(lane_x, end_anchor.y, 0.0),
            end_anchor,
        ]
    }

    /// Manual routing using explicit step directions
    fn manual_route(
        layouts: &[NodeLayout],
        from: usize,
        to: usize,
        steps: &[EdgeStep],
        lane_gap: f32,
        node_gap: f32,
        default_node_size: Vec2,
    ) -> Vec<Vec3> {
        let Some(first_step) = steps.first().copied() else {
            return Vec::new();
        };
        let last_step = steps.last().copied().unwrap_or(first_step);

        let mut x_levels: Vec<f32> = layouts.iter().map(|l| l.center.x).collect();
        let mut y_levels: Vec<f32> = layouts.iter().map(|l| l.center.y).collect();

        let dedup = |v: &mut Vec<f32>| {
            v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            v.dedup_by(|a, b| (*a - *b).abs() < 1e-4);
        };
        dedup(&mut x_levels);
        dedup(&mut y_levels);

        let start_node = layouts[from];
        let end_node = layouts[to];

        let start_anchor = anchor_for_step(start_node, first_step);
        let end_anchor = anchor_for_step(end_node, opposite_step(last_step));

        let mut cur_x = start_node.center.x;
        let mut cur_y = start_node.center.y;

        let mut points = vec![start_anchor];

        for (idx, step) in steps.iter().copied().enumerate() {
            let is_last = idx == steps.len() - 1;

            match step {
                EdgeStep::Up => {
                    cur_y = next_level_val(&y_levels, cur_y, 1, lane_gap + default_node_size.y);
                }
                EdgeStep::Down => {
                    cur_y = next_level_val(&y_levels, cur_y, -1, lane_gap + default_node_size.y);
                }
                EdgeStep::Left => {
                    cur_x = next_level_val(&x_levels, cur_x, -1, node_gap + default_node_size.x);
                }
                EdgeStep::Right => {
                    cur_x = next_level_val(&x_levels, cur_x, 1, node_gap + default_node_size.x);
                }
            }

            let next = if is_last {
                end_anchor
            } else {
                Vec3::new(cur_x, cur_y, 0.0)
            };

            points.push(next);
        }

        points
    }

    /// Choose the best route from multiple candidates based on scoring
    fn best_route(
        layouts: &[NodeLayout],
        from: usize,
        to: usize,
        candidates: Vec<Vec<Vec3>>,
    ) -> Vec<Vec3> {
        candidates
            .into_iter()
            .min_by(|a, b| {
                route_score(layouts, from, to, a)
                    .partial_cmp(&route_score(layouts, from, to, b))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or_default()
    }
}

/// Calculate anchor point for a given edge step direction
fn anchor_for_step(layout: NodeLayout, step: EdgeStep) -> Vec3 {
    match step {
        EdgeStep::Up => Vec3::new(layout.center.x, layout.center.y + layout.size.y * 0.5, 0.0),
        EdgeStep::Down => Vec3::new(layout.center.x, layout.center.y - layout.size.y * 0.5, 0.0),
        EdgeStep::Left => Vec3::new(layout.center.x - layout.size.x * 0.5, layout.center.y, 0.0),
        EdgeStep::Right => Vec3::new(layout.center.x + layout.size.x * 0.5, layout.center.y, 0.0),
    }
}

/// Get the opposite direction of an edge step
fn opposite_step(step: EdgeStep) -> EdgeStep {
    match step {
        EdgeStep::Up => EdgeStep::Down,
        EdgeStep::Down => EdgeStep::Up,
        EdgeStep::Left => EdgeStep::Right,
        EdgeStep::Right => EdgeStep::Left,
    }
}

/// Score a route based on node intersections and length
fn route_score(layouts: &[NodeLayout], from: usize, to: usize, route: &[Vec3]) -> f32 {
    let intersections = layouts
        .iter()
        .enumerate()
        .filter(|(idx, _)| *idx != from && *idx != to)
        .map(|(_, layout)| route_node_overlap_score(route, *layout))
        .sum::<f32>();
    intersections * 1000.0 + polyline_length(route)
}

/// Calculate overlap score between a route and a node
fn route_node_overlap_score(route: &[Vec3], layout: NodeLayout) -> f32 {
    route
        .windows(2)
        .map(|segment| segment_overlap_score(segment[0], segment[1], layout))
        .sum()
}

/// Calculate overlap between a line segment and a node's bounding box
fn segment_overlap_score(a: Vec3, b: Vec3, layout: NodeLayout) -> f32 {
    let pad = 0.18;
    let half = layout.size * 0.5 + Vec2::splat(pad);
    let min = layout.center.truncate() - half;
    let max = layout.center.truncate() + half;

    let a2 = a.truncate();
    let b2 = b.truncate();

    // Vertical segment
    if (a2.x - b2.x).abs() <= 1e-4 {
        if a2.x < min.x || a2.x > max.x {
            return 0.0;
        }
        let seg_min = a2.y.min(b2.y);
        let seg_max = a2.y.max(b2.y);
        let overlap = (seg_max.min(max.y) - seg_min.max(min.y)).max(0.0);
        return overlap;
    }

    // Horizontal segment
    if (a2.y - b2.y).abs() <= 1e-4 {
        if a2.y < min.y || a2.y > max.y {
            return 0.0;
        }
        let seg_min = a2.x.min(b2.x);
        let seg_max = a2.x.max(b2.x);
        let overlap = (seg_max.min(max.x) - seg_min.max(min.x)).max(0.0);
        return overlap;
    }

    0.0
}
