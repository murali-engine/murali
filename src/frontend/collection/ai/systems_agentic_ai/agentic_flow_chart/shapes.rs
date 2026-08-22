use glam::{Vec2, Vec3, vec2};
use std::sync::Arc;

use super::types::FlowNodeShape;
use crate::projection::Mesh;

/// Generate a mesh for a given shape
pub(super) fn shape_mesh(shape: FlowNodeShape, size: Vec2, color: glam::Vec4) -> Arc<Mesh> {
    match shape {
        FlowNodeShape::Rectangle => Mesh::rectangle(size.x, size.y, color),
        FlowNodeShape::Rounded | FlowNodeShape::Pill | FlowNodeShape::Diamond => {
            Mesh::polygon(shape_outline(shape, size), color)
        }
    }
}

/// Generate an outline (list of points) for a given shape
pub(super) fn shape_outline(shape: FlowNodeShape, size: Vec2) -> Vec<Vec2> {
    match shape {
        FlowNodeShape::Rectangle => rectangle_outline(size),
        FlowNodeShape::Rounded => {
            rounded_rect_outline(size, (size.x.min(size.y) * 0.22).min(size.y * 0.45), 8)
        }
        FlowNodeShape::Pill => rounded_rect_outline(size, (size.y * 0.5).min(size.x * 0.45), 10),
        FlowNodeShape::Diamond => vec![
            vec2(0.0, size.y * 0.5),
            vec2(size.x * 0.5, 0.0),
            vec2(0.0, -size.y * 0.5),
            vec2(-size.x * 0.5, 0.0),
        ],
    }
}

fn rectangle_outline(size: Vec2) -> Vec<Vec2> {
    vec![
        vec2(-size.x * 0.5, -size.y * 0.5),
        vec2(size.x * 0.5, -size.y * 0.5),
        vec2(size.x * 0.5, size.y * 0.5),
        vec2(-size.x * 0.5, size.y * 0.5),
    ]
}

fn rounded_rect_outline(size: Vec2, radius: f32, arc_steps: usize) -> Vec<Vec2> {
    let half = size * 0.5;
    let r = radius.max(0.01).min(half.x.min(half.y));
    let centers = [
        vec2(half.x - r, half.y - r),
        vec2(-(half.x - r), half.y - r),
        vec2(-(half.x - r), -(half.y - r)),
        vec2(half.x - r, -(half.y - r)),
    ];
    let ranges = [
        (0.0, std::f32::consts::FRAC_PI_2),
        (std::f32::consts::FRAC_PI_2, std::f32::consts::PI),
        (std::f32::consts::PI, std::f32::consts::PI * 1.5),
        (std::f32::consts::PI * 1.5, std::f32::consts::TAU),
    ];

    let mut points = Vec::new();
    for (center, (start, end)) in centers.into_iter().zip(ranges) {
        for step in 0..=arc_steps {
            let t = step as f32 / arc_steps.max(1) as f32;
            let angle = start + (end - start) * t;
            points.push(center + vec2(angle.cos() * r, angle.sin() * r));
        }
    }
    points
}

/// Calculate the total length of a polyline
pub(super) fn polyline_length(points: &[Vec3]) -> f32 {
    points
        .windows(2)
        .map(|segment| segment[0].distance(segment[1]))
        .sum()
}

/// Sample a point at position t (0.0 to 1.0) along a polyline
pub(super) fn sample_polyline(points: &[Vec3], t: f32) -> Option<Vec3> {
    let partial = partial_polyline(points, t);
    partial.last().copied()
}

/// Get a partial polyline up to position t (0.0 to 1.0)
pub(super) fn partial_polyline(points: &[Vec3], t: f32) -> Vec<Vec3> {
    if points.is_empty() {
        return Vec::new();
    }
    if points.len() == 1 {
        return points.to_vec();
    }

    let total_length = polyline_length(points);
    if total_length <= 1e-6 {
        return vec![points[0], *points.last().unwrap_or(&points[0])];
    }

    let target_length = total_length * t.clamp(0.0, 1.0);
    let mut traveled = 0.0;
    let mut result = vec![points[0]];

    for segment in points.windows(2) {
        let start = segment[0];
        let end = segment[1];
        let segment_length = start.distance(end);
        if segment_length <= 1e-6 {
            continue;
        }

        if traveled + segment_length < target_length {
            result.push(end);
            traveled += segment_length;
            continue;
        }

        let remaining = (target_length - traveled).clamp(0.0, segment_length);
        let local_t = remaining / segment_length;
        result.push(start.lerp(end, local_t));
        return dedup_points(result);
    }

    dedup_points(points.to_vec())
}

/// Remove duplicate consecutive points from a polyline
pub(super) fn dedup_points(points: Vec<Vec3>) -> Vec<Vec3> {
    let mut result = Vec::with_capacity(points.len());
    for point in points {
        if result
            .last()
            .map(|prev: &Vec3| prev.distance_squared(point) <= 1e-6)
            .unwrap_or(false)
        {
            continue;
        }
        result.push(point);
    }
    result
}

/// Calculate the next level value for grid-based routing
pub(super) fn next_level_val(levels: &[f32], current: f32, delta: i32, jump_dist: f32) -> f32 {
    let pos = levels.iter().position(|&v| (v - current).abs() < 1e-4);
    match pos {
        Some(idx) => {
            let next_idx = idx as i32 + delta;
            if next_idx >= 0 && next_idx < levels.len() as i32 {
                levels[next_idx as usize]
            } else if next_idx < 0 {
                levels[0] + jump_dist * next_idx as f32
            } else {
                levels.last().unwrap() + jump_dist * (next_idx - (levels.len() as i32 - 1)) as f32
            }
        }
        None => current + jump_dist * delta as f32,
    }
}
