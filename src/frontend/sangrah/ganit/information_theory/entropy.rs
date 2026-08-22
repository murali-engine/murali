use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};
use glam::{Vec3, Vec4, vec2};

#[derive(Debug, Clone)]
pub struct EntropyMeter {
    pub bits: f32,
    pub max_bits: f32,
    pub width: f32,
    pub height: f32,
    pub track_color: Vec4,
    pub fill_color: Vec4,
    pub label_color: Vec4,
    pub label: String,
    pub show_label: bool,
}

impl EntropyMeter {
    pub fn new(bits: f32, max_bits: f32) -> Self {
        Self {
            bits,
            max_bits: max_bits.max(0.0),
            width: 2.8,
            height: 0.16,
            track_color: Vec4::new(1.0, 1.0, 1.0, 0.25),
            fill_color: Vec4::ONE,
            label_color: Vec4::ONE,
            label: "entropy".to_string(),
            show_label: true,
        }
    }

    pub fn from_probabilities(probabilities: &[f32]) -> Self {
        let bits = probabilities
            .iter()
            .copied()
            .filter(|p| *p > 0.0)
            .map(|p| -p * p.log2())
            .sum();
        let max_bits = if probabilities.len() <= 1 {
            0.0
        } else {
            (probabilities.len() as f32).log2()
        };

        Self::new(bits, max_bits)
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn with_colors(mut self, track_color: Vec4, fill_color: Vec4, label_color: Vec4) -> Self {
        self.track_color = track_color;
        self.fill_color = fill_color;
        self.label_color = label_color;
        self
    }

    pub fn without_label(mut self) -> Self {
        self.show_label = false;
        self
    }

    pub fn ratio(&self) -> f32 {
        if self.max_bits <= 0.0 {
            0.0
        } else {
            (self.bits / self.max_bits).clamp(0.0, 1.0)
        }
    }
}

impl Project for EntropyMeter {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let left = -self.width * 0.5;
        let right = self.width * 0.5;
        let fill_right = left + self.width * self.ratio();

        ctx.emit(RenderPrimitive::Line {
            start: Vec3::new(left, 0.0, 0.0),
            end: Vec3::new(right, 0.0, 0.0),
            thickness: self.height,
            color: self.track_color,
            dash_length: 0.0,
            gap_length: 0.0,
            dash_offset: 0.0,
        });
        ctx.emit(RenderPrimitive::Line {
            start: Vec3::new(left, 0.0, 0.01),
            end: Vec3::new(fill_right, 0.0, 0.01),
            thickness: self.height,
            color: self.fill_color,
            dash_length: 0.0,
            gap_length: 0.0,
            dash_offset: 0.0,
        });

        if self.show_label {
            ctx.emit(RenderPrimitive::Text {
                content: format!("{}: {:.2} bits", self.label, self.bits),
                height: 0.16,
                color: self.label_color,
                font_name: None,
                offset: Vec3::new(0.0, self.height * 1.4, 0.0),
                rotation: 0.0,
            });
        }
    }
}

impl Bounded for EntropyMeter {
    fn local_bounds(&self) -> Bounds {
        Bounds::new(
            vec2(-self.width * 0.5, -self.height * 0.5),
            vec2(self.width * 0.5, self.height * 2.0),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entropy_meter_computes_entropy_from_probabilities() {
        let meter = EntropyMeter::from_probabilities(&[0.5, 0.5]);

        assert!((meter.bits - 1.0).abs() < 1.0e-5);
        assert!((meter.ratio() - 1.0).abs() < 1.0e-5);
    }
}
