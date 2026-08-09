use crate::resource::text::atlas::{GlyphAtlas, GlyphInfo};
use crate::resource::text::font::LabelFont;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

pub const DEFAULT_FONT_NAME: &str = "default";

pub struct LabelFontAsset {
    pub font: LabelFont,
    pub atlas: GlyphAtlas,
}

impl LabelFontAsset {
    fn new(font: LabelFont) -> Self {
        let atlas = GlyphAtlas::build(&font);
        Self { font, atlas }
    }
}

fn font_paths() -> &'static Mutex<HashMap<String, PathBuf>> {
    static FONT_PATHS: OnceLock<Mutex<HashMap<String, PathBuf>>> = OnceLock::new();
    FONT_PATHS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn font_cache() -> &'static Mutex<HashMap<String, Arc<LabelFontAsset>>> {
    static FONT_CACHE: OnceLock<Mutex<HashMap<String, Arc<LabelFontAsset>>>> = OnceLock::new();
    FONT_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn register_font_path(name: impl Into<String>, path: impl AsRef<Path>) -> Result<()> {
    let name = name.into();
    let path = path.as_ref();
    let _ = LabelFont::load_from_path(path)?;
    let mut paths = font_paths().lock().expect("font path registry poisoned");
    paths.insert(name.clone(), path.to_path_buf());
    drop(paths);

    let mut cache = font_cache().lock().expect("font cache poisoned");
    cache.remove(&name);
    Ok(())
}

pub fn font_asset(font_name: Option<&str>) -> Result<Arc<LabelFontAsset>> {
    let key = font_name.unwrap_or(DEFAULT_FONT_NAME).to_string();

    if let Some(existing) = font_cache()
        .lock()
        .expect("font cache poisoned")
        .get(&key)
        .cloned()
    {
        return Ok(existing);
    }

    let asset = if key == DEFAULT_FONT_NAME {
        Arc::new(LabelFontAsset::new(LabelFont::load_default()?))
    } else {
        let path = {
            let paths = font_paths().lock().expect("font path registry poisoned");
            paths.get(&key).cloned()
        }
        .with_context(|| format!("Font `{key}` is not registered"))?;
        Arc::new(LabelFontAsset::new(LabelFont::load_from_path(&path)?))
    };

    let mut cache = font_cache().lock().expect("font cache poisoned");
    cache.insert(key, asset.clone());
    Ok(asset)
}

/// Renamed from FontResourceManager to match the 'app.rs' expectation.
/// This acts as the "Baker" for all text metrics in the engine.
pub struct LabelResources {
    pub asset: Arc<LabelFontAsset>,
}

impl LabelResources {
    /// Lazy-loads the default font and builds the glyph atlas.
    pub fn new() -> Self {
        let asset = font_asset(None).expect("default font should always load");
        Self { asset }
    }

    /// Helper for the Layout engine to find where a character sits.
    pub fn get_glyph_metrics(&self, character: char) -> Option<&GlyphInfo> {
        self.asset.atlas.glyphs.get(&character)
    }
}

/// The state managed by the engine's Resource Registry.
pub struct TextResourceState {
    pub manager: Option<LabelResources>,
}

impl TextResourceState {
    pub fn ensure_loaded(&mut self) -> &LabelResources {
        self.manager.get_or_insert_with(LabelResources::new)
    }
}
