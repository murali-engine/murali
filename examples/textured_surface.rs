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
use murali::{App, BuiltinTexture, DepthMode, TextureImage};

fn sphere_surface(u: f32, v: f32) -> Vec3 {
    let radius = 1.35;
    let sin_u = u.sin();
    Vec3::new(
        radius * sin_u * v.cos(),
        radius * u.cos(),
        radius * sin_u * v.sin(),
    )
}

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    let title_id = scene.add_tattva(
        Label::new("Textured Surface", 0.38).with_color(WHITE),
        Vec3::new(0.0, 3.0, 0.0),
    );
    let subtitle_id = scene.add_tattva(
        Label::new(
            "Texture mapping becomes easier to understand when one familiar image wraps around one simple surface.",
            0.18,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, 2.45, 0.0),
    );

    let axes_id = scene.add_tattva(
        Axes3D::new((-2.2, 2.2), (-2.2, 2.2), (-2.2, 2.2))
            .with_step(1.0)
            .with_axis_thickness(0.03)
            .with_tick_size(0.11),
        Vec3::ZERO,
    );

    let wire_surface_id = scene.add_tattva(
        ParametricSurface::new(
            (0.0, std::f32::consts::PI),
            (0.0, std::f32::consts::TAU),
            sphere_surface,
        )
        .with_samples(40, 54)
        .with_write_progress(0.0)
        .with_render_mode(SurfaceRenderMode::Wireframe)
        .with_color(TEAL_C),
        Vec3::ZERO,
    );

    let surface_id = scene.add_tattva(
        ParametricSurface::new(
            (0.0, std::f32::consts::PI),
            (0.0, std::f32::consts::TAU),
            sphere_surface,
        )
        .with_samples(40, 54)
        .with_write_progress(1.0)
        .with_texture(TextureImage::builtin(BuiltinTexture::EarthMap)),
        Vec3::ZERO,
    );
    scene.hide(surface_id);

    let x_label_id = scene.add_tattva(
        Label::new("x", 0.2).with_color(ORANGE_B),
        Vec3::new(2.55, 0.0, 0.0),
    );
    let y_label_id = scene.add_tattva(
        Label::new("y", 0.2).with_color(BLUE_B),
        Vec3::new(0.0, 2.55, 0.0),
    );
    let z_label_id = scene.add_tattva(
        Label::new("z", 0.2).with_color(GOLD_C),
        Vec3::new(0.0, 0.0, 2.55),
    );
    let footer_id = scene.add_tattva(
        Label::new(
            "This example is about UV mapping: one image, one surface, and camera angles that reveal the wrap.",
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
    scene.camera_mut().position = Vec3::new(-3.2, 1.9, 5.8);
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
        .for_duration(0.85)
        .ease(Ease::Linear)
        .appear()
        .spawn();
    timeline
        .animate(x_label_id)
        .at(1.8)
        .for_duration(0.45)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(y_label_id)
        .at(1.95)
        .for_duration(0.45)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(z_label_id)
        .at(2.1)
        .for_duration(0.45)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(wire_surface_id)
        .at(2.25)
        .for_duration(2.3)
        .ease(Ease::InOutCubic)
        .write_surface()
        .spawn();
    timeline
        .animate(surface_id)
        .at(4.9)
        .for_duration(1.4)
        .ease(Ease::InOutCubic)
        .appear()
        .spawn();
    timeline
        .animate(wire_surface_id)
        .at(4.9)
        .for_duration(1.4)
        .ease(Ease::InOutCubic)
        .fade_to(0.0)
        .spawn();

    timeline
        .animate_camera()
        .at(0.0)
        .for_duration(4.2)
        .ease(Ease::InOutCubic)
        .frame_to(Vec3::new(-1.2, 1.55, 4.9), Vec3::new(0.0, 0.0, 0.0))
        .spawn();
    timeline
        .animate_camera()
        .at(4.2)
        .for_duration(4.1)
        .ease(Ease::InOutCubic)
        .frame_to(Vec3::new(2.9, 1.1, 4.2), Vec3::new(0.0, 0.0, 0.0))
        .spawn();

    timeline
        .animate(footer_id)
        .at(6.2)
        .for_duration(1.5)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    scene.play(timeline)?;

    App::new()?.with_scene(scene).run_app()
}
