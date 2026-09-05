//! # `murali::colors`
//!
//! Named color constants for use in animations.
//!
//! This module is **not** part of the prelude to avoid namespace pollution.
//! Import explicitly when needed:
//!
//! ```rust
//! use murali::colors::*;
//! ```
//!
//! ## Organization
//!
//! - **Achromatic:** `WHITE`, `BLACK`, `GRAY` (and shades `GRAY_A`–`GRAY_E`)
//! - **Color families:** `BLUE`, `TEAL`, `GREEN`, `YELLOW`, `GOLD`, `ORANGE`,
//!   `RED`, `MAROON`, `PURPLE`, `PINK` — each with five shades `_A` (lightest)
//!   through `_E` (darkest).
//! - **Pure primaries:** `PURE_RED`, `PURE_GREEN`, `PURE_BLUE`
//! - **Semantic colors:** [`semantic`] — theme-driven roles like `accent()`, `background()`.
//!
//! Shade suffixes run from `_A` (lightest) through `_E` (darkest). The
//! unsuffixed name (e.g. `BLUE`) is an alias for the mid-range shade (`_C`).
//!
//! All constants are `glam::Vec4` values in linear **RGBA** (0.0–1.0).

pub mod semantic;

use glam::Vec4;

// ---------------------------------------------------------------------------
// Internal compile-time hex helper
// ---------------------------------------------------------------------------

const fn hex(rgb: u32) -> Vec4 {
    let r = ((rgb >> 16) & 0xFF) as f32 / 255.0;
    let g = ((rgb >> 8) & 0xFF) as f32 / 255.0;
    let b = (rgb & 0xFF) as f32 / 255.0;
    Vec4::new(r, g, b, 1.0)
}

// ---------------------------------------------------------------------------
// Public runtime hex parser (used by Theme deserialization)
// ---------------------------------------------------------------------------

/// Parse a CSS-style hex color string into a `Vec4` (RGBA, 0.0–1.0).
///
/// Accepts both 6-digit (`#RRGGBB`, alpha = 1.0) and 8-digit
/// (`#RRGGBBAA`) forms. The leading `#` is optional.
///
/// # Errors
/// Returns a descriptive `String` on any parse failure.
pub fn from_hex(s: &str) -> Result<Vec4, String> {
    let s = s.trim().trim_start_matches('#');

    let parse_pair = |offset: usize| -> Result<f32, String> {
        let hi = s
            .chars()
            .nth(offset)
            .ok_or_else(|| format!("hex string too short: '{s}'"))?;
        let lo = s
            .chars()
            .nth(offset + 1)
            .ok_or_else(|| format!("hex string too short: '{s}'"))?;
        let byte = u8::from_str_radix(&format!("{hi}{lo}"), 16)
            .map_err(|e| format!("invalid hex byte at offset {offset}: {e}"))?;
        Ok(byte as f32 / 255.0)
    };

    match s.len() {
        6 => Ok(Vec4::new(
            parse_pair(0)?,
            parse_pair(2)?,
            parse_pair(4)?,
            1.0,
        )),
        8 => Ok(Vec4::new(
            parse_pair(0)?,
            parse_pair(2)?,
            parse_pair(4)?,
            parse_pair(6)?,
        )),
        n => Err(format!("expected 6 or 8 hex digits, got {n} in '{s}'")),
    }
}

// ---------------------------------------------------------------------------
// Achromatic
// ---------------------------------------------------------------------------

pub const WHITE: Vec4 = hex(0xFFFFFF);
pub const BLACK: Vec4 = hex(0x000000);

/// Light gray family
pub const GRAY_A: Vec4 = hex(0xDCDCDC);
pub const GRAY_B: Vec4 = hex(0xBBBBBB);
pub const GRAY_C: Vec4 = hex(0x888888);
pub const GRAY_D: Vec4 = hex(0x555555);
pub const GRAY_E: Vec4 = hex(0x222222);
/// Alias for `GRAY_C`.
pub const GRAY: Vec4 = GRAY_C;
/// Alias for `GRAY_C` (British spelling).
pub const GREY: Vec4 = GRAY_C;

// ---------------------------------------------------------------------------
// Blue
// ---------------------------------------------------------------------------

pub const BLUE_A: Vec4 = hex(0xC7E9F1);
pub const BLUE_B: Vec4 = hex(0x9CDCEB);
pub const BLUE_C: Vec4 = hex(0x58C4DD);
pub const BLUE_D: Vec4 = hex(0x29ABCA);
pub const BLUE_E: Vec4 = hex(0x1C758A);
/// Alias for `BLUE_C`.
pub const BLUE: Vec4 = BLUE_C;

// ---------------------------------------------------------------------------
// Teal
// ---------------------------------------------------------------------------

pub const TEAL_A: Vec4 = hex(0xACEAD7);
pub const TEAL_B: Vec4 = hex(0x76DDC0);
pub const TEAL_C: Vec4 = hex(0x5CD0B3);
pub const TEAL_D: Vec4 = hex(0x55C1A7);
pub const TEAL_E: Vec4 = hex(0x49A88F);
/// Alias for `TEAL_C`.
pub const TEAL: Vec4 = TEAL_C;

// ---------------------------------------------------------------------------
// Green
// ---------------------------------------------------------------------------

pub const GREEN_A: Vec4 = hex(0xC9E2AE);
pub const GREEN_B: Vec4 = hex(0xA6CF8C);
pub const GREEN_C: Vec4 = hex(0x83C167);
pub const GREEN_D: Vec4 = hex(0x77B05D);
pub const GREEN_E: Vec4 = hex(0x699C52);
/// Alias for `GREEN_C`.
pub const GREEN: Vec4 = GREEN_C;

// ---------------------------------------------------------------------------
// Yellow
// ---------------------------------------------------------------------------

pub const YELLOW_A: Vec4 = hex(0xFFF1B6);
pub const YELLOW_B: Vec4 = hex(0xFFEA94);
pub const YELLOW_C: Vec4 = hex(0xFFFF00);
pub const YELLOW_D: Vec4 = hex(0xF4D345);
pub const YELLOW_E: Vec4 = hex(0xE8C11C);
/// Alias for `YELLOW_C`.
pub const YELLOW: Vec4 = YELLOW_C;

// ---------------------------------------------------------------------------
// Gold
// ---------------------------------------------------------------------------

pub const GOLD_A: Vec4 = hex(0xF7C797);
pub const GOLD_B: Vec4 = hex(0xF9B775);
pub const GOLD_C: Vec4 = hex(0xF0AC5F);
pub const GOLD_D: Vec4 = hex(0xE1A158);
pub const GOLD_E: Vec4 = hex(0xC78D46);
/// Alias for `GOLD_C`.
pub const GOLD: Vec4 = GOLD_C;

// ---------------------------------------------------------------------------
// Orange
// ---------------------------------------------------------------------------

pub const ORANGE_A: Vec4 = hex(0xF7C59F);
pub const ORANGE_B: Vec4 = hex(0xFCAF80);
pub const ORANGE_C: Vec4 = hex(0xFF862F);
pub const ORANGE_D: Vec4 = hex(0xF26522); // slightly muted
pub const ORANGE_E: Vec4 = hex(0xD14C0A);
/// Alias for `ORANGE_C`.
pub const ORANGE: Vec4 = ORANGE_C;

// ---------------------------------------------------------------------------
// Red
// ---------------------------------------------------------------------------

pub const RED_A: Vec4 = hex(0xF7A1A3);
pub const RED_B: Vec4 = hex(0xFF8080);
pub const RED_C: Vec4 = hex(0xFC6255);
pub const RED_D: Vec4 = hex(0xE65A4C);
pub const RED_E: Vec4 = hex(0xCF5044);
/// Alias for `RED_C`.
pub const RED: Vec4 = RED_C;

// ---------------------------------------------------------------------------
// Maroon
// ---------------------------------------------------------------------------

pub const MAROON_A: Vec4 = hex(0xECABC1);
pub const MAROON_B: Vec4 = hex(0xEC92AB);
pub const MAROON_C: Vec4 = hex(0xC55F73);
pub const MAROON_D: Vec4 = hex(0xA24D61);
pub const MAROON_E: Vec4 = hex(0x94424F);
/// Alias for `MAROON_C`.
pub const MAROON: Vec4 = MAROON_C;

// ---------------------------------------------------------------------------
// Purple
// ---------------------------------------------------------------------------

pub const PURPLE_A: Vec4 = hex(0xCAA3E8);
pub const PURPLE_B: Vec4 = hex(0xB189C6);
pub const PURPLE_C: Vec4 = hex(0x9A72AC);
pub const PURPLE_D: Vec4 = hex(0x715582);
pub const PURPLE_E: Vec4 = hex(0x644172);
/// Alias for `PURPLE_C`.
pub const PURPLE: Vec4 = PURPLE_C;

// ---------------------------------------------------------------------------
// Pink
// ---------------------------------------------------------------------------

pub const PINK_A: Vec4 = hex(0xF4A2C0);
pub const PINK_B: Vec4 = hex(0xF5829B);
pub const PINK_C: Vec4 = hex(0xD147A3);
pub const PINK_D: Vec4 = hex(0xC2185B);
pub const PINK_E: Vec4 = hex(0xAD1457);
/// Alias for `PINK_C`.
pub const PINK: Vec4 = PINK_C;

// ---------------------------------------------------------------------------
// Pure primaries (saturated, display-RGB corners)
// ---------------------------------------------------------------------------

pub const PURE_RED: Vec4 = hex(0xFF0000);
pub const PURE_GREEN: Vec4 = hex(0x00FF00);
pub const PURE_BLUE: Vec4 = hex(0x0000FF);

// ---------------------------------------------------------------------------
// Color matching utilities
// ---------------------------------------------------------------------------

/// Find the closest color from the palette to the given RGBA value.
/// Uses Euclidean distance in RGB space (ignoring alpha for comparison).
pub fn closest_palette_color(color: Vec4) -> Vec4 {
    let palette = [
        WHITE, BLACK, GRAY_A, GRAY_B, GRAY_C, GRAY_D, GRAY_E, BLUE_A, BLUE_B, BLUE_C, BLUE_D,
        BLUE_E, TEAL_A, TEAL_B, TEAL_C, TEAL_D, TEAL_E, GREEN_A, GREEN_B, GREEN_C, GREEN_D,
        GREEN_E, YELLOW_A, YELLOW_B, YELLOW_C, YELLOW_D, YELLOW_E, GOLD_A, GOLD_B, GOLD_C, GOLD_D,
        GOLD_E, ORANGE_A, ORANGE_B, ORANGE_C, ORANGE_D, ORANGE_E, RED_A, RED_B, RED_C, RED_D,
        RED_E, MAROON_A, MAROON_B, MAROON_C, MAROON_D, MAROON_E, PURPLE_A, PURPLE_B, PURPLE_C,
        PURPLE_D, PURPLE_E, PINK_A, PINK_B, PINK_C, PINK_D, PINK_E, PURE_RED, PURE_GREEN,
        PURE_BLUE,
    ];

    let mut closest = palette[0];
    let mut min_distance = f32::INFINITY;

    for &palette_color in &palette {
        // Calculate Euclidean distance in RGB space
        let dr = color.x - palette_color.x;
        let dg = color.y - palette_color.y;
        let db = color.z - palette_color.z;
        let distance = (dr * dr + dg * dg + db * db).sqrt();

        if distance < min_distance {
            min_distance = distance;
            closest = palette_color;
        }
    }

    // Preserve the original alpha value
    Vec4::new(closest.x, closest.y, closest.z, color.w)
}
