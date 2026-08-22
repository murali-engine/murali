//! An opinionated, configurable 3D title opening.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use glam::{EulerRot, Quat, Vec3, Vec4, vec3};
use thiserror::Error;

use crate::engine::scene::Scene;
use crate::engine::timeline::Timeline;
use crate::frontend::TattvaId;
use crate::frontend::animation::Ease;
use crate::frontend::props::DepthMode;
use crate::frontend::sangrah::text::label::Label;
use crate::frontend::sangrah::text::letter3d::{Letter3D, Letter3DError, LetterParticles3D};
use crate::resource::texture::TextureImage;

#[derive(Debug, Error)]
pub enum OpeningError {
    #[error("opening title must contain at least one capital letter")]
    EmptyTitle,
    #[error("opening titles support only ASCII capitals and spaces, got {character:?} at {index}")]
    UnsupportedCharacter { index: usize, character: char },
    #[error("invalid opening setting `{0}`")]
    InvalidSetting(&'static str),
    #[error("failed to load opening texture at {path}: {source}")]
    Texture {
        path: String,
        #[source]
        source: anyhow::Error,
    },
    #[error(transparent)]
    Letter(#[from] Letter3DError),
}

/// Visual controls for [`Opening`].
#[derive(Debug, Clone)]
pub struct OpeningStyle {
    pub letter_height: f32,
    pub letter_depth: f32,
    pub letter_gap: f32,
    pub space_width: f32,
    pub final_y: f32,
    pub front_color: Vec4,
    pub back_color: Vec4,
    pub side_color: Vec4,
    pub particle_count: usize,
    pub particle_size: f32,
    pub particle_color: Vec4,
    pub particle_palette: Vec<Vec4>,
    pub particle_distance: f32,
    pub particle_rise: f32,
    pub particle_curl: f32,
    pub tagline_height: f32,
    pub tagline_color: Vec4,
    pub tagline_font_name: Option<String>,
}

impl Default for OpeningStyle {
    fn default() -> Self {
        Self {
            letter_height: 2.4,
            letter_depth: 0.95,
            letter_gap: 0.34,
            space_width: 0.85,
            final_y: -0.42,
            front_color: Vec4::new(1.0, 0.98, 0.91, 1.0),
            back_color: Vec4::new(0.58, 0.55, 0.49, 1.0),
            side_color: Vec4::new(0.42, 0.39, 0.34, 1.0),
            particle_count: 700,
            particle_size: 0.028,
            particle_color: Vec4::new(0.78, 0.79, 0.77, 0.88),
            particle_palette: vec![
                Vec4::new(0.18, 0.82, 0.78, 1.0),
                Vec4::new(0.32, 0.58, 1.0, 1.0),
                Vec4::new(0.92, 0.38, 0.68, 1.0),
                Vec4::new(1.0, 0.72, 0.22, 1.0),
                Vec4::new(0.48, 0.86, 0.38, 1.0),
                Vec4::new(1.0, 0.4, 0.3, 1.0),
            ],
            particle_distance: 4.8,
            particle_rise: 2.35,
            particle_curl: 1.0,
            tagline_height: 0.48,
            tagline_color: Vec4::new(0.95, 0.97, 0.99, 1.0),
            tagline_font_name: None,
        }
    }
}

/// Local-time choreography for [`Opening`].
#[derive(Debug, Clone, Copy)]
pub struct OpeningTiming {
    pub intro_delay: f32,
    pub landing_stagger: f32,
    pub landing_duration: f32,
    pub bounce_up_duration: f32,
    pub bounce_down_duration: f32,
    pub settled_hold: f32,
    pub shake_duration: f32,
    pub shake_beats: usize,
    pub particle_scatter_duration: f32,
    pub tagline_reveal_delay: f32,
    pub dissolve_duration: f32,
    pub end_hold: f32,
}

impl Default for OpeningTiming {
    fn default() -> Self {
        Self {
            intro_delay: 0.4,
            landing_stagger: 0.3,
            landing_duration: 1.38,
            bounce_up_duration: 0.18,
            bounce_down_duration: 0.24,
            settled_hold: 0.55,
            shake_duration: 0.97,
            shake_beats: 11,
            particle_scatter_duration: 1.5,
            tagline_reveal_delay: 0.72,
            dissolve_duration: 0.82,
            end_hold: 0.59,
        }
    }
}

/// An experimental composite that drops a 3D capital title, shakes it apart,
/// and dissolves the resulting particles to reveal a tagline.
#[derive(Debug, Clone)]
pub struct Opening {
    pub title: String,
    pub tagline: String,
    pub style: OpeningStyle,
    pub timing: OpeningTiming,
    font_path: Option<PathBuf>,
    texture: Option<Arc<TextureImage>>,
    texture_path: Option<PathBuf>,
}

impl Opening {
    pub fn new(title: impl Into<String>, tagline: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            tagline: tagline.into(),
            style: OpeningStyle::default(),
            timing: OpeningTiming::default(),
            font_path: None,
            texture: None,
            texture_path: None,
        }
    }

    pub fn with_font_path(mut self, path: impl AsRef<Path>) -> Self {
        self.font_path = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn with_texture_path(mut self, path: impl AsRef<Path>) -> Self {
        self.texture = None;
        self.texture_path = Some(path.as_ref().to_path_buf());
        self
    }

    pub fn with_texture(mut self, texture: TextureImage) -> Self {
        self.texture = Some(Arc::new(texture));
        self.texture_path = None;
        self
    }

    pub fn with_style(mut self, style: OpeningStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_timing(mut self, timing: OpeningTiming) -> Self {
        self.timing = timing;
        self
    }

    pub fn add_to_scene(self, scene: &mut Scene, origin: Vec3) -> Result<OpeningIds, OpeningError> {
        self.validate()?;

        let texture = self.load_texture()?;
        let mut slots = Vec::with_capacity(self.title.chars().count());
        for character in self.title.chars() {
            if character == ' ' {
                slots.push(OpeningSlot::Space(self.style.space_width));
                continue;
            }

            let mut letter = self.make_letter(character)?.with_face_colors(
                self.style.front_color,
                self.style.back_color,
                self.style.side_color,
            );
            if let Some(texture) = &texture {
                letter = letter.with_shared_texture(texture.clone());
            }
            slots.push(OpeningSlot::Letter(character, letter));
        }

        let total_width = slots.iter().map(OpeningSlot::width).sum::<f32>()
            + self.style.letter_gap * slots.len().saturating_sub(1) as f32;
        let mut cursor = -total_width * 0.5;
        let mut letters = Vec::new();
        for slot in slots {
            let width = slot.width();
            if let OpeningSlot::Letter(character, letter) = slot {
                let final_position = origin + vec3(cursor + width * 0.5, self.style.final_y, 0.0);
                let index = letters.len();
                let local_x = final_position.x - origin.x;
                let start_position = origin
                    + vec3(
                        local_x * 0.35,
                        4.6 + index as f32 * 0.18,
                        13.2 + index as f32 * 0.55,
                    );
                let solid = scene.add_tattva(letter, start_position);
                scene.set_depth_mode(solid, DepthMode::World);
                scene.set_rotation(
                    solid,
                    Quat::from_euler(
                        EulerRot::XYZ,
                        0.75 + index as f32 * 0.17,
                        -0.65 + index as f32 * 0.23,
                        0.35 - index as f32 * 0.11,
                    ),
                );

                let particles = self
                    .make_particles(character)?
                    .with_motion(
                        self.style.particle_distance + index as f32 * 0.16,
                        self.style.particle_rise,
                        self.style.particle_curl,
                    )
                    .with_particle_size(self.style.particle_size)
                    .with_color(self.style.particle_color)
                    .with_palette(self.style.particle_palette.iter().copied())
                    .with_seed(19.0 + index as f32 * 7.3);
                let particle_id = scene.add_tattva(particles, final_position);
                scene.set_depth_mode(particle_id, DepthMode::World);
                scene.hide(particle_id);

                letters.push(OpeningLetterIds {
                    solid,
                    particles: particle_id,
                    final_position,
                });
            }
            cursor += width + self.style.letter_gap;
        }

        let mut tagline_label = Label::new(self.tagline, self.style.tagline_height)
            .with_color(self.style.tagline_color);
        if let Some(font_name) = &self.style.tagline_font_name {
            tagline_label = tagline_label.with_font(font_name);
        }
        let tagline = scene.add_tattva(tagline_label, origin);
        scene.set_depth_mode(tagline, DepthMode::Overlay);
        scene.hide(tagline);

        Ok(OpeningIds {
            letters,
            tagline,
            timing: self.timing,
            origin,
        })
    }

    fn validate(&self) -> Result<(), OpeningError> {
        let mut letters = 0;
        for (index, character) in self.title.chars().enumerate() {
            if character.is_ascii_uppercase() {
                letters += 1;
            } else if character != ' ' {
                return Err(OpeningError::UnsupportedCharacter { index, character });
            }
        }
        if letters == 0 {
            return Err(OpeningError::EmptyTitle);
        }

        validate_positive("style.letter_height", self.style.letter_height)?;
        validate_positive("style.letter_depth", self.style.letter_depth)?;
        validate_nonnegative("style.letter_gap", self.style.letter_gap)?;
        validate_nonnegative("style.space_width", self.style.space_width)?;
        validate_finite("style.final_y", self.style.final_y)?;
        validate_positive("style.particle_size", self.style.particle_size)?;
        validate_nonnegative("style.particle_distance", self.style.particle_distance)?;
        validate_finite("style.particle_rise", self.style.particle_rise)?;
        validate_nonnegative("style.particle_curl", self.style.particle_curl)?;
        validate_positive("style.tagline_height", self.style.tagline_height)?;
        if self.style.particle_count == 0 {
            return Err(OpeningError::InvalidSetting("style.particle_count"));
        }
        if self.style.particle_palette.is_empty() {
            return Err(OpeningError::InvalidSetting("style.particle_palette"));
        }
        if self.timing.shake_beats == 0 {
            return Err(OpeningError::InvalidSetting("timing.shake_beats"));
        }
        for (field, value) in [
            ("timing.intro_delay", self.timing.intro_delay),
            ("timing.landing_stagger", self.timing.landing_stagger),
            ("timing.landing_duration", self.timing.landing_duration),
            ("timing.bounce_up_duration", self.timing.bounce_up_duration),
            (
                "timing.bounce_down_duration",
                self.timing.bounce_down_duration,
            ),
            ("timing.settled_hold", self.timing.settled_hold),
            ("timing.shake_duration", self.timing.shake_duration),
            (
                "timing.particle_scatter_duration",
                self.timing.particle_scatter_duration,
            ),
            (
                "timing.tagline_reveal_delay",
                self.timing.tagline_reveal_delay,
            ),
            ("timing.dissolve_duration", self.timing.dissolve_duration),
            ("timing.end_hold", self.timing.end_hold),
        ] {
            validate_nonnegative(field, value)?;
        }
        Ok(())
    }

    fn load_texture(&self) -> Result<Option<Arc<TextureImage>>, OpeningError> {
        if let Some(texture) = &self.texture {
            return Ok(Some(texture.clone()));
        }
        self.texture_path
            .as_deref()
            .map(|path| {
                TextureImage::from_path(path)
                    .map(Arc::new)
                    .map_err(|source| OpeningError::Texture {
                        path: path.display().to_string(),
                        source,
                    })
            })
            .transpose()
    }

    fn make_letter(&self, character: char) -> Result<Letter3D, Letter3DError> {
        match self.font_path.as_deref() {
            Some(path) => Letter3D::from_font_path(
                character,
                self.style.letter_height,
                self.style.letter_depth,
                path,
            ),
            None => Letter3D::new(character, self.style.letter_height, self.style.letter_depth),
        }
    }

    fn make_particles(&self, character: char) -> Result<LetterParticles3D, Letter3DError> {
        match self.font_path.as_deref() {
            Some(path) => LetterParticles3D::from_font_path(
                character,
                self.style.letter_height,
                self.style.letter_depth,
                self.style.particle_count,
                path,
            ),
            None => LetterParticles3D::new(
                character,
                self.style.letter_height,
                self.style.letter_depth,
                self.style.particle_count,
            ),
        }
    }
}

enum OpeningSlot {
    Space(f32),
    Letter(char, Letter3D),
}

impl OpeningSlot {
    fn width(&self) -> f32 {
        match self {
            Self::Space(width) => *width,
            Self::Letter(_, letter) => letter.width(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OpeningLetterIds {
    pub solid: TattvaId,
    pub particles: TattvaId,
    final_position: Vec3,
}

#[derive(Debug, Clone)]
pub struct OpeningIds {
    pub letters: Vec<OpeningLetterIds>,
    pub tagline: TattvaId,
    timing: OpeningTiming,
    origin: Vec3,
}

impl OpeningIds {
    pub fn all(&self) -> Vec<TattvaId> {
        let mut ids = Vec::with_capacity(self.letters.len() * 2 + 1);
        for letter in &self.letters {
            ids.extend([letter.solid, letter.particles]);
        }
        ids.push(self.tagline);
        ids
    }

    /// Authors the complete sequence in local time starting at `0.0`.
    /// Pass either a `Timeline` or a `Clip`, which dereferences to `Timeline`.
    pub fn animate(&self, timeline: &mut Timeline) {
        let shake_start = self.shake_start();
        let burst_time = shake_start + self.timing.shake_duration;

        for (index, letter) in self.letters.iter().enumerate() {
            let impact = self.timing.intro_delay + index as f32 * self.timing.landing_stagger;
            timeline
                .animate(letter.solid)
                .at(impact)
                .for_duration(self.timing.landing_duration)
                .ease(Ease::InCubic)
                .move_to(letter.final_position)
                .spawn();
            timeline
                .animate(letter.solid)
                .at(impact)
                .for_duration(self.timing.landing_duration)
                .ease(Ease::InOutCubic)
                .rotate_to(Quat::IDENTITY)
                .spawn();
            timeline
                .animate(letter.solid)
                .at(impact + self.timing.landing_duration)
                .for_duration(self.timing.bounce_up_duration)
                .ease(Ease::OutCubic)
                .move_to(letter.final_position + vec3(0.0, 0.2, 0.0))
                .spawn();
            timeline
                .animate(letter.solid)
                .at(impact + self.timing.landing_duration + self.timing.bounce_up_duration)
                .for_duration(self.timing.bounce_down_duration)
                .ease(Ease::InCubic)
                .move_to(letter.final_position)
                .spawn();

            self.animate_shake(timeline, index, letter, shake_start);
            timeline
                .animate(letter.solid)
                .at(burst_time)
                .for_duration(0.045)
                .ease(Ease::OutCubic)
                .fade_to(0.0)
                .spawn();
        }

        for (index, letter) in self.letters.iter().enumerate() {
            timeline
                .animate(letter.particles)
                .at(burst_time)
                .for_duration(0.035)
                .ease(Ease::OutCubic)
                .appear()
                .spawn();
            timeline
                .animate(letter.particles)
                .at(burst_time)
                .for_duration(self.timing.particle_scatter_duration)
                .ease(Ease::OutQuad)
                .letter_particle_scatter_to(1.0)
                .spawn();

            let local_x = letter.final_position.x - self.origin.x;
            let vertical = [1.25, -0.95, 0.7, -0.55, -0.95, 1.25][index % 6];
            timeline
                .animate(letter.particles)
                .at(burst_time)
                .for_duration(self.timing.particle_scatter_duration * 0.9)
                .ease(Ease::OutQuad)
                .move_to(self.origin + vec3(local_x * 1.45, vertical * 0.72, 2.2))
                .spawn();
            timeline
                .animate(letter.particles)
                .at(burst_time)
                .for_duration(self.timing.particle_scatter_duration * 0.9)
                .ease(Ease::OutQuad)
                .scale_to(Vec3::splat(1.65))
                .spawn();
            timeline
                .animate(letter.particles)
                .at(burst_time + self.timing.tagline_reveal_delay)
                .for_duration(self.timing.dissolve_duration)
                .ease(Ease::InOutCubic)
                .fade_to(0.0)
                .spawn();
        }

        timeline
            .animate(self.tagline)
            .at(burst_time + self.timing.tagline_reveal_delay)
            .for_duration(self.timing.dissolve_duration)
            .ease(Ease::OutCubic)
            .appear()
            .spawn();
        timeline.wait_until(self.duration());
    }

    pub fn duration(&self) -> f32 {
        let burst_time = self.shake_start() + self.timing.shake_duration;
        let particle_end = self
            .timing
            .particle_scatter_duration
            .max(self.timing.tagline_reveal_delay + self.timing.dissolve_duration);
        burst_time + particle_end + self.timing.end_hold
    }

    fn shake_start(&self) -> f32 {
        let last_impact = self.timing.intro_delay
            + self.letters.len().saturating_sub(1) as f32 * self.timing.landing_stagger;
        last_impact
            + self.timing.landing_duration
            + self.timing.bounce_up_duration
            + self.timing.bounce_down_duration
            + self.timing.settled_hold
    }

    fn animate_shake(
        &self,
        timeline: &mut Timeline,
        index: usize,
        letter: &OpeningLetterIds,
        shake_start: f32,
    ) {
        let mut beat_start = shake_start;
        let weight_sum = (0..self.timing.shake_beats)
            .map(|beat| {
                let denominator = self.timing.shake_beats.saturating_sub(1).max(1) as f32;
                let energy = beat as f32 / denominator;
                1.0 - energy * 0.54
            })
            .sum::<f32>();
        for beat in 0..self.timing.shake_beats {
            let denominator = self.timing.shake_beats.saturating_sub(1).max(1) as f32;
            let energy = beat as f32 / denominator;
            let weight = 1.0 - energy * 0.54;
            let duration = self.timing.shake_duration * weight / weight_sum;
            let direction = if (beat + index) % 2 == 0 { 1.0 } else { -1.0 };
            let amplitude = 0.014 + energy.powi(2) * 0.075;
            let position = letter.final_position
                + vec3(
                    direction * amplitude,
                    -direction * amplitude * (0.28 + index as f32 * 0.025),
                    0.0,
                );
            let rotation = Quat::from_euler(
                EulerRot::XYZ,
                direction * energy * 0.012,
                -direction * energy * 0.018,
                direction * (0.006 + energy.powi(2) * 0.045),
            );
            timeline
                .animate(letter.solid)
                .at(beat_start)
                .for_duration(duration)
                .ease(Ease::InOutCubic)
                .move_to(position)
                .spawn();
            timeline
                .animate(letter.solid)
                .at(beat_start)
                .for_duration(duration)
                .ease(Ease::InOutCubic)
                .rotate_to(rotation)
                .spawn();
            beat_start += duration;
        }
    }
}

fn validate_positive(field: &'static str, value: f32) -> Result<(), OpeningError> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(OpeningError::InvalidSetting(field))
    }
}

fn validate_finite(field: &'static str, value: f32) -> Result<(), OpeningError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(OpeningError::InvalidSetting(field))
    }
}

fn validate_nonnegative(field: &'static str, value: f32) -> Result<(), OpeningError> {
    if value.is_finite() && value >= 0.0 {
        Ok(())
    } else {
        Err(OpeningError::InvalidSetting(field))
    }
}

#[cfg(test)]
mod tests {
    use super::{Opening, OpeningError, OpeningStyle};
    use crate::engine::scene::Scene;
    use crate::engine::timeline::Clip;
    use crate::frontend::sangrah::text::label::Label;
    use crate::frontend::sangrah::text::letter3d::Letter3D;
    use crate::frontend::sangrah::text::letter3d::LetterParticles3D;
    use crate::resource::texture::{BuiltinTexture, TextureImage};
    use glam::{Vec3, Vec4};

    fn inexpensive_style() -> OpeningStyle {
        OpeningStyle {
            particle_count: 8,
            ..OpeningStyle::default()
        }
    }

    #[test]
    fn builds_capitals_and_authors_a_local_time_clip() {
        let mut scene = Scene::new();
        let ids = Opening::new("AI", "A useful tagline")
            .with_style(inexpensive_style())
            .add_to_scene(&mut scene, Vec3::ZERO)
            .unwrap();
        let mut clip = Clip::new();
        ids.animate(&mut clip);

        assert_eq!(ids.letters.len(), 2);
        assert_eq!(ids.all().len(), 5);
        assert!((clip.duration() - ids.duration()).abs() < 1e-6);
    }

    #[test]
    fn supports_spaces_but_rejects_other_characters() {
        let mut scene = Scene::new();
        let ids = Opening::new("AI LAB", "Tagline")
            .with_style(inexpensive_style())
            .add_to_scene(&mut scene, Vec3::ZERO)
            .unwrap();
        assert_eq!(ids.letters.len(), 5);

        let error = Opening::new("Ai", "Tagline")
            .with_style(inexpensive_style())
            .add_to_scene(&mut scene, Vec3::ZERO)
            .unwrap_err();
        assert!(matches!(
            error,
            OpeningError::UnsupportedCharacter { character: 'i', .. }
        ));
    }

    #[test]
    fn applies_the_configured_particle_palette() {
        let palette = vec![Vec4::new(0.2, 0.4, 0.8, 1.0)];
        let mut scene = Scene::new();
        let ids = Opening::new("A", "Tagline")
            .with_style(OpeningStyle {
                particle_count: 8,
                particle_palette: palette.clone(),
                ..OpeningStyle::default()
            })
            .add_to_scene(&mut scene, Vec3::ZERO)
            .unwrap();
        let particles = scene
            .get_tattva_typed::<LetterParticles3D>(ids.letters[0].particles)
            .unwrap();

        assert_eq!(particles.state.palette(), palette);
    }

    #[test]
    fn default_six_letter_opening_keeps_the_reference_duration() {
        let mut scene = Scene::new();
        let ids = Opening::new("KAVRIQ", "Tagline")
            .with_style(inexpensive_style())
            .add_to_scene(&mut scene, Vec3::ZERO)
            .unwrap();

        assert!((ids.duration() - 7.35).abs() < 1e-5);
    }

    #[test]
    fn passes_a_registered_font_name_to_the_tagline_label() {
        let mut scene = Scene::new();
        let ids = Opening::new("A", "Tagline")
            .with_style(OpeningStyle {
                particle_count: 8,
                tagline_font_name: Some("Brand Sans".to_owned()),
                ..OpeningStyle::default()
            })
            .add_to_scene(&mut scene, Vec3::ZERO)
            .unwrap();
        let tagline = scene.get_tattva_typed::<Label>(ids.tagline).unwrap();

        assert_eq!(tagline.state.font_name.as_deref(), Some("Brand Sans"));
    }

    #[test]
    fn accepts_an_embedded_texture_without_a_path() {
        let mut scene = Scene::new();
        let ids = Opening::new("A", "Tagline")
            .with_texture(TextureImage::builtin(BuiltinTexture::BlackMarble))
            .with_style(inexpensive_style())
            .add_to_scene(&mut scene, Vec3::ZERO)
            .unwrap();
        let letter = scene
            .get_tattva_typed::<Letter3D>(ids.letters[0].solid)
            .unwrap();

        assert_eq!(letter.state.texture().unwrap().width, 1254);
    }
}
