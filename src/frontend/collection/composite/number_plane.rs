use glam::{Vec3, Vec4};

use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};

#[derive(Debug, Clone)]
pub struct NumberPlane {
    pub x_range: (f32, f32),
    pub y_range: (f32, f32),
    pub x_step: f32,
    pub y_step: f32,
    pub grid_thickness: f32,
    pub axis_thickness: f32,
    pub grid_color: Vec4,
    pub axis_color: Vec4,
}

impl NumberPlane {
    pub fn new(x_range: (f32, f32), y_range: (f32, f32)) -> Self {
        Self {
            x_range,
            y_range,
            x_step: 1.0,
            y_step: 1.0,
            grid_thickness: 0.01,
            axis_thickness: 0.03,
            grid_color: Vec4::new(0.35, 0.39, 0.46, 1.0),
            axis_color: Vec4::new(0.78, 0.82, 0.88, 1.0),
        }
    }

    pub fn with_step(mut self, step: f32) -> Self {
        self.x_step = step;
        self.y_step = step;
        self
    }
}

impl Project for NumberPlane {
    fn project(&self, ctx: &mut ProjectionCtx) {
        if self.x_step > 0.0 {
            let mut x = self.x_range.0;
            while x <= self.x_range.1 {
                let is_axis = x.abs() <= f32::EPSILON;
                ctx.emit(RenderPrimitive::Line {
                    start: Vec3::new(x, self.y_range.0, 0.0),
                    end: Vec3::new(x, self.y_range.1, 0.0),
                    thickness: if is_axis {
                        self.axis_thickness
                    } else {
                        self.grid_thickness
                    },
                    color: if is_axis {
                        self.axis_color
                    } else {
                        self.grid_color
                    },
                    dash_length: 0.0,
                    gap_length: 0.0,
                    dash_offset: 0.0,
                });
                x += self.x_step;
            }
        }

        if self.y_step > 0.0 {
            let mut y = self.y_range.0;
            while y <= self.y_range.1 {
                let is_axis = y.abs() <= f32::EPSILON;
                ctx.emit(RenderPrimitive::Line {
                    start: Vec3::new(self.x_range.0, y, 0.0),
                    end: Vec3::new(self.x_range.1, y, 0.0),
                    thickness: if is_axis {
                        self.axis_thickness
                    } else {
                        self.grid_thickness
                    },
                    color: if is_axis {
                        self.axis_color
                    } else {
                        self.grid_color
                    },
                    dash_length: 0.0,
                    gap_length: 0.0,
                    dash_offset: 0.0,
                });
                y += self.y_step;
            }
        }
    }
}

impl Bounded for NumberPlane {
    fn local_bounds(&self) -> Bounds {
        Bounds::new(
            glam::vec2(self.x_range.0, self.y_range.0),
            glam::vec2(self.x_range.1, self.y_range.1),
        )
    }
}
