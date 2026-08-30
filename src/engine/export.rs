use std::collections::HashMap;
use std::fs;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use glam::Vec4;
use image::ColorType;
use image::ImageEncoder;
use image::codecs::png::{CompressionType, FilterType, PngEncoder};

use crate::engine::Engine;
use crate::engine::config::export_config::ExportConfig;
use crate::engine::render::RenderOptions;
use crate::engine::scene::Scene;
use crate::engine::timeline::Timeline;
use crate::frontend::theme::Theme;
use crate::utils::project::find_project_root;

#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PngCompressionMode {
    Fast,
    Balanced,
    Smallest,
}

impl Default for PngCompressionMode {
    fn default() -> Self {
        Self::Balanced
    }
}

impl PngCompressionMode {
    fn encoder_settings(self) -> (CompressionType, FilterType) {
        match self {
            Self::Fast => (CompressionType::Fast, FilterType::NoFilter),
            Self::Balanced => (CompressionType::Default, FilterType::Adaptive),
            Self::Smallest => (CompressionType::Best, FilterType::Adaptive),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExportSettings {
    /// Final output width in pixels. Height is derived from the scene frame.
    pub width: u32,
    pub fps: u32,
    pub duration_seconds: f32,
    pub artifact_dir: PathBuf,
    pub video_enabled: bool,
    pub preserve_frame_exports: bool,
    pub clear_color: Vec4,
    pub png_compression: PngCompressionMode,
}

impl Default for ExportSettings {
    fn default() -> Self {
        let stem = infer_default_export_stem();
        let artifact_dir = default_artifact_dir(&stem);
        Self {
            width: 1920,
            fps: 60,
            duration_seconds: 1.0,
            artifact_dir,
            video_enabled: true,
            preserve_frame_exports: false,
            clear_color: Vec4::new(0.05, 0.10, 0.15, 1.0),
            png_compression: PngCompressionMode::default(),
        }
    }
}

impl ExportSettings {
    pub fn from_scene(scene: &Scene) -> Self {
        let mut settings = Self::default();
        settings.width = scene.frame().default_export_width();
        settings.duration_seconds = infer_duration(scene);

        let bg = scene
            .background()
            .unwrap_or_else(|| Theme::global().background);
        settings.clear_color = bg;

        settings
    }

    pub fn from_project_config(scene: &Scene, options: &RenderOptions) -> Result<Self> {
        let cwd = std::env::current_dir()?;
        let project_root = find_project_root(&cwd);
        let cfg = ExportConfig::load(project_root.join("murali.toml"))?;

        let mut settings = Self::from_scene(scene);
        settings.width = options.width.or(cfg.width).unwrap_or(settings.width);
        settings.fps = options.fps.or(cfg.fps).unwrap_or(settings.fps);
        settings.duration_seconds = cfg.duration_seconds.unwrap_or(settings.duration_seconds);
        if let Some(artifact_dir) = cfg.artifact_dir {
            settings.artifact_dir = resolve_config_artifact_dir(&project_root, artifact_dir);
        }
        if let Some(clear_color) = cfg.clear_color {
            settings.clear_color = Vec4::new(
                clear_color[0],
                clear_color[1],
                clear_color[2],
                clear_color[3],
            );
        }
        if let Some(png_compression) = cfg.png_compression {
            settings.png_compression = png_compression;
        }
        settings.preserve_frame_exports = options.preserve_frames_explicitly_requested();
        if let Some(preserve_frame_exports) = cfg.preserve_frame_exports {
            settings.preserve_frame_exports = preserve_frame_exports;
        }
        settings.video_enabled = cfg.video_enabled.unwrap_or_else(|| options.video_enabled());

        Ok(settings)
    }

    pub fn total_frames(&self) -> u32 {
        let duration = self.duration_seconds.max(0.0);
        if duration == 0.0 {
            return 1;
        }
        let frame_intervals = duration * self.fps.max(1) as f32;
        let rounded = frame_intervals.round();
        let frame_intervals = if (frame_intervals - rounded).abs() <= 1e-4 {
            rounded as u32
        } else {
            frame_intervals.ceil() as u32
        }
        .max(1);
        frame_intervals.saturating_add(1)
    }

    pub fn frame_dt(&self) -> f32 {
        1.0 / self.fps.max(1) as f32
    }

    pub fn frame_time(&self, frame_index: u32) -> f32 {
        (frame_index as f32 * self.frame_dt()).min(self.duration_seconds.max(0.0))
    }

    pub fn pixel_dimensions(&self, scene: &Scene) -> Result<(u32, u32)> {
        if self.width == 0 {
            return Err(anyhow::anyhow!("export width must be greater than zero"));
        }
        Ok(scene.frame().pixel_dimensions(self.width))
    }

    pub fn export_stem(&self) -> String {
        self.resolved_artifact_dir()
            .file_name()
            .and_then(|name| name.to_str())
            .map(sanitize_stem)
            .filter(|name| !name.is_empty())
            .unwrap_or_else(infer_default_export_stem)
    }

    pub fn resolved_artifact_dir(&self) -> PathBuf {
        if self.artifact_dir.is_absolute() || starts_with_rendered_output(&self.artifact_dir) {
            self.artifact_dir.clone()
        } else {
            PathBuf::from("rendered_output").join(&self.artifact_dir)
        }
    }

    pub fn frame_dir(&self) -> PathBuf {
        self.resolved_artifact_dir().join("frames")
    }

    pub fn gif_dir(&self) -> PathBuf {
        self.resolved_artifact_dir().join("gifs")
    }

    pub fn video_path(&self) -> PathBuf {
        default_video_path(&self.resolved_artifact_dir(), &self.export_stem())
    }

    pub fn frame_path(&self, index: u32) -> PathBuf {
        self.frame_dir()
            .join(format!("{}_{index:05}.png", self.export_stem()))
    }
}

fn default_artifact_dir(stem: &str) -> PathBuf {
    PathBuf::from("rendered_output").join(stem)
}

fn default_video_path(artifact_dir: &Path, stem: &str) -> PathBuf {
    artifact_dir.join(format!("{stem}.mp4"))
}

fn starts_with_rendered_output(path: &Path) -> bool {
    path.components()
        .next()
        .and_then(|component| component.as_os_str().to_str())
        == Some("rendered_output")
}

fn infer_default_export_stem() -> String {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(stem) = exe_path.file_stem().and_then(|s| s.to_str()) {
            if !stem.is_empty() {
                return sanitize_stem(stem);
            }
        }
    }

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("murali_export_{ts}")
}

fn sanitize_stem(stem: &str) -> String {
    let sanitized: String = stem
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect();

    if sanitized.is_empty() {
        "murali_output".to_string()
    } else {
        sanitized
    }
}

fn resolve_config_artifact_dir(project_root: &Path, path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        return path;
    }
    if starts_with_rendered_output(&path) {
        project_root.join(path)
    } else {
        project_root.join("rendered_output").join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::{ExportSettings, infer_duration, resolve_config_artifact_dir};
    use crate::engine::scene::Scene;
    use crate::engine::scene_view::SceneView;
    use crate::engine::timeline::Timeline;
    use glam::Vec3;
    use std::path::{Path, PathBuf};

    #[test]
    fn export_stem_defaults_to_artifact_dir_name() {
        let settings = ExportSettings {
            artifact_dir: PathBuf::from("demo"),
            ..ExportSettings::default()
        };
        assert_eq!(settings.export_stem(), "demo");
    }

    #[test]
    fn relative_artifact_dir_is_nested_under_rendered_output() {
        let settings = ExportSettings {
            artifact_dir: PathBuf::from("demo"),
            ..ExportSettings::default()
        };
        assert_eq!(
            settings.resolved_artifact_dir(),
            PathBuf::from("rendered_output/demo")
        );
    }

    #[test]
    fn config_artifact_dir_is_nested_under_project_rendered_output() {
        let project_root = Path::new("/tmp/murali-project");
        let resolved = resolve_config_artifact_dir(project_root, PathBuf::from("demo"));
        assert_eq!(resolved, project_root.join("rendered_output/demo"));
    }

    #[test]
    fn non_aligned_duration_gets_an_exact_terminal_frame() {
        let settings = ExportSettings {
            fps: 10,
            duration_seconds: 1.04,
            ..ExportSettings::default()
        };

        assert_eq!(settings.total_frames(), 12);
        assert_eq!(settings.frame_time(10), 1.0);
        assert_eq!(settings.frame_time(11), 1.04);
    }

    #[test]
    fn aligned_duration_does_not_add_a_duplicate_terminal_frame() {
        let settings = ExportSettings {
            fps: 10,
            duration_seconds: 1.0,
            ..ExportSettings::default()
        };

        assert_eq!(settings.total_frames(), 11);
        assert_eq!(settings.frame_time(10), 1.0);
    }

    #[test]
    fn short_aligned_duration_is_stable_across_float_rounding() {
        let settings = ExportSettings {
            fps: 60,
            duration_seconds: 0.1,
            ..ExportSettings::default()
        };

        assert_eq!(settings.total_frames(), 7);
        assert_eq!(settings.frame_time(6), 0.1);
    }

    #[test]
    fn output_height_is_derived_from_the_scene_frame() {
        let settings = ExportSettings {
            width: 1080,
            ..ExportSettings::default()
        };

        assert_eq!(
            settings
                .pixel_dimensions(&Scene::new().with_frame(crate::engine::frame::Frame::portrait()))
                .unwrap(),
            (1080, 1920)
        );
        assert_eq!(
            settings
                .pixel_dimensions(&Scene::new().with_frame(crate::engine::frame::Frame::square()))
                .unwrap(),
            (1080, 1080)
        );
    }

    #[test]
    fn zero_export_width_is_rejected() {
        let settings = ExportSettings {
            width: 0,
            ..ExportSettings::default()
        };
        assert!(settings.pixel_dimensions(&Scene::new()).is_err());
    }

    #[test]
    fn child_timeline_contributes_to_inferred_export_duration() {
        let mut child = Scene::new();
        let mut child_timeline = Timeline::new();
        child_timeline.wait_until(3.0);
        child.play(child_timeline).unwrap();

        let mut parent = Scene::new();
        parent.add_scene_view(
            SceneView::new(child).start_at(2.0).time_scale(0.5),
            Vec3::ZERO,
        );

        assert!((infer_duration(&parent) - 8.0).abs() < 1e-6);
    }
}

pub fn infer_duration(scene: &Scene) -> f32 {
    let parent_end = scene
        .timeline
        .as_ref()
        .map(Timeline::end_time)
        .unwrap_or(0.0);
    scene
        .scene_views
        .values()
        .filter_map(|view| view.inferred_parent_end_time())
        .fold(parent_end, f32::max)
        .max(0.1)
}

pub fn export_scene(scene: Scene, settings: &ExportSettings) -> Result<()> {
    let (output_width, output_height) = settings.pixel_dimensions(&scene)?;
    fs::create_dir_all(settings.resolved_artifact_dir())?;
    if settings.video_enabled {
        fs::create_dir_all(settings.frame_dir())?;
        clear_existing_frame_outputs(settings)?;
    }
    let mut marker_gif_state = MarkerGifState::from_settings(settings, &scene)?;

    let mut engine = pollster::block_on(Engine::new_headless_with_scene(
        scene,
        output_width,
        output_height,
    ))?;
    engine
        .backend
        .renderer
        .resize(winit::dpi::PhysicalSize::new(output_width, output_height));
    engine.backend.renderer.clear_color = wgpu::Color {
        r: settings.clear_color.x as f64,
        g: settings.clear_color.y as f64,
        b: settings.clear_color.z as f64,
        a: settings.clear_color.w as f64,
    };

    for next_frame in 0..settings.total_frames() {
        let target_time = settings.frame_time(next_frame);
        let dt = (target_time - engine.scene.scene_time).max(0.0);

        let frame_start = Instant::now();
        if next_frame == 0 {
            eprintln!("Export frame 1: starting update");
        }
        engine
            .update(dt)
            .with_context(|| format!("Failed to update export frame {}", next_frame))?;
        if next_frame == 0 {
            eprintln!(
                "Export frame 1: update finished in {:.2?}",
                frame_start.elapsed()
            );
        }

        let render_start = Instant::now();
        let image = engine
            .render_to_image()
            .with_context(|| format!("Failed to render export frame {}", next_frame))?;
        if next_frame == 0 {
            eprintln!(
                "Export frame 1: render/readback finished in {:.2?}",
                render_start.elapsed()
            );
        }

        save_requested_screenshots(&image, &engine.scene, settings, &mut marker_gif_state)
            .with_context(|| {
                format!(
                    "Failed to save requested screenshot at frame {}",
                    next_frame
                )
            })?;

        if settings.video_enabled {
            let save_start = Instant::now();
            save_png_fast(
                &image,
                &settings.frame_path(next_frame),
                settings.png_compression,
            )
            .with_context(|| format!("Failed to save export frame {}", next_frame))?;
            if next_frame == 0 {
                eprintln!(
                    "Export frame 1: png save finished in {:.2?}",
                    save_start.elapsed()
                );
                eprintln!(
                    "Export frame 1: total frame time {:.2?}",
                    frame_start.elapsed()
                );
            }
        }

        if next_frame == 0 || next_frame + 1 == settings.total_frames() || next_frame % 10 == 0 {
            eprintln!(
                "Export progress: frame {}/{}",
                next_frame + 1,
                settings.total_frames()
            );
        }
    }

    let mut video_assembled = false;
    if settings.video_enabled {
        assemble_video(settings, &settings.video_path())?;
        video_assembled = true;
    }

    marker_gif_state.assemble_all(settings)?;

    if video_assembled && !settings.preserve_frame_exports {
        remove_main_frame_outputs(settings)?;
        marker_gif_state.cleanup_temp_frames()?;
    }

    Ok(())
}

fn assemble_video(settings: &ExportSettings, video_path: &Path) -> Result<()> {
    if !ffmpeg_available() {
        return Err(anyhow::anyhow!(
            "ffmpeg not found in PATH; PNG frames were exported to {}",
            settings.frame_dir().display()
        ));
    }

    if let Some(parent) = video_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let pattern = settings
        .frame_dir()
        .join(format!("{}_%05d.png", settings.export_stem()));

    let status = Command::new("ffmpeg")
        .arg("-y")
        .arg("-framerate")
        .arg(settings.fps.to_string())
        .arg("-i")
        .arg(pattern)
        .arg("-pix_fmt")
        .arg("yuv420p")
        .arg(video_path)
        .status()
        .context("Failed to spawn ffmpeg for video assembly")?;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "ffmpeg exited with status {status}; frames remain in {}",
            settings.frame_dir().display()
        ));
    }

    Ok(())
}

fn ffmpeg_available() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn clear_existing_frame_outputs(settings: &ExportSettings) -> Result<()> {
    let frame_dir = settings.frame_dir();
    if frame_dir.exists() {
        for entry in fs::read_dir(&frame_dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if name.starts_with(&format!("{}_", settings.export_stem())) && name.ends_with(".png") {
                fs::remove_file(path)?;
            }
        }
    }

    Ok(())
}

fn remove_main_frame_outputs(settings: &ExportSettings) -> Result<()> {
    clear_existing_frame_outputs(settings)?;
    let frame_dir = settings.frame_dir();
    if frame_dir.exists() && fs::read_dir(&frame_dir)?.next().is_none() {
        fs::remove_dir(&frame_dir)?;
    }
    Ok(())
}

fn save_requested_screenshots(
    image: &image::RgbaImage,
    scene: &Scene,
    settings: &ExportSettings,
    marker_gif_state: &mut MarkerGifState,
) -> Result<()> {
    marker_gif_state.capture_scene_specs(image, scene, settings)?;

    Ok(())
}

#[derive(Debug, Default)]
struct MarkerGifState {
    gif_dir: Option<PathBuf>,
    frame_counts: HashMap<String, usize>,
    screenshot_count: usize,
    scene_screenshot_indices: Vec<usize>,
    scene_gif_indices: Vec<usize>,
}

impl MarkerGifState {
    fn from_settings(settings: &ExportSettings, scene: &Scene) -> Result<Self> {
        for capture in &scene.screenshot_captures {
            if let Some(names) = &capture.names {
                if names.len() != capture.times.len() {
                    return Err(anyhow::anyhow!(
                        "ScreenshotCapture names length ({}) must match times length ({})",
                        names.len(),
                        capture.times.len()
                    ));
                }
            }
        }

        if scene.gif_captures.is_empty() {
            return Ok(Self {
                gif_dir: None,
                frame_counts: HashMap::new(),
                screenshot_count: 0,
                scene_screenshot_indices: vec![0; scene.screenshot_captures.len()],
                scene_gif_indices: vec![0; scene.gif_captures.len()],
            });
        }
        let gif_dir = settings.gif_dir();
        fs::create_dir_all(&gif_dir)?;
        let frames_root = marker_gif_frames_root(&gif_dir);
        if frames_root.exists() {
            fs::remove_dir_all(&frames_root)?;
        }
        fs::create_dir_all(&frames_root)?;
        Ok(Self {
            gif_dir: Some(gif_dir),
            frame_counts: HashMap::new(),
            screenshot_count: 0,
            scene_screenshot_indices: vec![0; scene.screenshot_captures.len()],
            scene_gif_indices: vec![0; scene.gif_captures.len()],
        })
    }

    fn capture_scene_specs(
        &mut self,
        image: &image::RgbaImage,
        scene: &Scene,
        settings: &ExportSettings,
    ) -> Result<()> {
        let current_time = scene.scene_time;

        for (capture_idx, capture) in scene.screenshot_captures.iter().enumerate() {
            while self.scene_screenshot_indices[capture_idx] < capture.times.len()
                && current_time + 1e-4 >= capture.times[self.scene_screenshot_indices[capture_idx]]
            {
                let current_idx = self.scene_screenshot_indices[capture_idx];
                let requested = capture
                    .names
                    .as_ref()
                    .and_then(|names| names.get(current_idx))
                    .and_then(|name| name.as_deref());
                let path = self.resolve_screenshot_path(settings, requested);
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }
                save_png_fast(image, &path, settings.png_compression)?;
                self.scene_screenshot_indices[capture_idx] += 1;
            }
        }

        for (capture_idx, capture) in scene.gif_captures.iter().enumerate() {
            while self.scene_gif_indices[capture_idx] < capture.times.len()
                && current_time + 1e-4 >= capture.times[self.scene_gif_indices[capture_idx]]
            {
                self.capture_groups(image, settings, std::slice::from_ref(&capture.name))?;
                self.scene_gif_indices[capture_idx] += 1;
            }
        }

        Ok(())
    }

    fn capture_groups(
        &mut self,
        image: &image::RgbaImage,
        settings: &ExportSettings,
        groups: &[String],
    ) -> Result<()> {
        let Some(gif_dir) = self.gif_dir.as_ref() else {
            return Ok(());
        };

        for group in groups {
            let group_key = sanitize_stem(group);
            let next_index = self.frame_counts.entry(group_key.clone()).or_insert(0);
            let frame_path = marker_gif_frames_root(gif_dir)
                .join(&group_key)
                .join(format!("{group_key}_{:05}.png", *next_index));
            if let Some(parent) = frame_path.parent() {
                fs::create_dir_all(parent)?;
            }
            save_png_fast(image, &frame_path, settings.png_compression)?;
            *next_index += 1;
        }

        let _ = settings;
        Ok(())
    }

    fn assemble_all(&self, settings: &ExportSettings) -> Result<()> {
        let Some(gif_dir) = self.gif_dir.as_ref() else {
            return Ok(());
        };
        if self.frame_counts.is_empty() {
            return Ok(());
        }
        if !ffmpeg_available() {
            return Err(anyhow::anyhow!(
                "ffmpeg not found in PATH; marker GIF frames were exported to {}",
                marker_gif_frames_root(gif_dir).display()
            ));
        }

        for (group, frame_count) in &self.frame_counts {
            if *frame_count == 0 {
                continue;
            }
            let gif_path = gif_dir.join(format!("{group}.gif"));
            assemble_marker_gif(settings, gif_dir, group, &gif_path)?;
        }
        Ok(())
    }

    fn cleanup_temp_frames(&self) -> Result<()> {
        let Some(gif_dir) = self.gif_dir.as_ref() else {
            return Ok(());
        };
        let frames_root = marker_gif_frames_root(gif_dir);
        if frames_root.exists() {
            fs::remove_dir_all(frames_root)?;
        }
        Ok(())
    }

    fn resolve_screenshot_path(
        &mut self,
        settings: &ExportSettings,
        requested: Option<&Path>,
    ) -> PathBuf {
        match requested {
            Some(path) => resolve_screenshot_path(settings, path),
            None => {
                let path = settings
                    .resolved_artifact_dir()
                    .join("captures")
                    .join(format!("capture_{:05}.png", self.screenshot_count));
                self.screenshot_count += 1;
                path
            }
        }
    }
}

fn assemble_marker_gif(
    settings: &ExportSettings,
    output_dir: &Path,
    group: &str,
    gif_path: &Path,
) -> Result<()> {
    if let Some(parent) = gif_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let pattern = marker_gif_frames_root(output_dir)
        .join(group)
        .join(format!("{group}_%05d.png"));
    let filter = format!(
        "fps={},scale={}:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse",
        settings.fps, settings.width
    );

    let status = Command::new("ffmpeg")
        .arg("-y")
        .arg("-i")
        .arg(pattern)
        .arg("-vf")
        .arg(filter)
        .arg(gif_path)
        .status()
        .context("Failed to spawn ffmpeg for marker GIF assembly")?;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "ffmpeg exited with status {status}; marker GIF frames remain in {}",
            marker_gif_frames_root(output_dir).display()
        ));
    }

    Ok(())
}

fn marker_gif_frames_root(output_dir: &Path) -> PathBuf {
    output_dir.join(".frames")
}

fn resolve_screenshot_path(settings: &ExportSettings, requested: &Path) -> PathBuf {
    if requested.is_absolute() {
        requested.to_path_buf()
    } else {
        settings.resolved_artifact_dir().join(requested)
    }
}

fn save_png_fast(
    image: &image::RgbaImage,
    path: &Path,
    png_compression: PngCompressionMode,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let file = fs::File::create(path)?;
    let writer = BufWriter::new(file);
    let (compression, filter) = png_compression.encoder_settings();
    let encoder = PngEncoder::new_with_quality(writer, compression, filter);
    encoder.write_image(
        image.as_raw(),
        image.width(),
        image.height(),
        ColorType::Rgba8.into(),
    )?;
    Ok(())
}
