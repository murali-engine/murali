use std::path::PathBuf;

use glam::{Vec3, Vec4, vec3};
use murali::engine::camera::Projection;
use murali::engine::export::{ExportSettings, PngCompressionMode};
use murali::frontend::collection::composite::beta::opening::{
    Opening, OpeningStyle, OpeningTiming,
};
use murali::{App, BuiltinTexture, Clip, Scene, TextureImage, Timeline};

const OPENING_TEXT: &str = "KAVRIQ";
const TAGLINE: &str = "The Science Behind AI";
const BACKGROUND: Vec4 = Vec4::new(0.039, 0.071, 0.11, 1.0);
const TAGLINE_COLOR: Vec4 = Vec4::new(0.95, 0.97, 0.99, 1.0);

const LETTER_HEIGHT: f32 = 2.4;
const LETTER_DEPTH: f32 = 0.95;
const LETTER_GAP: f32 = 0.34;
const PARTICLES_PER_LETTER: usize = 1400;
const PARTICLE_SIZE: f32 = 0.028;
const DISSOLVE_DURATION: f32 = 0.82;

const SATOSHI_BOLD: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/fonts/private/Satoshi-Bold.ttf"
);

const PARTICLE_PALETTE: [Vec4; 6] = [
    Vec4::new(0.18, 0.82, 0.78, 1.0),
    Vec4::new(0.32, 0.58, 1.0, 1.0),
    Vec4::new(0.92, 0.38, 0.68, 1.0),
    Vec4::new(1.0, 0.72, 0.22, 1.0),
    Vec4::new(0.48, 0.86, 0.38, 1.0),
    Vec4::new(1.0, 0.4, 0.3, 1.0),
];

fn build_scene() -> anyhow::Result<(Scene, f32)> {
    let mut scene = Scene::new();
    scene.camera_mut().projection = Projection::Perspective {
        fov_y_rad: 43.0_f32.to_radians(),
        aspect: 16.0 / 9.0,
        near: 0.1,
        far: 80.0,
    };
    scene.camera_mut().position = vec3(0.0, 2.15, 10.8);
    scene.camera_mut().target = vec3(0.0, -0.35, 0.0);

    let style = OpeningStyle {
        letter_height: LETTER_HEIGHT,
        letter_depth: LETTER_DEPTH,
        letter_gap: LETTER_GAP,
        particle_count: PARTICLES_PER_LETTER,
        particle_size: PARTICLE_SIZE,
        particle_palette: PARTICLE_PALETTE.to_vec(),
        tagline_color: TAGLINE_COLOR,
        ..OpeningStyle::default()
    };
    let timing = OpeningTiming {
        dissolve_duration: DISSOLVE_DURATION,
        ..OpeningTiming::default()
    };
    let mut opening = Opening::new(OPENING_TEXT, TAGLINE)
        .with_texture(TextureImage::builtin(BuiltinTexture::WhiteMarble))
        .with_style(style)
        .with_timing(timing);

    let brand_font = PathBuf::from(SATOSHI_BOLD);
    if brand_font.is_file() {
        opening = opening.with_font_path(brand_font);
    } else {
        eprintln!("private Satoshi Bold asset is missing at {SATOSHI_BOLD}; using bundled Inter");
    }

    let ids = opening.add_to_scene(&mut scene, Vec3::ZERO)?;
    let mut opening_clip = Clip::new();
    ids.animate(&mut opening_clip);
    let duration = opening_clip.duration();

    let mut timeline = Timeline::new();
    timeline.append(opening_clip);
    scene.play(timeline)?;
    Ok((scene, duration))
}

fn main() -> anyhow::Result<()> {
    let (scene, duration) = build_scene()?;
    let settings = ExportSettings {
        width: 1280,
        fps: 30,
        duration_seconds: duration,
        video_enabled: true,
        preserve_frame_exports: false,
        png_compression: PngCompressionMode::Fast,
        clear_color: BACKGROUND,
        ..ExportSettings::from_scene(&scene)
    };

    App::new()?
        .with_scene(scene)
        .with_export_settings(settings)
        .run_app()
}
