use glam::{Vec3, Vec4, vec2};
use murali::App;
use murali::engine::export::{ExportSettings, PngCompressionMode, export_scene};
use murali::engine::frame::Frame;
use murali::engine::scene::Scene;
use murali::frontend::TattvaId;
use murali::frontend::collection::primitives::{circle::Circle, line::Line, path::Path};
use murali::positions::CAMERA_DEFAULT_POS;
use std::path::PathBuf;

const DRAW_OUTER_SQUARE: bool = false;

#[derive(Clone, Copy)]
enum LogoMode {
    Dark,
    Light,
}

#[derive(Clone, Copy)]
struct LogoPalette {
    frame: Vec4,
    grid: Vec4,
    axis: Vec4,
    support: Vec4,
    mark: Vec4,
    handle: Vec4,
    guide_a: Vec4,
    guide_b: Vec4,
    dot_a: Vec4,
    dot_b: Vec4,
    dot_c: Vec4,
    dot_stroke: Vec4,
}

fn palette_for(mode: LogoMode) -> LogoPalette {
    match mode {
        LogoMode::Dark => LogoPalette {
            frame: Vec4::new(0.20, 0.72, 0.98, 0.78),
            grid: Vec4::new(0.20, 0.72, 0.98, 0.38),
            axis: Vec4::new(0.28, 0.84, 0.95, 0.92),
            support: Vec4::new(0.18, 0.58, 0.98, 0.96),
            mark: Vec4::new(0.18, 0.96, 0.76, 1.0),
            handle: Vec4::new(0.92, 0.98, 1.0, 0.42),
            guide_a: Vec4::new(0.30, 0.70, 0.98, 0.74),
            guide_b: Vec4::new(0.16, 0.94, 0.80, 0.74),
            dot_a: Vec4::new(0.18, 0.58, 0.98, 1.0),
            dot_b: Vec4::new(0.28, 0.74, 1.0, 1.0),
            dot_c: Vec4::new(0.18, 0.96, 0.76, 1.0),
            dot_stroke: Vec4::new(1.0, 1.0, 1.0, 0.95),
        },
        LogoMode::Light => {
            let black = Vec4::new(0.0, 0.0, 0.0, 1.0);
            let black_soft = Vec4::new(0.0, 0.0, 0.0, 0.72);
            let black_faint = Vec4::new(0.0, 0.0, 0.0, 0.30);
            LogoPalette {
                frame: black_soft,
                grid: black_faint,
                axis: black_soft,
                support: black,
                mark: black,
                handle: black_faint,
                guide_a: black_faint,
                guide_b: black_faint,
                dot_a: black,
                dot_b: black,
                dot_c: black,
                dot_stroke: Vec4::new(0.0, 0.0, 0.0, 0.9),
            }
        }
    }
}

fn add_line(
    scene: &mut Scene,
    start: (f32, f32),
    end: (f32, f32),
    thickness: f32,
    color: Vec4,
) -> TattvaId {
    scene.add_tattva(
        Line::new(
            Vec3::new(start.0, start.1, 0.0),
            Vec3::new(end.0, end.1, 0.0),
            thickness,
            color,
        ),
        Vec3::ZERO,
    )
}

fn build_scene(mode: LogoMode, capture_name: Option<&str>) -> Scene {
    let mut scene = Scene::new().with_frame(Frame::square());
    let palette = palette_for(mode);

    let left = -3.2;
    let right = 3.2;
    let bottom = -2.4;
    let top = 2.4;

    if DRAW_OUTER_SQUARE {
        let _ = vec![
            add_line(&mut scene, (left, bottom), (left, top), 0.06, palette.frame),
            add_line(&mut scene, (left, top), (right, top), 0.06, palette.frame),
            add_line(
                &mut scene,
                (right, top),
                (right, bottom),
                0.06,
                palette.frame,
            ),
            add_line(
                &mut scene,
                (right, bottom),
                (left, bottom),
                0.06,
                palette.frame,
            ),
        ];
    }

    let _ = vec![
        add_line(&mut scene, (-2.4, bottom), (-2.4, top), 0.03, palette.grid),
        add_line(&mut scene, (-1.6, bottom), (-1.6, top), 0.034, palette.grid),
        add_line(&mut scene, (-0.8, bottom), (-0.8, top), 0.03, palette.grid),
        add_line(&mut scene, (0.0, bottom), (0.0, top), 0.04, palette.axis),
        add_line(&mut scene, (0.8, bottom), (0.8, top), 0.03, palette.grid),
        add_line(&mut scene, (1.6, bottom), (1.6, top), 0.034, palette.grid),
        add_line(&mut scene, (2.4, bottom), (2.4, top), 0.03, palette.grid),
        add_line(&mut scene, (left, -1.8), (right, -1.8), 0.03, palette.grid),
        add_line(&mut scene, (left, -1.2), (right, -1.2), 0.034, palette.grid),
        add_line(&mut scene, (left, -0.6), (right, -0.6), 0.03, palette.grid),
        add_line(&mut scene, (left, 0.0), (right, 0.0), 0.04, palette.axis),
        add_line(&mut scene, (left, 0.6), (right, 0.6), 0.03, palette.grid),
        add_line(&mut scene, (left, 1.2), (right, 1.2), 0.034, palette.grid),
        add_line(&mut scene, (left, 1.8), (right, 1.8), 0.03, palette.grid),
    ];

    scene.add_tattva(
        Path::new()
            .move_to(vec2(-2.45, -1.62))
            .cubic_to(vec2(-1.85, -1.05), vec2(-0.92, -0.72), vec2(0.0, -0.72))
            .cubic_to(vec2(0.92, -0.72), vec2(1.85, -1.05), vec2(2.45, -1.62))
            .with_thickness(0.13)
            .with_color(palette.support),
        Vec3::ZERO,
    );

    let _ = vec![
        add_line(
            &mut scene,
            (-2.45, -1.62),
            (-1.85, -1.05),
            0.03,
            palette.handle,
        ),
        add_line(
            &mut scene,
            (0.0, -0.72),
            (-0.92, -0.72),
            0.03,
            palette.handle,
        ),
        add_line(
            &mut scene,
            (0.0, -0.72),
            (0.92, -0.72),
            0.03,
            palette.handle,
        ),
        add_line(
            &mut scene,
            (2.45, -1.62),
            (1.85, -1.05),
            0.03,
            palette.handle,
        ),
    ];

    scene.add_tattva(
        Path::new()
            .move_to(vec2(-2.55, -1.52))
            .cubic_to(vec2(-2.62, -0.18), vec2(-2.30, 1.52), vec2(-1.72, 1.86))
            .cubic_to(vec2(-1.18, 2.12), vec2(-0.58, 0.92), vec2(0.0, 0.08))
            .cubic_to(vec2(0.58, 0.92), vec2(1.18, 2.12), vec2(1.72, 1.86))
            .cubic_to(vec2(2.30, 1.52), vec2(2.62, -0.18), vec2(2.55, -1.52))
            .with_thickness(0.18)
            .with_color(palette.mark),
        Vec3::ZERO,
    );

    let _ = vec![
        add_line(
            &mut scene,
            (-2.55, -1.52),
            (-2.62, -0.18),
            0.032,
            palette.handle,
        ),
        add_line(
            &mut scene,
            (-1.72, 1.86),
            (-2.30, 1.52),
            0.032,
            palette.handle,
        ),
        add_line(
            &mut scene,
            (-1.72, 1.86),
            (-1.18, 2.12),
            0.032,
            palette.handle,
        ),
        add_line(
            &mut scene,
            (0.0, 0.08),
            (-0.58, 0.92),
            0.032,
            palette.handle,
        ),
        add_line(&mut scene, (0.0, 0.08), (0.58, 0.92), 0.032, palette.handle),
        add_line(
            &mut scene,
            (1.72, 1.86),
            (1.18, 2.12),
            0.032,
            palette.handle,
        ),
        add_line(
            &mut scene,
            (1.72, 1.86),
            (2.30, 1.52),
            0.032,
            palette.handle,
        ),
        add_line(
            &mut scene,
            (2.55, -1.52),
            (2.62, -0.18),
            0.032,
            palette.handle,
        ),
    ];

    let _ = vec![
        add_line(&mut scene, (-1.6, 2.4), (0.0, 0.0), 0.045, palette.guide_a),
        add_line(&mut scene, (0.0, 0.0), (1.6, 2.4), 0.045, palette.guide_b),
    ];

    let _ = vec![
        scene.add_tattva(
            Circle::new(0.135, 28, palette.dot_a).with_stroke(0.028, palette.dot_stroke),
            Vec3::new(-2.45, -1.62, 0.0),
        ),
        scene.add_tattva(
            Circle::new(0.135, 28, palette.dot_b).with_stroke(0.028, palette.dot_stroke),
            Vec3::new(-1.6, 1.9, 0.0),
        ),
        scene.add_tattva(
            Circle::new(0.135, 28, palette.dot_c).with_stroke(0.028, palette.dot_stroke),
            Vec3::new(1.6, 1.9, 0.0),
        ),
        scene.add_tattva(
            Circle::new(0.135, 28, palette.dot_a).with_stroke(0.028, palette.dot_stroke),
            Vec3::new(-1.55, -0.98, 0.0),
        ),
        scene.add_tattva(
            Circle::new(0.135, 28, palette.dot_a).with_stroke(0.028, palette.dot_stroke),
            Vec3::new(1.55, -0.98, 0.0),
        ),
    ];

    let _ = vec![
        scene.add_tattva(
            Circle::new(0.075, 24, palette.handle).with_stroke(0.02, palette.dot_stroke),
            Vec3::new(-2.62, -0.18, 0.0),
        ),
        scene.add_tattva(
            Circle::new(0.075, 24, palette.handle).with_stroke(0.02, palette.dot_stroke),
            Vec3::new(-2.30, 1.52, 0.0),
        ),
        scene.add_tattva(
            Circle::new(0.075, 24, palette.handle).with_stroke(0.02, palette.dot_stroke),
            Vec3::new(-1.18, 2.12, 0.0),
        ),
        scene.add_tattva(
            Circle::new(0.075, 24, palette.handle).with_stroke(0.02, palette.dot_stroke),
            Vec3::new(-0.58, 0.92, 0.0),
        ),
        scene.add_tattva(
            Circle::new(0.075, 24, palette.handle).with_stroke(0.02, palette.dot_stroke),
            Vec3::new(0.58, 0.92, 0.0),
        ),
        scene.add_tattva(
            Circle::new(0.075, 24, palette.handle).with_stroke(0.02, palette.dot_stroke),
            Vec3::new(1.18, 2.12, 0.0),
        ),
        scene.add_tattva(
            Circle::new(0.075, 24, palette.handle).with_stroke(0.02, palette.dot_stroke),
            Vec3::new(2.30, 1.52, 0.0),
        ),
        scene.add_tattva(
            Circle::new(0.075, 24, palette.handle).with_stroke(0.02, palette.dot_stroke),
            Vec3::new(2.62, -0.18, 0.0),
        ),
    ];

    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    scene.camera_mut().set_view_width(10.0);

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
    let preview = std::env::args().any(|arg| arg == "--preview");

    if preview {
        return App::new()?
            .with_scene(build_scene(LogoMode::Dark, None))
            .with_preview()
            .run_app();
    }

    let settings = export_settings();
    export_scene(
        build_scene(LogoMode::Dark, Some("murali_logo_dark.png")),
        &settings,
    )?;
    export_scene(
        build_scene(LogoMode::Light, Some("murali_logo_light.png")),
        &settings,
    )?;
    Ok(())
}
