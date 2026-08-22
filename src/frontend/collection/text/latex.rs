use glam::Vec4;

use crate::frontend::animation::indicate::Indicate;
use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};

/// The Frontend LaTeX object.
/// Pure semantic intent. No file IO occurs here.
pub struct Latex {
    pub source: String,
    pub world_height: f32,
    pub color: Vec4,
    /// Character reveal progress: 0.0 = no characters, 1.0 = all characters
    pub char_reveal: f32,
    /// Reveal mode: true = typewriter (fixed position), false = reveal (shifting)
    pub typewriter_mode: bool,
    /// Indication event progress: 0.0 = inactive, 1.0 = event complete
    pub indicate_t: f32,
}

impl Latex {
    /// Creates a new LaTeX Tattva from a raw string.
    /// This is a fast, pure operation.
    pub fn new<S: Into<String>>(source: S, world_height: f32) -> Self {
        Self {
            source: source.into(),
            world_height,
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            char_reveal: 1.0,
            typewriter_mode: false, // Default to reveal mode
            indicate_t: 0.0,
        }
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }

    /// Builder-style method to set character reveal progress.
    pub fn with_char_reveal(mut self, char_reveal: f32) -> Self {
        self.char_reveal = char_reveal.clamp(0.0, 1.0);
        self
    }

    /// Get the revealed text based on char_reveal progress
    fn get_revealed_text(&self) -> String {
        let char_count = self.source.chars().count();
        let reveal_count = (char_count as f32 * self.char_reveal.clamp(0.0, 1.0)).ceil() as usize;
        self.source.chars().take(reveal_count).collect()
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
            let full_width = self.source.chars().count() as f32 * self.world_height * 0.55;
            let revealed_width = revealed_text.chars().count() as f32 * self.world_height * 0.55;
            let offset_x = (revealed_width - full_width) / 2.0;

            ctx.emit(RenderPrimitive::Latex {
                source: revealed_text,
                height: self.world_height,
                color,
                offset: glam::Vec3::new(offset_x, 0.0, 0.0),
            });
        } else {
            ctx.emit(RenderPrimitive::Latex {
                source: revealed_text,
                height: self.world_height,
                color,
                offset: glam::Vec3::ZERO,
            });
        }
    }
}

impl Project for Latex {
    fn project(&self, ctx: &mut ProjectionCtx) {
        if self.indicate_t > 0.0 {
            self.project_indicated(ctx, self.indicate_t);
        } else {
            self.emit_revealed_text(ctx, self.color);
        }
    }
}

impl Indicate for Latex {
    fn project_indicated(&self, ctx: &mut ProjectionCtx, t: f32) {
        let intensity = Self::indicate_intensity(t);
        let scale = 1.0 + 0.12 * intensity;
        let color = self.indicated_color(intensity);
        ctx.with_scale(scale, |ctx| self.emit_revealed_text(ctx, color));
    }
}

impl Bounded for Latex {
    fn local_bounds(&self) -> Bounds {
        let width = self.source.chars().count() as f32 * self.world_height * 0.55;
        Bounds::from_center_size(
            glam::Vec2::ZERO,
            glam::vec2(width.max(self.world_height), self.world_height),
        )
    }
}
