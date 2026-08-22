use glam::{Vec2, Vec4, vec2};

use crate::frontend::layout::{Bounded, Bounds};
use crate::frontend::sangrah::primitives::rounded_rectangle::RoundedRectangle;
use crate::frontend::sangrah::text::label::Label;
use crate::projection::{Project, ProjectionCtx};
use crate::resource::text::layout::measure_label;

#[derive(Debug, Clone)]
/// Experimental API: this type is part of the evolving linear algebra visual toolkit.
pub struct QuantityBadge {
    pub label: String,
    pub value: String,
    pub text_height: f32,
    pub padding: Vec2,
    pub min_width: f32,
    pub min_height: f32,
    pub fill_color: Vec4,
    pub stroke_color: Vec4,
    pub text_color: Vec4,
    pub stroke_thickness: f32,
    pub radius: f32,
}

impl QuantityBadge {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            text_height: 0.16,
            padding: vec2(0.18, 0.09),
            min_width: 0.82,
            min_height: 0.34,
            fill_color: Vec4::new(0.10, 0.12, 0.15, 0.58),
            stroke_color: Vec4::new(0.72, 0.84, 0.96, 0.78),
            text_color: Vec4::new(1.0, 1.0, 1.0, 0.94),
            stroke_thickness: 0.018,
            radius: 0.08,
        }
    }

    pub fn text(&self) -> String {
        if self.label.is_empty() {
            self.value.clone()
        } else {
            format!("{}: {}", self.label, self.value)
        }
    }

    pub fn resolved_size(&self) -> Vec2 {
        let layout = measure_label(&self.text(), self.text_height, None);
        vec2(
            (layout.width + self.padding.x * 2.0).max(self.min_width),
            (layout.height + self.padding.y * 2.0).max(self.min_height),
        )
    }

    pub fn with_text_height(mut self, text_height: f32) -> Self {
        self.text_height = text_height.max(0.01);
        self
    }

    pub fn with_padding(mut self, padding: Vec2) -> Self {
        self.padding = padding.max(Vec2::ZERO);
        self
    }

    pub fn with_min_size(mut self, width: f32, height: f32) -> Self {
        self.min_width = width.max(0.0);
        self.min_height = height.max(0.0);
        self
    }

    pub fn with_fill(mut self, color: Vec4) -> Self {
        self.fill_color = color;
        self
    }

    pub fn with_stroke(mut self, thickness: f32, color: Vec4) -> Self {
        self.stroke_thickness = thickness.max(0.0);
        self.stroke_color = color;
        self
    }

    pub fn with_text_color(mut self, color: Vec4) -> Self {
        self.text_color = color;
        self
    }
}

impl Project for QuantityBadge {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let size = self.resolved_size();
        RoundedRectangle::new(size.x, size.y, self.radius, self.fill_color)
            .with_stroke(self.stroke_thickness, self.stroke_color)
            .project(ctx);
        Label::new(self.text(), self.text_height)
            .with_color(self.text_color)
            .project(ctx);
    }
}

impl Bounded for QuantityBadge {
    fn local_bounds(&self) -> Bounds {
        Bounds::from_center_size(Vec2::ZERO, self.resolved_size())
    }
}

#[derive(Debug, Clone)]
/// Experimental API: this type is part of the evolving linear algebra visual toolkit.
pub struct DimensionBadge {
    pub label: String,
    pub rows: usize,
    pub columns: usize,
    pub badge: QuantityBadge,
}

impl DimensionBadge {
    pub fn new(label: impl Into<String>, rows: usize, columns: usize) -> Self {
        let label = label.into();
        Self {
            badge: QuantityBadge::new(label.clone(), format!("{rows}x{columns}")),
            label,
            rows,
            columns,
        }
    }

    pub fn vector(label: impl Into<String>, length: usize) -> Self {
        Self::new(label, length, 1)
    }

    pub fn value(&self) -> String {
        format!("{}x{}", self.rows, self.columns)
    }

    pub fn as_quantity_badge(&self) -> QuantityBadge {
        let mut badge = self.badge.clone();
        badge.label = self.label.clone();
        badge.value = self.value();
        badge
    }

    pub fn with_text_height(mut self, text_height: f32) -> Self {
        self.badge = self.badge.with_text_height(text_height);
        self
    }

    pub fn with_fill(mut self, color: Vec4) -> Self {
        self.badge = self.badge.with_fill(color);
        self
    }

    pub fn with_stroke(mut self, thickness: f32, color: Vec4) -> Self {
        self.badge = self.badge.with_stroke(thickness, color);
        self
    }

    pub fn with_text_color(mut self, color: Vec4) -> Self {
        self.badge = self.badge.with_text_color(color);
        self
    }
}

impl Project for DimensionBadge {
    fn project(&self, ctx: &mut ProjectionCtx) {
        self.as_quantity_badge().project(ctx);
    }
}

impl Bounded for DimensionBadge {
    fn local_bounds(&self) -> Bounds {
        self.as_quantity_badge().local_bounds()
    }
}
