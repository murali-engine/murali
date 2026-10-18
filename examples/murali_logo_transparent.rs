use glam::{Vec3, Vec4, vec2};
use murali::App;
use murali::engine::export::{ExportSettings, PngCompressionMode, export_scene};
use murali::engine::frame::Frame;
use murali::engine::scene::Scene;
use murali::frontend::collection::primitives::path::Path;
use murali::frontend::style::Style;
use murali::positions::CAMERA_DEFAULT_POS;
use std::path::PathBuf;

const MURALI_CORAL: Vec4 = Vec4::new(1.0, 107.0 / 255.0, 95.0 / 255.0, 1.0);

fn murali_mark() -> Path {
    Path::new()
        .move_to(vec2(-2.60, -1.70))
        .line_to(vec2(-2.60, 1.45))
        .quad_to(vec2(-2.60, 1.80), vec2(-2.25, 1.80))
        .line_to(vec2(-1.85, 1.80))
        .quad_to(vec2(-1.65, 1.80), vec2(-1.50, 1.62))
        .line_to(vec2(0.00, 0.05))
        .line_to(vec2(1.50, 1.62))
        .quad_to(vec2(1.65, 1.80), vec2(1.85, 1.80))
        .line_to(vec2(2.25, 1.80))
        .quad_to(vec2(2.60, 1.80), vec2(2.60, 1.45))
        .line_to(vec2(2.60, -1.70))
        .line_to(vec2(1.62, -1.70))
        .line_to(vec2(1.62, 0.14))
        .line_to(vec2(0.42, -1.12))
        .quad_to(vec2(0.00, -1.56), vec2(-0.42, -1.12))
        .line_to(vec2(-1.62, 0.14))
        .line_to(vec2(-1.62, -1.70))
        .close()
        .with_style(Style::new().with_fill(MURALI_CORAL))
}

fn build_scene(capture_name: Option<&str>) -> Scene {
    let mut scene = Scene::new().with_frame(Frame::square());
    scene.add_tattva(murali_mark(), Vec3::ZERO);
    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    scene.camera_mut().set_view_width(7.2);

    if let Some(name) = capture_name {
        scene.capture_screenshots_named([(0.0, Some(name))]);
    }

    scene
}

fn export_settings() -> ExportSettings {
    ExportSettings {
        width: 1200,
        fps: 30,
        duration_seconds: 0.1,
        artifact_dir: PathBuf::from("murali_logo_transparent"),
        video_enabled: false,
        preserve_frame_exports: false,
        clear_color: Vec4::new(0.0, 0.0, 0.0, 0.0),
        png_compression: PngCompressionMode::Balanced,
    }
}

fn main() -> anyhow::Result<()> {
    if std::env::args().any(|arg| arg == "--preview") {
        return App::new()?
            .with_scene(build_scene(None))
            .with_preview()
            .run_app();
    }

    let settings = export_settings();
    export_scene(build_scene(Some("murali_logo_dark.png")), &settings)?;
    export_scene(build_scene(Some("murali_logo_light.png")), &settings)?;
    Ok(())
}
