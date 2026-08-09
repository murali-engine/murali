use crate::engine::scene::Scene;
use crate::frontend::TattvaId;
use crate::frontend::collection::primitives::rounded_rectangle::RoundedRectangle;
use crate::frontend::collection::text::label::Label;
use glam::{Vec3, Vec4};

#[derive(Debug, Clone, Copy)]
pub struct CardIds {
    pub background: TattvaId,
    pub label: TattvaId,
}

impl CardIds {
    pub fn all(self) -> [TattvaId; 2] {
        [self.background, self.label]
    }
}

#[derive(Debug, Clone)]
pub struct Card {
    pub text: String,
    pub width: f32,
    pub height: f32,
    pub radius: f32,
    pub fill: Vec4,
    pub stroke_width: f32,
    pub stroke_color: Vec4,
    pub text_height: f32,
    pub text_color: Vec4,
    pub text_z_offset: f32,
}

impl Card {
    pub fn new(text: impl Into<String>, width: f32, height: f32) -> Self {
        Self {
            text: text.into(),
            width,
            height,
            radius: 0.12,
            fill: Vec4::new(0.10, 0.12, 0.15, 0.45),
            stroke_width: 0.025,
            stroke_color: Vec4::new(0.72, 0.84, 0.96, 0.82),
            text_height: 0.17,
            text_color: Vec4::new(1.0, 1.0, 1.0, 0.92),
            text_z_offset: 0.08,
        }
    }

    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn with_fill(mut self, fill: Vec4) -> Self {
        self.fill = fill;
        self
    }

    pub fn with_stroke(mut self, width: f32, color: Vec4) -> Self {
        self.stroke_width = width;
        self.stroke_color = color;
        self
    }

    pub fn with_text_style(mut self, height: f32, color: Vec4) -> Self {
        self.text_height = height;
        self.text_color = color;
        self
    }

    pub fn with_text_z_offset(mut self, z_offset: f32) -> Self {
        self.text_z_offset = z_offset;
        self
    }

    pub fn add_to_scene(self, scene: &mut Scene, position: Vec3) -> CardIds {
        let background = scene.add_tattva(
            RoundedRectangle::new(self.width, self.height, self.radius, self.fill)
                .with_stroke(self.stroke_width, self.stroke_color),
            position,
        );
        let label = scene.add_tattva(
            Label::new(self.text, self.text_height).with_color(self.text_color),
            Vec3::new(position.x, position.y, position.z + self.text_z_offset),
        );

        CardIds { background, label }
    }
}
