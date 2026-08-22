use crate::engine::scene::Scene;
use crate::frontend::TattvaId;
use crate::frontend::sangrah::primitives::chat_bubble::{ChatBubble, ChatBubbleTipSide};
use crate::frontend::sangrah::primitives::rounded_rectangle::RoundedRectangle;
use crate::frontend::sangrah::text::label::Label;
use crate::resource::text::layout::measure_label;
use glam::{Vec2, Vec3, Vec4, vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatInputTipSide {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub struct ChatInputBoxIds {
    pub bubble: TattvaId,
    pub text: TattvaId,
    pub send_button: Option<TattvaId>,
}

impl ChatInputBoxIds {
    pub fn all(self) -> Vec<TattvaId> {
        let mut ids = vec![self.bubble, self.text];
        if let Some(send_button) = self.send_button {
            ids.push(send_button);
        }
        ids
    }
}

#[derive(Debug, Clone)]
pub struct ChatInputBox {
    pub text: String,
    pub width: f32,
    pub height: f32,
    pub radius: f32,
    pub fill: Vec4,
    pub stroke_width: f32,
    pub stroke_color: Vec4,
    pub text_height: f32,
    pub text_color: Vec4,
    pub text_inset: Vec2,
    pub text_z_offset: f32,
    pub tip_side: ChatInputTipSide,
    pub tip_width: f32,
    pub tip_height: f32,
    pub tip_inset: f32,
    pub show_send_button: bool,
    pub send_button_color: Vec4,
    pub send_button_radius: f32,
    pub send_button_size: f32,
}

impl ChatInputBox {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            width: 5.8,
            height: 0.82,
            radius: 0.18,
            fill: Vec4::new(0.08, 0.10, 0.13, 0.92),
            stroke_width: 0.018,
            stroke_color: Vec4::new(0.64, 0.76, 0.88, 0.46),
            text_height: 0.22,
            text_color: Vec4::new(0.94, 0.97, 1.0, 0.96),
            text_inset: vec2(0.38, 0.0),
            text_z_offset: 0.08,
            tip_side: ChatInputTipSide::Left,
            tip_width: 0.42,
            tip_height: 0.28,
            tip_inset: 0.52,
            show_send_button: false,
            send_button_color: Vec4::new(0.28, 0.60, 0.90, 0.92),
            send_button_radius: 0.16,
            send_button_size: 0.34,
        }
    }

    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
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

    pub fn with_text_inset(mut self, inset: Vec2) -> Self {
        self.text_inset = inset;
        self
    }

    pub fn with_tip_side(mut self, side: ChatInputTipSide) -> Self {
        self.tip_side = side;
        self
    }

    pub fn with_tip(mut self, side: ChatInputTipSide, width: f32, height: f32) -> Self {
        self.tip_side = side;
        self.tip_width = width;
        self.tip_height = height;
        self
    }

    pub fn with_tip_inset(mut self, inset: f32) -> Self {
        self.tip_inset = inset;
        self
    }

    pub fn with_send_button(mut self, show: bool) -> Self {
        self.show_send_button = show;
        self
    }

    pub fn with_send_button_style(mut self, size: f32, radius: f32, color: Vec4) -> Self {
        self.send_button_size = size;
        self.send_button_radius = radius;
        self.send_button_color = color;
        self
    }

    pub fn add_to_scene(self, scene: &mut Scene, position: Vec3) -> ChatInputBoxIds {
        let bubble = scene.add_tattva(
            ChatBubble::new(self.width, self.height, self.radius, self.fill)
                .with_tip(
                    match self.tip_side {
                        ChatInputTipSide::Left => ChatBubbleTipSide::Left,
                        ChatInputTipSide::Right => ChatBubbleTipSide::Right,
                    },
                    self.tip_width,
                    self.tip_height,
                )
                .with_tip_inset(self.tip_inset)
                .with_stroke(self.stroke_width, self.stroke_color),
            position,
        );

        let text_layout = measure_label(&self.text, self.text_height, None);
        let text_x = position.x - self.width * 0.5 + self.text_inset.x + text_layout.width * 0.5;
        let text_y = position.y + self.text_inset.y;
        let mut label = Label::new(self.text, self.text_height)
            .with_color(self.text_color)
            .with_char_reveal(0.0);
        label.typewriter_mode = true;
        let text = scene.add_tattva(
            label,
            Vec3::new(text_x, text_y, position.z + self.text_z_offset),
        );

        let send_button = self.show_send_button.then(|| {
            let x = position.x + self.width * 0.5 - self.text_inset.x - self.send_button_size * 0.5;
            scene.add_tattva(
                RoundedRectangle::new(
                    self.send_button_size,
                    self.send_button_size,
                    self.send_button_radius,
                    self.send_button_color,
                ),
                Vec3::new(x, position.y, position.z + 0.06),
            )
        });

        ChatInputBoxIds {
            bubble,
            text,
            send_button,
        }
    }
}
