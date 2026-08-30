// src/resources/latex/mod.rs
pub mod backend;
pub mod error;
pub mod raster;
pub mod template;

use crate::resource::typst_resource::vector::{VectorSymbol, parse_svg_to_paths, scale_symbols};
use glam::Vec4;

const VECTOR_BASE_SIZE: f32 = 32.0;

/// Compile a LaTeX formula into filled, world-scaled path glyphs.
pub fn latex_vector_paths(
    source: &str,
    world_height: f32,
    color: Vec4,
) -> anyhow::Result<Vec<VectorSymbol>> {
    if source.trim().is_empty() {
        anyhow::bail!("source must not be empty");
    }
    if world_height <= 0.0 || !world_height.is_finite() {
        anyhow::bail!("height must be a positive finite number");
    }
    let cache_dir = std::env::temp_dir().join("murali_latex_cache");
    let latex = backend::compile_latex(source, &cache_dir)?;
    let mut symbols = parse_svg_to_paths(&latex.svg_content, color)?;
    if symbols.is_empty() {
        anyhow::bail!("formula vectorization produced no paths");
    }
    scale_symbols(&mut symbols, world_height / VECTOR_BASE_SIZE);
    Ok(symbols)
}
