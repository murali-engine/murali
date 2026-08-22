use glam::{Vec2, Vec4, vec2, vec3};

use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Mesh, Project, ProjectionCtx, RenderPrimitive};

use super::EPSILON;
use super::vector::VectorArrow2D;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Experimental API: this type is part of the evolving linear algebra visual toolkit.
pub enum MeterMode {
    DotProduct,
    CosineSimilarity,
    DotAndCosine,
}

#[derive(Debug, Clone)]
/// Experimental API: this type is part of the evolving linear algebra visual toolkit.
pub struct DotProductMeter {
    pub a: Vec2,
    pub b: Vec2,
    pub mode: MeterMode,
    pub width: f32,
    pub height: f32,
    pub text_height: f32,
    pub positive_color: Vec4,
    pub negative_color: Vec4,
    pub neutral_color: Vec4,
    pub label_color: Vec4,
}

impl DotProductMeter {
    pub fn new(a: Vec2, b: Vec2) -> Self {
        Self {
            a,
            b,
            mode: MeterMode::DotAndCosine,
            width: 2.5,
            height: 0.18,
            text_height: 0.18,
            positive_color: Vec4::new(0.42, 0.82, 0.48, 1.0),
            negative_color: Vec4::new(0.95, 0.36, 0.34, 1.0),
            neutral_color: Vec4::new(0.44, 0.49, 0.55, 1.0),
            label_color: Vec4::ONE,
        }
    }

    pub fn with_mode(mut self, mode: MeterMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn dot(&self) -> f32 {
        self.a.dot(self.b)
    }

    pub fn cosine(&self) -> f32 {
        let denom = self.a.length() * self.b.length();
        if denom <= EPSILON {
            0.0
        } else {
            (self.dot() / denom).clamp(-1.0, 1.0)
        }
    }

    fn label(&self) -> String {
        match self.mode {
            MeterMode::DotProduct => format!("dot = {:.2}", self.dot()),
            MeterMode::CosineSimilarity => format!("cos = {:.2}", self.cosine()),
            MeterMode::DotAndCosine => {
                format!("dot = {:.2}   cos = {:.2}", self.dot(), self.cosine())
            }
        }
    }
}

impl Project for DotProductMeter {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let cosine = self.cosine();
        let color = if cosine > 0.02 {
            self.positive_color
        } else if cosine < -0.02 {
            self.negative_color
        } else {
            self.neutral_color
        };

        ctx.emit(RenderPrimitive::Mesh(Mesh::rectangle(
            self.width,
            self.height,
            Vec4::new(0.18, 0.21, 0.27, 1.0),
        )));

        let fill_width = self.width * cosine.abs();
        if fill_width > EPSILON {
            let center_x = if cosine >= 0.0 {
                fill_width * 0.5
            } else {
                -fill_width * 0.5
            };
            ctx.emit(RenderPrimitive::Mesh(
                Mesh::rectangle(fill_width, self.height * 0.78, color)
                    .as_ref()
                    .translated(vec3(center_x, 0.0, 0.01)),
            ));
        }

        VectorArrow2D::emit_line(
            ctx,
            vec2(0.0, -self.height * 0.8),
            vec2(0.0, self.height * 0.8),
            0.018,
            self.neutral_color,
        );

        ctx.emit(RenderPrimitive::Text {
            content: self.label(),
            height: self.text_height,
            color: self.label_color,
            font_name: None,
            offset: vec3(0.0, self.height * 1.7, 0.0),
            rotation: 0.0,
        });
    }
}

impl Bounded for DotProductMeter {
    fn local_bounds(&self) -> Bounds {
        Bounds::from_center_size(
            Vec2::ZERO,
            vec2(self.width, self.height + self.text_height * 3.0),
        )
    }
}
