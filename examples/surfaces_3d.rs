use glam::{Vec3, Vec4};
use murali::colors::*;
use murali::engine::camera::Projection;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::animation::Ease;
use murali::frontend::sangrah::composite::axes3d::Axes3D;
use murali::frontend::sangrah::ganit::calculus::parametric_surface::ParametricSurface;
use murali::frontend::sangrah::text::label::Label;
use murali::{App, DepthMode};

fn hill_surface(u: f32, v: f32) -> Vec3 {
    let x = u;
    let ridge = 0.95 * (-(0.38 * (u - 0.55).powi(2) + 0.82 * (v + 0.15).powi(2))).exp();
    let shoulder = 0.48 * (-(1.1 * (u + 0.95).powi(2) + 0.46 * (v - 0.45).powi(2))).exp();
    let ripple = 0.14 * (1.7 * u).sin() * (1.25 * v).cos();
    let basin = 0.10 * (0.55 * u * u + 0.9 * v * v);
    let y = ridge + shoulder + ripple - basin - 0.28;
    let z = v;
    Vec3::new(x, y, z)
}

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    let title_id = scene.add_tattva(
        Label::new("3D Surfaces", 0.38).with_color(WHITE),
        Vec3::new(0.0, 3.0, 0.0),
    );
    let subtitle_id = scene.add_tattva(
        Label::new(
            "A more expressive surface reads best when color and camera movement both reinforce where it rises away from the xz plane.",
            0.18,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, 2.45, 0.0),
    );

    let axes_id = scene.add_tattva(
        Axes3D::new((-2.4, 2.4), (-0.6, 1.4), (-2.2, 2.2))
            .with_step(1.0)
            .with_axis_thickness(0.03)
            .with_tick_size(0.13),
        Vec3::ZERO,
    );

    let surface_id = scene.add_tattva(
        ParametricSurface::new((-2.0, 2.0), (-1.8, 1.8), hill_surface)
            .with_samples(42, 42)
            .with_write_progress(0.0)
            .with_color_fn(move |h| {
                let t = ((h + 0.42) / 1.42).clamp(0.0, 1.0);
                let low = Vec4::new(0.10, 0.48, 0.72, 1.0);
                let mid = Vec4::new(0.24, 0.78, 0.64, 1.0);
                let high = Vec4::new(0.96, 0.76, 0.22, 1.0);
                let peak = Vec4::new(0.95, 0.36, 0.20, 1.0);
                let mut color = if t < 0.55 {
                    low.lerp(mid, t / 0.55)
                } else if t < 0.82 {
                    mid.lerp(high, (t - 0.55) / 0.27)
                } else {
                    high.lerp(peak, (t - 0.82) / 0.18)
                };
                color.w = 0.46;
                color
            }),
        Vec3::ZERO,
    );

    let x_label_id = scene.add_tattva(
        Label::new("x", 0.2).with_color(ORANGE_B),
        Vec3::new(2.85, 0.0, 0.0),
    );
    let y_label_id = scene.add_tattva(
        Label::new("y", 0.2).with_color(BLUE_B),
        Vec3::new(0.0, 1.75, 0.0),
    );
    let z_label_id = scene.add_tattva(
        Label::new("z", 0.2).with_color(GOLD_C),
        Vec3::new(0.0, 0.0, 2.55),
    );
    let footer_id = scene.add_tattva(
        Label::new(
            "Warm color now marks the parts farthest from the xz plane, while the lighter surface lets the axes stay visible through the form.",
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
    scene.camera_mut().position = Vec3::new(-5.6, 2.25, 8.2);
    scene.camera_mut().target = Vec3::new(0.0, 0.24, 0.0);
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
        .for_duration(2.9)
        .ease(Ease::InOutCubic)
        .write_surface()
        .spawn();

    timeline
        .animate_camera()
        .at(0.0)
        .for_duration(4.8)
        .ease(Ease::InOutCubic)
        .frame_to(Vec3::new(-2.6, 2.0, 7.2), Vec3::new(0.0, 0.26, 0.0))
        .spawn();
    timeline
        .animate_camera()
        .at(4.8)
        .for_duration(4.6)
        .ease(Ease::InOutCubic)
        .frame_to(Vec3::new(4.4, 1.45, 5.8), Vec3::new(0.0, 0.32, 0.0))
        .spawn();
    timeline
        .animate_camera()
        .at(9.4)
        .for_duration(4.2)
        .ease(Ease::InOutCubic)
        .frame_to(Vec3::new(0.0, 4.9, 4.8), Vec3::new(0.0, 0.18, 0.0))
        .spawn();

    timeline
        .animate(footer_id)
        .at(10.7)
        .for_duration(1.8)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    scene.play(timeline)?;

    App::new()?.with_scene(scene).run_app()
}
