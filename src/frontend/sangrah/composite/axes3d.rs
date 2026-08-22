use glam::{Vec3, Vec4, vec2};

use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};

#[derive(Debug, Clone)]
pub struct Axes3D {
    pub x_range: (f32, f32),
    pub y_range: (f32, f32),
    pub z_range: (f32, f32),
    pub x_step: f32,
    pub y_step: f32,
    pub z_step: f32,
    pub axis_thickness: f32,
    pub tick_thickness: f32,
    pub tick_size: f32,
    pub x_color: Vec4,
    pub y_color: Vec4,
    pub z_color: Vec4,
    pub show_ticks: bool,
}

impl Axes3D {
    pub fn new(x_range: (f32, f32), y_range: (f32, f32), z_range: (f32, f32)) -> Self {
        Self {
            x_range,
            y_range,
            z_range,
            x_step: 1.0,
            y_step: 1.0,
            z_step: 1.0,
            axis_thickness: 0.03,
            tick_thickness: 0.018,
            tick_size: 0.16,
            x_color: Vec4::new(0.96, 0.42, 0.34, 1.0),
            y_color: Vec4::new(0.34, 0.78, 0.95, 1.0),
            z_color: Vec4::new(0.95, 0.82, 0.34, 1.0),
            show_ticks: true,
        }
    }

    pub fn with_step(mut self, step: f32) -> Self {
        self.x_step = step;
        self.y_step = step;
        self.z_step = step;
        self
    }

    pub fn with_tick_size(mut self, tick_size: f32) -> Self {
        self.tick_size = tick_size;
        self
    }

    pub fn with_axis_thickness(mut self, thickness: f32) -> Self {
        self.axis_thickness = thickness;
        self
    }

    pub fn without_ticks(mut self) -> Self {
        self.show_ticks = false;
        self
    }

    fn emit_line(ctx: &mut ProjectionCtx, start: Vec3, end: Vec3, thickness: f32, color: Vec4) {
        ctx.emit(RenderPrimitive::Line {
            start,
            end,
            thickness,
            color,
            dash_length: 0.0,
            gap_length: 0.0,
            dash_offset: 0.0,
        });
    }
}

impl Project for Axes3D {
    fn project(&self, ctx: &mut ProjectionCtx) {
        Self::emit_line(
            ctx,
            Vec3::new(self.x_range.0, 0.0, 0.0),
            Vec3::new(self.x_range.1, 0.0, 0.0),
            self.axis_thickness,
            self.x_color,
        );
        Self::emit_line(
            ctx,
            Vec3::new(0.0, self.y_range.0, 0.0),
            Vec3::new(0.0, self.y_range.1, 0.0),
            self.axis_thickness,
            self.y_color,
        );
        Self::emit_line(
            ctx,
            Vec3::new(0.0, 0.0, self.z_range.0),
            Vec3::new(0.0, 0.0, self.z_range.1),
            self.axis_thickness,
            self.z_color,
        );

        if !self.show_ticks {
            return;
        }

        if self.x_step > 0.0 {
            let mut x = self.x_range.0;
            while x <= self.x_range.1 {
                if x.abs() > 0.001 {
                    Self::emit_line(
                        ctx,
                        Vec3::new(x, -self.tick_size * 0.5, 0.0),
                        Vec3::new(x, self.tick_size * 0.5, 0.0),
                        self.tick_thickness,
                        self.x_color,
                    );
                    Self::emit_line(
                        ctx,
                        Vec3::new(x, 0.0, -self.tick_size * 0.35),
                        Vec3::new(x, 0.0, self.tick_size * 0.35),
                        self.tick_thickness * 0.8,
                        self.x_color,
                    );
                }
                x += self.x_step;
            }
        }

        if self.y_step > 0.0 {
            let mut y = self.y_range.0;
            while y <= self.y_range.1 {
                if y.abs() > 0.001 {
                    Self::emit_line(
                        ctx,
                        Vec3::new(-self.tick_size * 0.5, y, 0.0),
                        Vec3::new(self.tick_size * 0.5, y, 0.0),
                        self.tick_thickness,
                        self.y_color,
                    );
                    Self::emit_line(
                        ctx,
                        Vec3::new(0.0, y, -self.tick_size * 0.35),
                        Vec3::new(0.0, y, self.tick_size * 0.35),
                        self.tick_thickness * 0.8,
                        self.y_color,
                    );
                }
                y += self.y_step;
            }
        }

        if self.z_step > 0.0 {
            let mut z = self.z_range.0;
            while z <= self.z_range.1 {
                if z.abs() > 0.001 {
                    Self::emit_line(
                        ctx,
                        Vec3::new(-self.tick_size * 0.45, 0.0, z),
                        Vec3::new(self.tick_size * 0.45, 0.0, z),
                        self.tick_thickness,
                        self.z_color,
                    );
                    Self::emit_line(
                        ctx,
                        Vec3::new(0.0, -self.tick_size * 0.45, z),
                        Vec3::new(0.0, self.tick_size * 0.45, z),
                        self.tick_thickness,
                        self.z_color,
                    );
                }
                z += self.z_step;
            }
        }
    }
}

impl Bounded for Axes3D {
    fn local_bounds(&self) -> Bounds {
        Bounds::new(
            vec2(
                self.x_range.0.min(self.z_range.0),
                self.y_range.0.min(self.z_range.0),
            ),
            vec2(
                self.x_range.1.max(self.z_range.1),
                self.y_range.1.max(self.z_range.1),
            ),
        )
    }
}
