use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};
use glam::{Vec3, Vec4, vec2};

#[derive(Debug, Clone)]
pub struct NumberLine {
    pub range: (f32, f32),
    pub step: f32,
    pub thickness: f32,
    pub tick_size: f32,
    pub color: Vec4,
    pub origin_color: Vec4,
    pub show_ticks: bool,
    pub emphasize_origin: bool,
}

impl NumberLine {
    pub fn new(range: (f32, f32)) -> Self {
        Self {
            range,
            step: 1.0,
            thickness: 0.02,
            tick_size: 0.14,
            color: Vec4::ONE,
            origin_color: Vec4::ONE,
            show_ticks: true,
            emphasize_origin: true,
        }
    }

    pub fn with_step(mut self, step: f32) -> Self {
        self.step = step;
        self
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }

    pub fn with_origin_color(mut self, color: Vec4) -> Self {
        self.origin_color = color;
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

    pub fn without_origin_emphasis(mut self) -> Self {
        self.emphasize_origin = false;
        self
    }

    fn emits_origin_tick(&self) -> bool {
        self.emphasize_origin && self.range.0 <= 0.0 && self.range.1 >= 0.0
    }
}

impl Project for NumberLine {
    fn project(&self, ctx: &mut ProjectionCtx) {
        ctx.emit(RenderPrimitive::Line {
            start: Vec3::new(self.range.0, 0.0, 0.0),
            end: Vec3::new(self.range.1, 0.0, 0.0),
            thickness: self.thickness,
            color: self.color,
            dash_length: 0.0,
            gap_length: 0.0,
            dash_offset: 0.0,
        });

        if self.show_ticks && self.step > 0.0 {
            let start = (self.range.0 / self.step).ceil() as i32;
            let end = (self.range.1 / self.step).floor() as i32;

            for i in start..=end {
                let x = i as f32 * self.step;
                let is_origin = x.abs() <= 1.0e-5;
                let color = if is_origin && self.emits_origin_tick() {
                    self.origin_color
                } else {
                    self.color
                };
                let tick_size = if is_origin && self.emits_origin_tick() {
                    self.tick_size * 1.3
                } else {
                    self.tick_size
                };

                ctx.emit(RenderPrimitive::Line {
                    start: Vec3::new(x, -tick_size * 0.5, 0.0),
                    end: Vec3::new(x, tick_size * 0.5, 0.0),
                    thickness: self.thickness * 0.75,
                    color,
                    dash_length: 0.0,
                    gap_length: 0.0,
                    dash_offset: 0.0,
                });
            }
        }
    }
}

impl Bounded for NumberLine {
    fn local_bounds(&self) -> Bounds {
        Bounds::new(
            vec2(self.range.0, -self.tick_size * 0.5),
            vec2(self.range.1, self.tick_size * 0.5),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_line_detects_origin_inside_range() {
        assert!(NumberLine::new((-2.0, 3.0)).emits_origin_tick());
        assert!(!NumberLine::new((1.0, 3.0)).emits_origin_tick());
    }
}
