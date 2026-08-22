// src/frontend/sangrah/ganit/graph/vector_field.rs
//! Vector field visualization similar to Manim's VectorField

use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};
use glam::{Vec2, Vec3, Vec4};
use std::sync::Arc;

/// A vector field that displays arrows at grid points
/// The arrows represent a vector function evaluated at each point
pub struct VectorField {
    /// Range of x values
    pub x_range: (f32, f32),
    /// Range of y values
    pub y_range: (f32, f32),
    /// Number of grid points in x direction
    pub x_steps: usize,
    /// Number of grid points in y direction
    pub y_steps: usize,
    /// Function that returns a vector for a given position
    pub field_fn: Arc<dyn Fn(Vec2) -> Vec2 + Send + Sync>,
    /// Base color for vectors
    pub color: Vec4,
    /// Optional function to color vectors based on magnitude
    pub color_fn: Option<Arc<dyn Fn(f32) -> Vec4 + Send + Sync>>,
    /// Scale factor for vector lengths
    pub length_scale: f32,
    /// Minimum vector length to display
    pub min_length: f32,
    /// Maximum vector length to display
    pub max_length: f32,
    /// Arrow shaft thickness
    pub shaft_thickness: f32,
    /// Arrow tip length
    pub tip_length: f32,
    /// Arrow tip width
    pub tip_width: f32,
}

impl VectorField {
    /// Create a new vector field
    pub fn new<F>(
        x_range: (f32, f32),
        y_range: (f32, f32),
        x_steps: usize,
        y_steps: usize,
        field_fn: F,
    ) -> Self
    where
        F: Fn(Vec2) -> Vec2 + Send + Sync + 'static,
    {
        Self {
            x_range,
            y_range,
            x_steps,
            y_steps,
            field_fn: Arc::new(field_fn),
            color: Vec4::new(0.5, 0.7, 1.0, 0.8),
            color_fn: None,
            length_scale: 0.5,
            min_length: 0.01,
            max_length: 2.0,
            shaft_thickness: 0.03,
            tip_length: 0.1,
            tip_width: 0.08,
        }
    }

    /// Set the base color
    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }

    /// Set a function to color vectors based on magnitude
    pub fn with_color_fn<F>(mut self, color_fn: F) -> Self
    where
        F: Fn(f32) -> Vec4 + Send + Sync + 'static,
    {
        self.color_fn = Some(Arc::new(color_fn));
        self
    }

    /// Set the length scale factor
    pub fn with_length_scale(mut self, scale: f32) -> Self {
        self.length_scale = scale;
        self
    }

    /// Set min and max vector lengths
    pub fn with_length_limits(mut self, min: f32, max: f32) -> Self {
        self.min_length = min;
        self.max_length = max;
        self
    }

    /// Set arrow dimensions
    pub fn with_arrow_style(
        mut self,
        shaft_thickness: f32,
        tip_length: f32,
        tip_width: f32,
    ) -> Self {
        self.shaft_thickness = shaft_thickness;
        self.tip_length = tip_length;
        self.tip_width = tip_width;
        self
    }

    /// Calculate grid spacing
    fn grid_spacing(&self) -> (f32, f32) {
        let dx = (self.x_range.1 - self.x_range.0) / (self.x_steps - 1) as f32;
        let dy = (self.y_range.1 - self.y_range.0) / (self.y_steps - 1) as f32;
        (dx, dy)
    }
}

impl Bounded for VectorField {
    fn local_bounds(&self) -> Bounds {
        Bounds::new(
            Vec2::new(self.x_range.0, self.y_range.0),
            Vec2::new(self.x_range.1, self.y_range.1),
        )
    }
}

impl Project for VectorField {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let (dx, dy) = self.grid_spacing();

        for i in 0..self.x_steps {
            for j in 0..self.y_steps {
                let x = self.x_range.0 + i as f32 * dx;
                let y = self.y_range.0 + j as f32 * dy;
                let pos = Vec2::new(x, y);

                // Evaluate the vector field at this point
                let vector = (self.field_fn)(pos);
                let magnitude = vector.length();

                // Skip if too small
                if magnitude < self.min_length {
                    continue;
                }

                // Scale and clamp the vector
                let scaled_length = (magnitude * self.length_scale).min(self.max_length);
                let direction = if magnitude > 0.0 {
                    vector.normalize()
                } else {
                    Vec2::X
                };
                let scaled_vector = direction * scaled_length;

                // Determine color
                let color = if let Some(ref color_fn) = self.color_fn {
                    color_fn(magnitude)
                } else {
                    self.color
                };

                // Draw arrow from pos to pos + scaled_vector
                let start = pos;
                let end = pos + scaled_vector;

                // Draw arrow shaft
                let dir = (end - start).normalize_or_zero();
                let perp = Vec2::new(-dir.y, dir.x);
                let shaft_end = end - dir * self.tip_length;

                ctx.emit(RenderPrimitive::Line {
                    start: Vec3::new(start.x, start.y, 0.0),
                    end: Vec3::new(shaft_end.x, shaft_end.y, 0.0),
                    thickness: self.shaft_thickness,
                    color,
                    dash_length: 0.0,
                    gap_length: 0.0,
                    dash_offset: 0.0,
                });

                // Draw arrow tip
                let tip = end;
                let base_center = end - dir * self.tip_length;
                let base_left = base_center - perp * (self.tip_width * 0.5);
                let base_right = base_center + perp * (self.tip_width * 0.5);

                ctx.emit(RenderPrimitive::Line {
                    start: Vec3::new(tip.x, tip.y, 0.0),
                    end: Vec3::new(base_left.x, base_left.y, 0.0),
                    thickness: self.shaft_thickness,
                    color,
                    dash_length: 0.0,
                    gap_length: 0.0,
                    dash_offset: 0.0,
                });

                ctx.emit(RenderPrimitive::Line {
                    start: Vec3::new(tip.x, tip.y, 0.0),
                    end: Vec3::new(base_right.x, base_right.y, 0.0),
                    thickness: self.shaft_thickness,
                    color,
                    dash_length: 0.0,
                    gap_length: 0.0,
                    dash_offset: 0.0,
                });

                ctx.emit(RenderPrimitive::Line {
                    start: Vec3::new(base_left.x, base_left.y, 0.0),
                    end: Vec3::new(base_right.x, base_right.y, 0.0),
                    thickness: self.shaft_thickness,
                    color,
                    dash_length: 0.0,
                    gap_length: 0.0,
                    dash_offset: 0.0,
                });
            }
        }
    }
}
