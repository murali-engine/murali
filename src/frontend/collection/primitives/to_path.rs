use crate::frontend::collection::primitives::circle::Circle;
use crate::frontend::collection::primitives::ellipse::Ellipse;
use crate::frontend::collection::primitives::line::Line;
use crate::frontend::collection::primitives::path::{Path, PathSegment};
use crate::frontend::collection::primitives::polygon::Polygon;
use crate::frontend::collection::primitives::rectangle::Rectangle;
use crate::frontend::collection::primitives::rounded_rectangle::RoundedRectangle;
use crate::frontend::collection::primitives::square::Square;
use glam::{Vec2, vec2};

pub trait ToPath {
    fn to_path(&self) -> Path;
}

impl ToPath for Rectangle {
    fn to_path(&self) -> Path {
        let w = self.width / 2.0;
        let h = self.height / 2.0;
        // Start at 3 o'clock (w, 0) and traverse CCW with 8 segments for alignment
        Path::new()
            .move_to(vec2(w, 0.0))
            .line_to(vec2(w, h))
            .line_to(vec2(0.0, h))
            .line_to(vec2(-w, h))
            .line_to(vec2(-w, 0.0))
            .line_to(vec2(-w, -h))
            .line_to(vec2(0.0, -h))
            .line_to(vec2(w, -h))
            .close() // closes back to (w,0)
            .with_style(self.style.clone())
    }
}

impl ToPath for RoundedRectangle {
    fn to_path(&self) -> Path {
        let outline =
            rounded_rect_outline(self.width, self.height, self.radius, self.corner_segments);
        let Some(first) = outline.first().copied() else {
            return Path::new();
        };

        let mut path = Path::new().move_to(first);
        for point in outline.into_iter().skip(1) {
            path = path.line_to(point);
        }

        path.close().with_style(self.style.clone())
    }
}

impl ToPath for Square {
    fn to_path(&self) -> Path {
        let r = self.size / 2.0;
        // Start at 3 o'clock (r, 0) and traverse CCW with 8 segments
        Path::new()
            .move_to(vec2(r, 0.0))
            .line_to(vec2(r, r))
            .line_to(vec2(0.0, r))
            .line_to(vec2(-r, r))
            .line_to(vec2(-r, 0.0))
            .line_to(vec2(-r, -r))
            .line_to(vec2(0.0, -r))
            .line_to(vec2(r, -r))
            .close()
            .with_style(self.style.clone())
    }
}

impl ToPath for Circle {
    fn to_path(&self) -> Path {
        let r = self.radius;
        // Standard Cubic Bezier circle split into 8 segments (2 per quadrant) for better alignment
        let k = 4.0 / 3.0 * (std::f32::consts::PI / 16.0).tan() * r;

        let mut path = Path::new().move_to(vec2(r, 0.0));
        let segments = 8;
        for i in 0..segments {
            let a1 = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let a2 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;

            // Re-calculate control points for the 1/8 segment
            let mid_a = (a1 + a2) / 2.0;
            let h = 4.0 / 3.0 * ((a2 - a1) / 4.0).tan() * r;

            let p1 = vec2(a1.cos(), a1.sin()) * r;
            let p2 = vec2(a2.cos(), a2.sin()) * r;
            let c1 = p1 + vec2(-a1.sin(), a1.cos()) * h;
            let c2 = p2 - vec2(-a2.sin(), a2.cos()) * h;

            path = path.cubic_to(c1, c2, p2);
        }

        path.close().with_style(self.style.clone())
    }
}

impl ToPath for Ellipse {
    fn to_path(&self) -> Path {
        let rx = self.radius_x;
        let ry = self.radius_y;

        let mut path = Path::new().move_to(vec2(rx, 0.0));
        let segments = 8;
        for i in 0..segments {
            let a1 = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let a2 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;

            let h = 4.0 / 3.0 * ((a2 - a1) / 4.0).tan();

            let p1 = vec2(a1.cos() * rx, a1.sin() * ry);
            let p2 = vec2(a2.cos() * rx, a2.sin() * ry);
            let c1 = p1 + vec2(-a1.sin() * rx, a1.cos() * ry) * h;
            let c2 = p2 - vec2(-a2.sin() * rx, a2.cos() * ry) * h;

            path = path.cubic_to(c1, c2, p2);
        }

        path.close().with_style(self.style.clone())
    }
}

impl ToPath for Polygon {
    fn to_path(&self) -> Path {
        if self.vertices.is_empty() {
            return Path::new();
        }
        // Polygons are user-defined, but we ensure they have at least 8 segments for morphing
        let mut path = Path::new().move_to(self.vertices[0]);
        for &p in &self.vertices[1..] {
            path = path.line_to(p);
        }
        let mut path = path.close().with_style(self.style.clone());
        path.resample(8);
        path
    }
}

impl ToPath for Line {
    fn to_path(&self) -> Path {
        Path::new()
            .move_to(self.start.truncate())
            .line_to(self.end.truncate())
            .with_style(self.style.clone())
    }
}

fn rounded_rect_outline(width: f32, height: f32, radius: f32, corner_segments: usize) -> Vec<Vec2> {
    let half = vec2(width.abs() * 0.5, height.abs() * 0.5);
    let r = radius.abs().max(0.01).min(half.x.min(half.y));
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
        for step in 0..=corner_segments.max(1) {
            let t = step as f32 / corner_segments.max(1) as f32;
            let angle = start + (end - start) * t;
            points.push(center + vec2(angle.cos() * r, angle.sin() * r));
        }
    }
    points
}
