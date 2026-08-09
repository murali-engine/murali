use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

use crate::engine::export::PngCompressionMode;
use crate::utils::project::find_project_root;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ExportConfig {
    #[serde(default = "default_text_px_per_world_unit")]
    pub text_px_per_world_unit: f32,
    pub width: Option<u32>,
    pub fps: Option<u32>,
    pub duration_seconds: Option<f32>,
    #[serde(alias = "output_dir")]
    pub artifact_dir: Option<PathBuf>,
    pub video_enabled: Option<bool>,
    pub preserve_frame_exports: Option<bool>,
    pub clear_color: Option<[f32; 4]>,
    pub png_compression: Option<PngCompressionMode>,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            text_px_per_world_unit: default_text_px_per_world_unit(),
            width: None,
            fps: None,
            duration_seconds: None,
            artifact_dir: None,
            video_enabled: None,
            preserve_frame_exports: None,
            clear_color: None,
            png_compression: None,
        }
    }
}

fn default_text_px_per_world_unit() -> f32 {
    512.0
}

impl ExportConfig {
    #[must_use]
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(path)?;
        let file_cfg: FileConfig = toml::from_str(&contents)?;

        Ok(file_cfg.export.unwrap_or_default())
    }

    pub fn load_nearest_project_file(start_dir: impl AsRef<Path>) -> Result<Self> {
        let project_dir = find_project_root(start_dir.as_ref());
        Self::load(project_dir.join("murali.toml"))
    }
}

#[derive(Debug, Deserialize)]
struct FileConfig {
    pub export: Option<ExportConfig>,
}
