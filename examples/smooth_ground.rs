//! Filled dune sheet, the smooth counterpart of `wave_mesh_background`.
//!
//! ```text
//! cargo run --release --example smooth_ground
//! ```

use glam::{Vec3, Vec4};
use murali::App;
use murali::engine::camera::Projection;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::animation::Ease;
use murali::frontend::collection::composite::SmoothGround;
use murali::frontend::collection::text::label::Label;

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();
    let background = Vec4::new(7.0 / 255.0, 10.0 / 255.0, 22.0 / 255.0, 1.0);
    scene.set_background(background);

    let eye = Vec3::new(-22.0, 9.0, 18.0);
    let fov = 32.0_f32.to_radians();
    scene.camera_mut().projection = Projection::Perspective {
        fov_y_rad: fov,
        aspect: 16.0 / 9.0,
        near: 0.1,
        far: 240.0,
    };
    scene.camera_mut().up = Vec3::Y;
    scene.camera_mut().position = eye;
    scene.camera_mut().target = Vec3::new(0.0, -0.4, 0.0);

    let placed_at = Vec3::new(0.0, -2.4, 0.0);
    let ground = scene.add_tattva(
        SmoothGround::new()
            .with_horizon(background)
            .with_eye(eye - placed_at),
        placed_at,
    );

    let title = scene.add_tattva(
        Label::new("A smooth ground", 0.46).with_color(Vec4::new(
            248.0 / 255.0,
            250.0 / 255.0,
            252.0 / 255.0,
            1.0,
        )),
        Vec3::new(0.0, 2.4, 0.0),
    );
    let subtitle = scene.add_tattva(
        Label::new(
            "Filled dunes. The horizon matches the sky, and the sheet keeps its depth.",
            0.2,
        )
        .with_color(Vec4::new(154.0 / 255.0, 167.0 / 255.0, 193.0 / 255.0, 1.0)),
        Vec3::new(0.0, 1.7, 0.0),
    );

    let mut timeline = Timeline::new();
    timeline.call_during_reversible(
        0.0,
        10.0,
        move |scene, progress| {
            let eye = scene.camera().position;
            if let Some(mesh) = scene.get_tattva_typed_mut::<SmoothGround>(ground) {
                mesh.state.phase = progress * 0.7;
                mesh.state.eye = eye - placed_at;
            }
        },
        move |scene| {
            if let Some(mesh) = scene.get_tattva_typed_mut::<SmoothGround>(ground) {
                mesh.state.phase = 0.0;
            }
        },
    );
    for (index, id) in [title, subtitle].into_iter().enumerate() {
        timeline
            .animate(id)
            .at(index as f32 * 0.25)
            .for_duration(0.8 + index as f32 * 0.1)
            .ease(Ease::OutCubic)
            .appear()
            .spawn();
    }
    scene.play(timeline)?;
    App::new()?.with_scene(scene).run_app()
}
