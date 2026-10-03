// murali-example-tags: math,3d,fractal,portrait,shorts

// Reuse the generated fractal and color palettes from the landscape example.
#[allow(dead_code)]
#[path = "mandelbulb_3d.rs"]
mod mandelbulb_3d;

use glam::{Quat, Vec3, Vec4};
use mandelbulb_3d::{ColorPalette, Mandelbulb, tilted_rotation};
use murali::colors::*;
use murali::engine::camera::Projection;
use murali::engine::frame::Frame;
use murali::engine::render::RenderOptions;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::animation::Ease;
use murali::frontend::collection::primitives::ellipse::Ellipse;
use murali::frontend::collection::text::label::Label;
use murali::{App, DepthMode};

fn main() -> anyhow::Result<()> {
    let palette = ColorPalette::from_args();
    println!("Mandelbulb Shorts palette: {palette:?}");

    let mut scene = Scene::new()
        .with_frame(Frame::portrait())
        .with_background(Vec4::new(0.008, 0.010, 0.020, 1.0));

    let ground_id = scene.add_tattva(
        Ellipse::new(4.2, 3.2, Vec4::new(0.022, 0.028, 0.050, 1.0)).with_segments(96),
        Vec3::new(0.0, -2.05, 0.0),
    );
    scene.set_rotation(
        ground_id,
        Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
    );

    let shadow_id = scene.add_tattva(
        Ellipse::new(1.20, 0.60, Vec4::new(0.0, 0.0, 0.0, 0.66)).with_segments(72),
        Vec3::new(0.0, -2.01, 0.12),
    );
    scene.set_rotation(
        shadow_id,
        Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
    );

    let bulb_id = scene.add_tattva(Mandelbulb::new(palette), Vec3::new(0.0, -0.28, 0.0));
    scene.set_rotation(bulb_id, tilted_rotation(-0.55));
    scene.set_scale(bulb_id, Vec3::splat(0.08));
    scene.set_opacity(bulb_id, 0.0);

    scene.add_updater(bulb_id, move |scene, id, _dt| {
        let time = scene.scene_time;
        if let Some(bulb) = scene.get_tattva_typed_mut::<Mandelbulb>(id) {
            bulb.state.evolve(time);
        }

        scene.set_rotation(id, tilted_rotation(-0.55 + time * 0.36));
        scene.set_position_3d(id, Vec3::new(0.0, -0.28 + 0.11 * (time * 1.15).sin(), 0.0));
        let shadow_pulse = 1.0 - 0.08 * (time * 1.15).sin();
        scene.set_scale(shadow_id, Vec3::splat(shadow_pulse));
    });

    let title_id = scene.add_tattva(
        // Sized to keep the full word inside the 9:16 mobile-safe area.
        Label::new("MANDELBULB", 0.38).with_color(WHITE),
        Vec3::new(0.0, 2.62, 0.0),
    );
    let accent_id = scene.add_tattva(
        Label::new("THE LIVING FRACTAL", 0.18).with_color(BLUE_B),
        Vec3::new(0.0, 2.14, 0.0),
    );

    scene.camera_mut().projection = Projection::Perspective {
        fov_y_rad: 38.0_f32.to_radians(),
        aspect: 9.0 / 16.0,
        near: 0.1,
        far: 100.0,
    };
    scene.camera_mut().position = Vec3::new(0.12, 0.15, 9.5);
    scene.camera_mut().target = Vec3::new(0.0, -0.18, 0.0);
    for id in [title_id, accent_id] {
        scene.set_depth_mode(id, DepthMode::Overlay);
    }

    let mut timeline = Timeline::new();
    timeline
        .animate(title_id)
        .at(0.0)
        .for_duration(0.8)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(accent_id)
        .at(0.35)
        .for_duration(0.9)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(bulb_id)
        .at(0.35)
        .for_duration(1.6)
        .ease(Ease::OutCubic)
        .fade_to(1.0)
        .spawn();
    timeline
        .animate(bulb_id)
        .at(0.35)
        .for_duration(1.6)
        .ease(Ease::OutCubic)
        .scale_to(Vec3::splat(1.55))
        .spawn();

    timeline
        .animate(bulb_id)
        .at(2.0)
        .for_duration(10.5)
        .ease(Ease::InOutCubic)
        .scale_to(Vec3::splat(1.72))
        .spawn();

    scene.play(timeline)?;

    App::new()?
        .with_scene(scene)
        .with_render_options(RenderOptions {
            width: Some(1080),
            ..RenderOptions::default()
        })
        .run_app()
}
