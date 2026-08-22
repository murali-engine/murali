use glam::{Vec2, Vec4, vec2, vec3};

use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};
use crate::resource::text::layout::measure_label;

use super::EPSILON;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Experimental API: this type is part of the evolving linear algebra visual toolkit.
pub enum VectorLabelAnchor {
    Tip,
    Midpoint,
    ShaftSide,
}

#[derive(Debug, Clone)]
/// Experimental API: this type is part of the evolving linear algebra visual toolkit.
pub struct VectorArrow2D {
    pub start: Vec2,
    pub end: Vec2,
    pub shaft_thickness: f32,
    pub tip_length: f32,
    pub tip_width: f32,
    pub color: Vec4,
    pub opacity: f32,
}

impl VectorArrow2D {
    pub fn new(start: Vec2, end: Vec2) -> Self {
        Self {
            start,
            end,
            shaft_thickness: 0.045,
            tip_length: 0.18,
            tip_width: 0.16,
            color: Vec4::new(0.34, 0.78, 0.95, 1.0),
            opacity: 1.0,
        }
    }

    pub fn from_origin(vector: Vec2) -> Self {
        Self::new(Vec2::ZERO, vector)
    }

    pub fn vector(&self) -> Vec2 {
        self.end - self.start
    }

    pub fn length(&self) -> f32 {
        self.vector().length()
    }

    pub fn direction(&self) -> Vec2 {
        self.vector().normalize_or_zero()
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }

    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    pub fn with_thickness(mut self, thickness: f32) -> Self {
        self.shaft_thickness = thickness.max(0.0);
        self
    }

    pub fn with_tip(mut self, length: f32, width: f32) -> Self {
        self.tip_length = length.max(0.0);
        self.tip_width = width.max(0.0);
        self
    }

    fn color_with_opacity(&self) -> Vec4 {
        Vec4::new(
            self.color.x,
            self.color.y,
            self.color.z,
            self.color.w * self.opacity,
        )
    }

    pub(super) fn emit_line(
        ctx: &mut ProjectionCtx,
        start: Vec2,
        end: Vec2,
        thickness: f32,
        color: Vec4,
    ) {
        ctx.emit(RenderPrimitive::Line {
            start: vec3(start.x, start.y, 0.0),
            end: vec3(end.x, end.y, 0.0),
            thickness,
            color,
            dash_length: 0.0,
            gap_length: 0.0,
            dash_offset: 0.0,
        });
    }
}

impl Project for VectorArrow2D {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let vector = self.vector();
        let length = vector.length();
        if length <= EPSILON {
            return;
        }

        let color = self.color_with_opacity();
        let dir = vector / length;
        let perp = vec2(-dir.y, dir.x);
        let tip_length = self.tip_length.min(length * 0.45);
        let shaft_end = self.end - dir * tip_length;

        Self::emit_line(ctx, self.start, shaft_end, self.shaft_thickness, color);

        let base_left = shaft_end - perp * self.tip_width * 0.5;
        let base_right = shaft_end + perp * self.tip_width * 0.5;
        Self::emit_line(ctx, self.end, base_left, self.shaft_thickness, color);
        Self::emit_line(ctx, self.end, base_right, self.shaft_thickness, color);
        Self::emit_line(ctx, base_left, base_right, self.shaft_thickness, color);
    }
}

impl Bounded for VectorArrow2D {
    fn local_bounds(&self) -> Bounds {
        let pad = self.tip_width.max(self.shaft_thickness) * 0.75;
        Bounds::new(
            self.start.min(self.end) - Vec2::splat(pad),
            self.start.max(self.end) + Vec2::splat(pad),
        )
    }
}

#[derive(Debug, Clone)]
/// Experimental API: this type is part of the evolving linear algebra visual toolkit.
pub struct LabeledVector2D {
    pub arrow: VectorArrow2D,
    pub label: String,
    pub label_height: f32,
    pub label_color: Vec4,
    pub anchor: VectorLabelAnchor,
    pub label_offset: Vec2,
    pub show_coordinates: bool,
}

impl LabeledVector2D {
    pub fn new(label: impl Into<String>, vector: Vec2) -> Self {
        Self {
            arrow: VectorArrow2D::from_origin(vector),
            label: label.into(),
            label_height: 0.22,
            label_color: Vec4::ONE,
            anchor: VectorLabelAnchor::Tip,
            label_offset: vec2(0.16, 0.16),
            show_coordinates: false,
        }
    }

    pub fn from_arrow(label: impl Into<String>, arrow: VectorArrow2D) -> Self {
        Self {
            arrow,
            ..Self::new(label, Vec2::X)
        }
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.arrow.color = color;
        self
    }

    pub fn with_thickness(mut self, thickness: f32) -> Self {
        self.arrow.shaft_thickness = thickness.max(0.0);
        self
    }

    pub fn with_label_color(mut self, color: Vec4) -> Self {
        self.label_color = color;
        self
    }

    pub fn with_anchor(mut self, anchor: VectorLabelAnchor) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn with_label_offset(mut self, offset: Vec2) -> Self {
        self.label_offset = offset;
        self
    }

    pub fn with_coordinates(mut self, show: bool) -> Self {
        self.show_coordinates = show;
        self
    }

    fn label_position(&self) -> Vec2 {
        let base = match self.anchor {
            VectorLabelAnchor::Tip => self.arrow.end,
            VectorLabelAnchor::Midpoint => (self.arrow.start + self.arrow.end) * 0.5,
            VectorLabelAnchor::ShaftSide => {
                let dir = self.arrow.direction();
                let perp = vec2(-dir.y, dir.x);
                (self.arrow.start + self.arrow.end) * 0.5 + perp * 0.22
            }
        };
        base + self.label_offset
    }

    fn label_text(&self) -> String {
        if self.show_coordinates {
            let v = self.arrow.vector();
            format!("{} ({:.2}, {:.2})", self.label, v.x, v.y)
        } else {
            self.label.clone()
        }
    }
}

impl Project for LabeledVector2D {
    fn project(&self, ctx: &mut ProjectionCtx) {
        self.arrow.project(ctx);
        let pos = self.label_position();
        ctx.emit(RenderPrimitive::Text {
            content: self.label_text(),
            height: self.label_height,
            color: self.label_color,
            font_name: None,
            offset: vec3(pos.x, pos.y, 0.0),
            rotation: 0.0,
        });
    }
}

impl Bounded for LabeledVector2D {
    fn local_bounds(&self) -> Bounds {
        let text = self.label_text();
        let pos = self.label_position();
        let layout = measure_label(&text, self.label_height, None);
        self.arrow.local_bounds().union(&Bounds::from_center_size(
            pos,
            vec2(layout.width, layout.height),
        ))
    }
}
