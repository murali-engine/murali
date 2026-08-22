//! # `murali::colors::semantic`
//!
//! Theme-role color accessors. Instead of choosing a literal hex color, use
//! these functions to pick up whatever theme is active at runtime.
//!
//! ```rust
//! use murali::colors::semantic::*;
//! use murali::frontend::collection::primitives::circle::Circle;
//!
//! let circle = Circle::new(1.0, 48, accent());
//! ```
//!
//! The active theme is set by the `theme` key in `murali.toml`:
//! ```toml
//! [preview]
//! theme = "light"
//! ```
//! or by placing a custom TOML file at `themes/{name}.toml` in the project root.
//!
//! If no theme is configured, the built-in **dark** theme is used.

use crate::frontend::theme::Theme;
use glam::Vec4;

/// Canvas background color.
pub fn background() -> Vec4 {
    Theme::global().background
}

/// Primary card / panel surface color.
pub fn surface() -> Vec4 {
    Theme::global().surface
}

/// Secondary / elevated surface color.
pub fn surface_alt() -> Vec4 {
    Theme::global().surface_alt
}

/// Main body text / foreground color.
pub fn text_primary() -> Vec4 {
    Theme::global().text_primary
}

/// Secondary or de-emphasized text color.
pub fn text_muted() -> Vec4 {
    Theme::global().text_muted
}

/// Primary accent color — highlights, active elements, key annotations.
pub fn accent() -> Vec4 {
    Theme::global().accent
}

/// Secondary accent — alternate highlights or complementary annotations.
pub fn accent_alt() -> Vec4 {
    Theme::global().accent_alt
}

/// Positive / success indicator color.
pub fn positive() -> Vec4 {
    Theme::global().positive
}

/// Warning / caution indicator color.
pub fn warning() -> Vec4 {
    Theme::global().warning
}
