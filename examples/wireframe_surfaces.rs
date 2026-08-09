use glam::Vec3;
use murali::colors::*;
use murali::engine::camera::Projection;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::animation::Ease;
use murali::frontend::collection::composite::axes3d::Axes3D;
use murali::frontend::collection::graph::parametric_surface::{
    ParametricSurface, SurfaceRenderMode,
};
use murali::frontend::collection::text::label::Label;
use murali::{App, DepthMode};

fn saddle_surface(u: f32, v: f32) -> Vec3 {
    Vec3::new(u, 0.26 * (u * u - v * v), v)
}

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    let title_id = scene.add_tattva(
        Label::new("Wireframe Surfaces", 0.38).with_color(WHITE),
        Vec3::new(0.0, 3.0, 0.0),
    );
    let subtitle_id = scene.add_tattva(
        Label::new(
            "Wireframes are best when the grid itself teaches the curvature, without a filled surface competing for attention.",
            0.18,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, 2.45, 0.0),
    );

    let axes_id = scene.add_tattva(
        Axes3D::new((-2.6, 2.6), (-1.4, 1.4), (-2.6, 2.6))
            .with_step(1.0)
            .with_axis_thickness(0.03)
            .with_tick_size(0.12),
        Vec3::ZERO,
    );

    let surface_id = scene.add_tattva(
        ParametricSurface::new((-2.0, 2.0), (-2.0, 2.0), saddle_surface)
            .with_samples(30, 30)
            .with_render_mode(SurfaceRenderMode::Wireframe)
            .with_write_progress(0.0)
            .with_color_fn(|h| {
                let t = ((h + 1.0) / 2.0).clamp(0.0, 1.0);
                BLUE_B.lerp(TEAL_C, 0.45 + 0.3 * t).lerp(GOLD_C, t * 0.8)
            }),
        Vec3::ZERO,
    );

    let x_label_id = scene.add_tattva(
        Label::new("x", 0.2).with_color(ORANGE_B),
        Vec3::new(3.0, 0.0, 0.0),
    );
    let y_label_id = scene.add_tattva(
        Label::new("y", 0.2).with_color(BLUE_B),
        Vec3::new(0.0, 1.8, 0.0),
    );
    let z_label_id = scene.add_tattva(
        Label::new("z", 0.2).with_color(GOLD_C),
        Vec3::new(0.0, 0.0, 3.0),
    );
    let footer_id = scene.add_tattva(
        Label::new(
            "This mode is for reading structure: the crossings and bends of the grid are the explanation.",
            0.17,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, -3.0, 0.0),
    );

    scene.camera_mut().projection = Projection::Perspective {
        fov_y_rad: 42.0_f32.to_radians(),
        aspect: 16.0 / 9.0,
        near: 0.1,
        far: 100.0,
    };
    scene.camera_mut().position = Vec3::new(-4.6, 2.4, 6.4);
    scene.camera_mut().target = Vec3::new(0.0, 0.0, 0.0);
    for id in [title_id, subtitle_id, footer_id] {
        scene.set_depth_mode(id, DepthMode::Overlay);
    }

    let mut timeline = Timeline::new();
    timeline
        .animate(title_id)
        .at(0.0)
        .for_duration(1.0)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(subtitle_id)
        .at(0.35)
        .for_duration(1.6)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    timeline
        .animate(axes_id)
        .at(1.45)
        .for_duration(0.9)
        .ease(Ease::Linear)
        .appear()
        .spawn();
    timeline
        .animate(x_label_id)
        .at(1.85)
        .for_duration(0.45)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(y_label_id)
        .at(2.0)
        .for_duration(0.45)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(z_label_id)
        .at(2.15)
        .for_duration(0.45)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(surface_id)
        .at(2.35)
        .for_duration(2.5)
        .ease(Ease::InOutCubic)
        .write_surface()
        .spawn();

    timeline
        .animate_camera()
        .at(0.0)
        .for_duration(4.3)
        .ease(Ease::InOutCubic)
        .frame_to(Vec3::new(-2.2, 2.0, 7.4), Vec3::new(0.0, 0.0, 0.0))
        .spawn();
    timeline
        .animate_camera()
        .at(4.3)
        .for_duration(4.0)
        .ease(Ease::InOutCubic)
        .frame_to(Vec3::new(3.0, 1.55, 5.8), Vec3::new(0.0, 0.0, 0.0))
        .spawn();

    timeline
        .animate(footer_id)
        .at(6.25)
        .for_duration(1.5)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    scene.play(timeline)?;

    App::new()?.with_scene(scene).run_app()
}
