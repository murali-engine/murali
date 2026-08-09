// src/resources/text/font.rs

use anyhow::{Context, Result};
use fontdue::Font;
use std::path::Path;

pub const LABEL_FONT_RASTER_PX: f32 = 64.0;

/// Font metrics normalized to font units.
#[derive(Debug, Clone, Copy)]
pub struct FontMetrics {
    /// Canonical raster size used for both atlas generation and layout metrics.
    pub raster_px: f32,

    /// Distance from baseline to top of capital letters
    pub cap_height: f32,

    /// Full ascent (baseline → highest point)
    pub ascent: f32,

    /// Full descent (baseline → lowest point, positive value)
    pub descent: f32,

    /// Line height recommended by the font
    pub line_height: f32,
}

/// Loaded font + cached metrics.
///
/// Phase 2 constraints:
/// - Single font
/// - No shaping
/// - Metrics only
pub struct LabelFont {
    font: Font,
    metrics: FontMetrics,
}

impl LabelFont {
    /// Load the embedded label font.
    pub fn load() -> Self {
        Self::load_default().expect("Failed to load embedded label font")
    }

    pub fn load_default() -> Result<Self> {
        let font_bytes = include_bytes!("../assets/fonts/Inter-Regular.ttf");
        Self::load_from_bytes(font_bytes as &[u8]).context("Failed to load embedded label font")
    }

    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let font_bytes = std::fs::read(path)
            .with_context(|| format!("Failed to read font file at {}", path.display()))?;
        Self::load_from_bytes(&font_bytes)
            .with_context(|| format!("Failed to parse font file at {}", path.display()))
    }

    pub fn load_from_bytes(font_bytes: &[u8]) -> Result<Self> {
        let font = Font::from_bytes(font_bytes as &[u8], fontdue::FontSettings::default())
            .map_err(|error| anyhow::anyhow!("fontdue failed to load font: {error}"))?;

        let metrics = Self::compute_metrics(&font);

        Ok(Self { font, metrics })
    }

    /// Access font metrics.
    pub fn metrics(&self) -> FontMetrics {
        self.metrics
    }

    /// Internal: compute normalized font metrics.
    fn compute_metrics(font: &Font) -> FontMetrics {
        // Use the same canonical raster size everywhere in the regular text pipeline.
        let px = LABEL_FONT_RASTER_PX;

        let m = font.metrics('H', px);

        // `height` is the actual rasterized pixel height.
        let cap_height = m.height as f32;
        let ascent = cap_height + m.ymin.max(0) as f32;
        let descent = (-m.ymin).max(0) as f32;

        FontMetrics {
            raster_px: px,
            cap_height,
            ascent,
            descent,
            line_height: ascent + descent,
        }
    }

    /// Access underlying font (used later for glyph rasterization).
    pub(crate) fn font(&self) -> &Font {
        &self.font
    }
}
