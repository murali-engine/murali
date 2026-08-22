// src/frontend/collection/primitives/arrow.rs
//! Arrow primitive with shaft and triangular tip

use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};
use glam::{Vec2, Vec3, Vec4};

/// An arrow with a shaft (line) and a triangular tip
pub struct Arrow {
    /// Start point of the arrow
    pub start: Vec2,
    /// End point of the arrow (tip of the arrowhead)
    pub end: Vec2,
    /// Thickness of the shaft
    pub shaft_thickness: f32,
    /// Length of the arrowhead
    pub tip_length: f32,
    /// Width of the arrowhead base
    pub tip_width: f32,
    /// Color of the arrow
    pub color: Vec4,
}

impl Arrow {
    /// Create a new arrow
    ///
    /// # Arguments
    /// * `start` - Starting point of the arrow
    /// * `end` - End point (tip of the arrowhead)
    /// * `shaft_thickness` - Thickness of the arrow shaft
    /// * `tip_length` - Length of the triangular tip
    /// * `tip_width` - Width of the triangular tip base
    /// * `color` - Color of the arrow
    pub fn new(
        start: Vec2,
        end: Vec2,
        shaft_thickness: f32,
        tip_length: f32,
        tip_width: f32,
        color: Vec4,
    ) -> Self {
        Self {
            start,
            end,
            shaft_thickness,
            tip_length,
            tip_width,
            color,
        }
    }

    /// Create an arrow with default tip proportions
    /// Tip length is 3x shaft thickness, tip width is 2x shaft thickness
    pub fn with_default_tip(start: Vec2, end: Vec2, shaft_thickness: f32, color: Vec4) -> Self {
        Self::new(
            start,
            end,
            shaft_thickness,
            shaft_thickness * 3.0,
            shaft_thickness * 2.0,
            color,
        )
    }

    /// Create a simple arrow pointing up with unit length
    pub fn simple(color: Vec4) -> Self {
        Self::with_default_tip(Vec2::ZERO, Vec2::new(0.0, 1.0), 0.05, color)
    }

    /// Set the start point
    pub fn with_start(mut self, start: Vec2) -> Self {
        self.start = start;
        self
    }

    /// Set the end point
    pub fn with_end(mut self, end: Vec2) -> Self {
        self.end = end;
        self
    }

    /// Set the color
    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }

    /// Calculate the direction vector from start to end
    fn direction(&self) -> Vec2 {
        (self.end - self.start).normalize_or_zero()
    }

    /// Calculate the perpendicular vector (for arrowhead base)
    fn perpendicular(&self) -> Vec2 {
        let dir = self.direction();
        Vec2::new(-dir.y, dir.x)
    }
}

impl Bounded for Arrow {
    fn local_bounds(&self) -> Bounds {
        let min_x = self.start.x.min(self.end.x);
        let max_x = self.start.x.max(self.end.x);
        let min_y = self.start.y.min(self.end.y);
        let max_y = self.start.y.max(self.end.y);

        // Expand bounds slightly to account for arrowhead width
        let padding = self.tip_width * 0.5;
        Bounds::new(
            Vec2::new(min_x - padding, min_y - padding),
            Vec2::new(max_x + padding, max_y + padding),
        )
    }
}

impl Project for Arrow {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let dir = self.direction();
        let perp = self.perpendicular();

        // Calculate where the shaft ends (before the arrowhead starts)
        let shaft_end = self.end - dir * self.tip_length;

        // Draw the shaft (line from start to where arrowhead begins)
        ctx.emit(RenderPrimitive::Line {
            start: Vec3::new(self.start.x, self.start.y, 0.0),
            end: Vec3::new(shaft_end.x, shaft_end.y, 0.0),
            thickness: self.shaft_thickness,
            color: self.color,
            dash_length: 0.0,
            gap_length: 0.0,
            dash_offset: 0.0,
        });

        // Draw the arrowhead as a filled triangle
        // Three vertices: tip, base-left, base-right
        let tip = self.end;
        let base_center = self.end - dir * self.tip_length;
        let base_left = base_center - perp * (self.tip_width * 0.5);
        let base_right = base_center + perp * (self.tip_width * 0.5);

        // Draw three lines forming the triangle
        // We could also emit a filled triangle mesh, but for now use lines
        ctx.emit(RenderPrimitive::Line {
            start: Vec3::new(tip.x, tip.y, 0.0),
            end: Vec3::new(base_left.x, base_left.y, 0.0),
            thickness: self.shaft_thickness,
            color: self.color,
            dash_length: 0.0,
            gap_length: 0.0,
            dash_offset: 0.0,
        });

        ctx.emit(RenderPrimitive::Line {
            start: Vec3::new(tip.x, tip.y, 0.0),
            end: Vec3::new(base_right.x, base_right.y, 0.0),
            thickness: self.shaft_thickness,
            color: self.color,
            dash_length: 0.0,
            gap_length: 0.0,
            dash_offset: 0.0,
        });

        ctx.emit(RenderPrimitive::Line {
            start: Vec3::new(base_left.x, base_left.y, 0.0),
            end: Vec3::new(base_right.x, base_right.y, 0.0),
            thickness: self.shaft_thickness,
            color: self.color,
            dash_length: 0.0,
            gap_length: 0.0,
            dash_offset: 0.0,
        });
    }
}
