use glam::{Vec2, Vec4, vec2, vec3};

use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};

use super::EPSILON;
use super::vector::VectorArrow2D;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Experimental API: this type is part of the evolving linear algebra visual toolkit.
pub enum AngleUnit {
    Radians,
    Degrees,
}

#[derive(Debug, Clone)]
/// Experimental API: this type is part of the evolving linear algebra visual toolkit.
pub struct AngleArc {
    pub from: Vec2,
    pub to: Vec2,
    pub radius: f32,
    pub color: Vec4,
    pub thickness: f32,
    pub segments: usize,
    pub label: Option<String>,
    pub label_height: f32,
    pub unit: AngleUnit,
}

impl AngleArc {
    pub fn between(from: Vec2, to: Vec2) -> Self {
        Self {
            from,
            to,
            radius: 0.7,
            color: Vec4::new(0.95, 0.82, 0.34, 1.0),
            thickness: 0.025,
            segments: 32,
            label: None,
            label_height: 0.18,
            unit: AngleUnit::Degrees,
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_auto_label(mut self, unit: AngleUnit) -> Self {
        self.unit = unit;
        self.label = Some(self.formatted_angle());
        self
    }

    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = radius.max(0.0);
        self
    }

    pub fn signed_angle(&self) -> f32 {
        self.from.angle_to(self.to)
    }

    pub fn angle(&self) -> f32 {
        self.signed_angle().abs()
    }

    fn formatted_angle(&self) -> String {
        match self.unit {
            AngleUnit::Radians => format!("{:.2} rad", self.angle()),
            AngleUnit::Degrees => format!("{:.0} deg", self.angle().to_degrees()),
        }
    }
}

impl Project for AngleArc {
    fn project(&self, ctx: &mut ProjectionCtx) {
        if self.from.length() <= EPSILON || self.to.length() <= EPSILON || self.radius <= EPSILON {
            return;
        }

        let start = self.from.y.atan2(self.from.x);
        let sweep = self.signed_angle();
        let steps = self.segments.max(4);
        let mut previous = None;
        for idx in 0..=steps {
            let t = idx as f32 / steps as f32;
            let angle = start + sweep * t;
            let point = vec2(angle.cos(), angle.sin()) * self.radius;
            if let Some(prev) = previous {
                VectorArrow2D::emit_line(ctx, prev, point, self.thickness, self.color);
            }
            previous = Some(point);
        }

        if let Some(label) = &self.label {
            let mid = start + sweep * 0.5;
            let pos = vec2(mid.cos(), mid.sin()) * (self.radius + self.label_height * 1.4);
            ctx.emit(RenderPrimitive::Text {
                content: label.clone(),
                height: self.label_height,
                color: self.color,
                font_name: None,
                offset: vec3(pos.x, pos.y, 0.0),
                rotation: 0.0,
            });
        }
    }
}

impl Bounded for AngleArc {
    fn local_bounds(&self) -> Bounds {
        let r = self.radius + self.label_height * 2.0;
        Bounds::from_center_size(Vec2::ZERO, Vec2::splat(r * 2.0))
    }
}

#[derive(Debug, Clone)]
/// Experimental API: this type is part of the evolving linear algebra visual toolkit.
pub struct OrthogonalityMarker {
    pub first: Vec2,
    pub second: Vec2,
    pub vertex: Vec2,
    pub size: f32,
    pub color: Vec4,
    pub thickness: f32,
    pub label: Option<String>,
    pub label_height: f32,
}

impl OrthogonalityMarker {
    pub fn new(first: Vec2, second: Vec2) -> Self {
        Self {
            first,
            second,
            vertex: Vec2::ZERO,
            size: 0.28,
            color: Vec4::new(0.95, 0.82, 0.34, 1.0),
            thickness: 0.025,
            label: None,
            label_height: 0.14,
        }
    }

    pub fn with_vertex(mut self, vertex: Vec2) -> Self {
        self.vertex = vertex;
        self
    }

    pub fn with_size(mut self, size: f32) -> Self {
        self.size = size.max(0.0);
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn is_orthogonal(&self) -> bool {
        self.first.dot(self.second).abs() <= EPSILON
    }

    pub fn corner_points(&self) -> Option<[Vec2; 3]> {
        let first = self.first.normalize_or_zero();
        let second = self.second.normalize_or_zero();
        if first.length_squared() <= EPSILON
            || second.length_squared() <= EPSILON
            || self.size <= EPSILON
        {
            return None;
        }

        let first_point = self.vertex + first * self.size;
        let second_point = self.vertex + second * self.size;
        Some([first_point, first_point + second * self.size, second_point])
    }
}

impl Project for OrthogonalityMarker {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let Some([first_point, corner, second_point]) = self.corner_points() else {
            return;
        };

        VectorArrow2D::emit_line(ctx, first_point, corner, self.thickness, self.color);
        VectorArrow2D::emit_line(ctx, corner, second_point, self.thickness, self.color);

        if let Some(label) = &self.label {
            let label_pos = corner + (corner - self.vertex).normalize_or_zero() * self.label_height;
            ctx.emit(RenderPrimitive::Text {
                content: label.clone(),
                height: self.label_height,
                color: self.color,
                font_name: None,
                offset: vec3(label_pos.x, label_pos.y, 0.0),
                rotation: 0.0,
            });
        }
    }
}

impl Bounded for OrthogonalityMarker {
    fn local_bounds(&self) -> Bounds {
        if let Some([first_point, corner, second_point]) = self.corner_points() {
            let min = self.vertex.min(first_point).min(corner).min(second_point);
            let max = self.vertex.max(first_point).max(corner).max(second_point);
            Bounds::new(
                min - Vec2::splat(self.label_height),
                max + Vec2::splat(self.label_height),
            )
        } else {
            Bounds::from_center_size(self.vertex, Vec2::ZERO)
        }
    }
}
