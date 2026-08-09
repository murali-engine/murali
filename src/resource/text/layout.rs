use crate::resource::text::font::{FontMetrics, LabelFont};
use crate::resource::text::manager::font_asset;

/// A positioned glyph in a single-line label.
#[derive(Debug, Clone)]
pub struct GlyphInstance {
    pub ch: char,

    /// X position relative to label origin (baseline)
    pub x: f32,

    /// Advance width in world units.
    pub advance: f32,

    pub width: f32,
    pub height: f32,
    pub bearing_x: f32,
    pub bearing_y: f32,
}

/// Result of laying out a label.
#[derive(Debug, Clone)]
pub struct LabelLayout {
    pub glyphs: Vec<GlyphInstance>,

    /// Total width in world units
    pub width: f32,

    /// Total height in world units
    pub height: f32,

    pub ascent: f32,
    pub descent: f32,
}

/// Compute glyph layout for a single-line label.
///
/// This is CPU-only and renderer-agnostic.
pub fn layout_label(font: &LabelFont, text: &str, world_height: f32) -> LabelLayout {
    let metrics: FontMetrics = font.metrics();

    // Scale font units → world units
    let scale = world_height / metrics.cap_height;

    let mut glyphs = Vec::new();
    let mut cursor_x = 0.0;
    let mut previous_char = None;
    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;

    for ch in text.chars() {
        if let Some(prev) = previous_char {
            if let Some(kern) = font.font().horizontal_kern(prev, ch, metrics.raster_px) {
                cursor_x += kern;
            }
        }

        // Query glyph metrics at the same canonical size used by the atlas.
        let m = font.font().metrics(ch, metrics.raster_px);

        let advance = m.advance_width * scale;
        let glyph_x = cursor_x * scale;
        let width = m.width as f32 * scale;
        let height = m.height as f32 * scale;
        let bearing_x = m.xmin as f32 * scale;
        let bearing_y = m.ymin as f32 * scale;

        min_x = min_x.min(glyph_x + bearing_x);
        max_x = max_x.max(glyph_x + bearing_x + width);

        glyphs.push(GlyphInstance {
            ch,
            x: glyph_x,
            advance,
            width,
            height,
            bearing_x,
            bearing_y,
        });

        cursor_x += m.advance_width;
        previous_char = Some(ch);
    }

    let width = if glyphs.is_empty() {
        0.0
    } else {
        (max_x - min_x).max(0.0)
    };
    let height = (metrics.ascent + metrics.descent) * scale;
    let ascent = metrics.ascent * scale;
    let descent = metrics.descent * scale;

    LabelLayout {
        glyphs,
        width,
        height,
        ascent,
        descent,
    }
}

pub fn measure_label(text: &str, world_height: f32, font_name: Option<&str>) -> LabelLayout {
    match font_asset(font_name) {
        Ok(asset) => layout_label(&asset.font, text, world_height),
        Err(_) => layout_label(&LabelFont::load(), text, world_height),
    }
}
