use glam::{Quat, Vec3};
use murali::colors::*;
use murali::engine::camera::Projection;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::animation::Ease;
use murali::frontend::collection::primitives::prop3d::Prop3D;
use murali::frontend::collection::text::label::Label;
use murali::{App, DepthMode};

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    let title_id = scene.add_tattva(
        Label::new("Prop3D: glTF", 0.42).with_color(WHITE),
        Vec3::new(0.0, 2.85, 0.0),
    );
    let subtitle_id = scene.add_tattva(
        Label::new(
            "A loose .gltf apple prop loaded with its sibling .bin file, then animated with ordinary transforms.",
            0.18,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, 2.35, 0.0),
    );

    let prop_id = scene.add_tattva(
        Prop3D::from_gltf("assets/props/demo-apple/demo-apple.gltf")?,
        Vec3::new(-0.85, -0.65, 0.0),
    );
    scene.set_scale(prop_id, Vec3::splat(1.65));
    scene.set_rotation(prop_id, Quat::from_rotation_y(-0.35));

    let note_id = scene.add_tattva(
        Label::new(
            "For loose glTF assets, keep the .gltf, .bin, and texture files together.",
            0.17,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, -2.85, 0.0),
    );

    scene.camera_mut().projection = Projection::Perspective {
        fov_y_rad: 44.0_f32.to_radians(),
        aspect: 16.0 / 9.0,
        near: 0.1,
        far: 100.0,
    };
    scene.camera_mut().position = Vec3::new(0.0, 1.25, 7.2);
    scene.camera_mut().target = Vec3::new(0.0, 0.1, 0.0);

    for id in [title_id, subtitle_id, note_id] {
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
        .animate(subtitle_id)
        .at(0.25)
        .for_duration(1.5)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(prop_id)
        .at(0.8)
        .for_duration(0.55)
        .ease(Ease::OutQuad)
        .move_to(Vec3::new(-0.25, 0.12, 0.0))
        .spawn();
    timeline
        .animate(prop_id)
        .at(1.35)
        .for_duration(0.55)
        .ease(Ease::InQuad)
        .move_to(Vec3::new(0.55, -0.65, 0.0))
        .spawn();
    timeline
        .animate(prop_id)
        .at(2.05)
        .for_duration(2.2)
        .ease(Ease::InOutCubic)
        .rotate_to(Quat::from_rotation_y(std::f32::consts::TAU - 0.35))
        .spawn();
    timeline
        .animate(note_id)
        .at(3.0)
        .for_duration(1.5)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    scene.play(timeline)?;
    App::new()?.with_scene(scene).run_app()
}
