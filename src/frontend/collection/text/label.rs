use crate::frontend::animation::indicate::Indicate;
use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};
use crate::resource::text::layout::measure_label;
use glam::Vec4;

/// Frontend Label.
/// Lightweight text intended for UI, ticks, or annotations.
///
/// This represents the *intent* to show text. The actual glyph generation
/// happens in the Projection/Backend boundary using the Resource Layer.
#[derive(Debug, Clone)]
pub struct Label {
    pub text: String,
    pub world_height: f32,
    pub color: Vec4,
    pub font_name: Option<String>,
    /// Character reveal progress: 0.0 = no characters, 1.0 = all characters
    pub char_reveal: f32,
    /// Reveal mode: true = typewriter (fixed position), false = reveal (shifting)
    pub typewriter_mode: bool,
    /// Indication event progress: 0.0 = inactive, 1.0 = event complete
    pub indicate_t: f32,
}

impl Label {
    /// Creates a new Label with specified text and world-space height.
    pub fn new<S: Into<String>>(text: S, world_height: f32) -> Self {
        Self {
            text: text.into(),
            world_height,
            color: Vec4::new(1.0, 1.0, 1.0, 1.0), // Default to white
            font_name: None,
            char_reveal: 1.0,
            typewriter_mode: false, // Default to reveal mode
            indicate_t: 0.0,
        }
    }

    /// Builder-style method to set color.
    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }

    /// Builder-style method to select a registered font by name.
    pub fn with_font<S: Into<String>>(mut self, font_name: S) -> Self {
        self.font_name = Some(font_name.into());
        self
    }

    /// Builder-style method to set character reveal progress.
    pub fn with_char_reveal(mut self, char_reveal: f32) -> Self {
        self.char_reveal = char_reveal.clamp(0.0, 1.0);
        self
    }

    /// Get the revealed text based on char_reveal progress
    fn get_revealed_text(&self) -> String {
        let char_count = self.text.chars().count();
        let reveal_count = (char_count as f32 * self.char_reveal.clamp(0.0, 1.0)).ceil() as usize;
        self.text.chars().take(reveal_count).collect()
    }

    fn indicate_intensity(t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        let pulse = 1.0 - (2.0 * t - 1.0).abs();
        pulse * pulse * (3.0 - 2.0 * pulse)
    }

    fn indicated_color(&self, intensity: f32) -> Vec4 {
        let lift = 0.28 * intensity.clamp(0.0, 1.0);
        let target = Vec4::new(1.0, 1.0, 1.0, self.color.w);
        self.color.lerp(target, lift)
    }

    fn emit_revealed_text(&self, ctx: &mut ProjectionCtx, color: Vec4) {
        let revealed_text = self.get_revealed_text();

        if self.typewriter_mode {
            let full_layout =
                measure_label(&self.text, self.world_height, self.font_name.as_deref());
            let revealed_layout =
                measure_label(&revealed_text, self.world_height, self.font_name.as_deref());
            let offset_x = (revealed_layout.width - full_layout.width) / 2.0;

            ctx.emit(RenderPrimitive::Text {
                content: revealed_text,
                height: self.world_height,
                color,
                font_name: self.font_name.clone(),
                offset: glam::Vec3::new(offset_x, 0.0, 0.0),
                rotation: 0.0,
            });
        } else {
            ctx.emit(RenderPrimitive::Text {
                content: revealed_text,
                height: self.world_height,
                color,
                font_name: self.font_name.clone(),
                offset: glam::Vec3::ZERO,
                rotation: 0.0,
            });
        }
    }
}

impl Project for Label {
    fn project(&self, ctx: &mut ProjectionCtx) {
        if self.indicate_t > 0.0 {
            self.project_indicated(ctx, self.indicate_t);
        } else {
            self.emit_revealed_text(ctx, self.color);
        }
    }
}

impl Indicate for Label {
    fn project_indicated(&self, ctx: &mut ProjectionCtx, t: f32) {
        let intensity = Self::indicate_intensity(t);
        let scale = 1.0 + 0.12 * intensity;
        let color = self.indicated_color(intensity);
        ctx.with_scale(scale, |ctx| self.emit_revealed_text(ctx, color));
    }
}

impl Bounded for Label {
    fn local_bounds(&self) -> Bounds {
        let layout = measure_label(&self.text, self.world_height, self.font_name.as_deref());
        Bounds::from_center_size(
            glam::Vec2::ZERO,
            glam::vec2(layout.width.max(self.world_height * 0.4), layout.height),
        )
    }
}
