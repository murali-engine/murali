use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};
use glam::{Vec3, Vec4};

/// Frontend Axes.
/// A composite Tattva that projects into multiple lines and (eventually) labels.
pub struct Axes {
    pub x_range: (f32, f32),
    pub y_range: (f32, f32),
    pub x_step: f32,
    pub y_step: f32,
    pub thickness: f32,
    pub color: Vec4,
    pub tick_size: f32,
    pub show_ticks: bool,
}

impl Axes {
    pub fn new(x_range: (f32, f32), y_range: (f32, f32)) -> Self {
        Self {
            x_range,
            y_range,
            x_step: 1.0,
            y_step: 1.0,
            thickness: 0.02,
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            tick_size: 0.1,
            show_ticks: true,
        }
    }

    pub fn with_step(mut self, step: f32) -> Self {
        self.x_step = step;
        self.y_step = step;
        self
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }

    pub fn with_thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness;
        self
    }

    pub fn with_tick_size(mut self, tick_size: f32) -> Self {
        self.tick_size = tick_size;
        self
    }

    pub fn without_ticks(mut self) -> Self {
        self.show_ticks = false;
        self
    }
}

impl Project for Axes {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let color_rgb = self.color;

        // 1. Project X-Axis Main Line
        ctx.emit(RenderPrimitive::Line {
            start: Vec3::new(self.x_range.0, 0.0, 0.0),
            end: Vec3::new(self.x_range.1, 0.0, 0.0),
            thickness: self.thickness,
            color: color_rgb,
            dash_length: 0.0,
            gap_length: 0.0,
            dash_offset: 0.0,
        });

        // 2. Project Y-Axis Main Line
        ctx.emit(RenderPrimitive::Line {
            start: Vec3::new(0.0, self.y_range.0, 0.0),
            end: Vec3::new(0.0, self.y_range.1, 0.0),
            thickness: self.thickness,
            color: color_rgb,
            dash_length: 0.0,
            gap_length: 0.0,
            dash_offset: 0.0,
        });

        // 3. Project X Ticks
        if self.show_ticks && self.x_step > 0.0 {
            let mut x = self.x_range.0;
            while x <= self.x_range.1 {
                if x.abs() > 0.001 {
                    ctx.emit(RenderPrimitive::Line {
                        start: Vec3::new(x, -self.tick_size * 0.5, 0.0),
                        end: Vec3::new(x, self.tick_size * 0.5, 0.0),
                        thickness: self.thickness * 0.6,
                        color: color_rgb,
                        dash_length: 0.0,
                        gap_length: 0.0,
                        dash_offset: 0.0,
                    });
                }
                x += self.x_step;
            }
        }

        // 4. Project Y Ticks
        if self.show_ticks && self.y_step > 0.0 {
            let mut y = self.y_range.0;
            while y <= self.y_range.1 {
                if y.abs() > 0.001 {
                    ctx.emit(RenderPrimitive::Line {
                        start: Vec3::new(-self.tick_size * 0.5, y, 0.0),
                        end: Vec3::new(self.tick_size * 0.5, y, 0.0),
                        thickness: self.thickness * 0.6,
                        color: color_rgb,
                        dash_length: 0.0,
                        gap_length: 0.0,
                        dash_offset: 0.0,
                    });
                }
                y += self.y_step;
            }
        }
    }
}

impl Bounded for Axes {
    fn local_bounds(&self) -> Bounds {
        Bounds::new(
            glam::vec2(self.x_range.0, self.y_range.0),
            glam::vec2(self.x_range.1, self.y_range.1),
        )
    }
}
