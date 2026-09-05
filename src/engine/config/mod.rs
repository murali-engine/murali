use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::path::Path;

pub(crate) mod export_config;
mod preview_config;

use export_config::ExportConfig;
use preview_config::PreviewConfig;

use crate::frontend::theme::Theme;
use crate::utils::project::find_project_root;

#[derive(Debug, Deserialize)]
struct MuraliConfig {
    pub theme: Option<String>,
    pub preview: Option<PreviewConfig>,
    pub export: Option<ExportConfig>,
}

fn default_theme_name() -> String {
    "dark".to_string()
}

fn load_murali_config(path: &Path) -> Result<MuraliConfig> {
    if !path.exists() {
        return Ok(MuraliConfig {
            theme: None,
            preview: None,
            export: None,
        });
    }

    let contents = fs::read_to_string(path)?;
    let cfg: MuraliConfig = toml::from_str(&contents)?;
    Ok(cfg)
}

#[derive(Debug, Clone)]
pub struct RenderConfig {
    pub text_px_per_world_unit: f32,
    pub fps: u32,
}

trait ResolveRenderConfig {
    fn resolve(self) -> RenderConfig;
}

impl ResolveRenderConfig for PreviewConfig {
    fn resolve(self) -> RenderConfig {
        RenderConfig {
            text_px_per_world_unit: self.text_px_per_world_unit,
            fps: self.fps,
        }
    }
}

impl ResolveRenderConfig for ExportConfig {
    fn resolve(self) -> RenderConfig {
        RenderConfig {
            text_px_per_world_unit: self.text_px_per_world_unit,
            fps: self.fps.unwrap_or(60),
        }
    }
}

impl RenderConfig {
    pub fn preview() -> Result<Self> {
        let cwd = std::env::current_dir()?;
        Self::preview_from(cwd)
    }

    pub fn preview_from(start: impl AsRef<Path>) -> Result<Self> {
        let project_root = find_project_root(start.as_ref());
        let murali_toml = project_root.join("murali.toml");

        let cfg = load_murali_config(&murali_toml)?;

        let preview_cfg = cfg.preview.unwrap_or_default();
        let theme_name = cfg.theme.unwrap_or_else(default_theme_name);

        let theme = Theme::load_by_name(&theme_name, &project_root);
        Theme::init_global(theme);

        Ok(preview_cfg.resolve())
    }

    pub fn export<P: AsRef<Path>>(config_path: Option<P>) -> Result<Self> {
        let cwd = std::env::current_dir()?;
        let project_root = find_project_root(&cwd);
        let murali_toml = project_root.join("murali.toml");

        let cfg = load_murali_config(&murali_toml)?;
        let theme_name = cfg.theme.unwrap_or_else(default_theme_name);

        let export_cfg = match config_path {
            Some(path) => ExportConfig::load(path)?,
            None => cfg.export.unwrap_or_default(),
        };

        let theme = Theme::load_by_name(&theme_name, &project_root);
        Theme::init_global(theme);

        Ok(export_cfg.resolve())
    }
}
