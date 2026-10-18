//! Lower-screen animated mesh, the Rust counterpart of murali-js `wave-mesh-background`.
//!
//! ```text
//! cargo run --release --example wave_mesh_background
//! ```

use glam::{Vec3, Vec4};
use murali::App;
use murali::engine::camera::Projection;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::animation::Ease;
use murali::frontend::collection::composite::{WaveMesh, WaveMeshPalette};
use murali::frontend::collection::text::label::Label;

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();
    let background = Vec4::new(9.0 / 255.0, 7.0 / 255.0, 42.0 / 255.0, 1.0);
    scene.set_background(background);

    let eye = Vec3::new(0.0, 9.5, 10.5);
    let fov = 44.0_f32.to_radians();
    scene.camera_mut().projection = Projection::Perspective {
        fov_y_rad: fov,
        aspect: 16.0 / 9.0,
        near: 0.1,
        far: 100.0,
    };
    scene.camera_mut().up = Vec3::Y;
    scene.camera_mut().position = eye;
    scene.camera_mut().target = Vec3::new(0.0, 4.0, -3.5);

    let placed_at = Vec3::new(0.0, -2.35, -5.0);
    let terrain = scene.add_tattva(
        WaveMesh::new()
            .with_size(32.0, 18.0)
            .with_amplitude(1.3)
            .with_samples(81, 45)
            .with_palette(WaveMeshPalette {
                near: hex(0xf0, 0xa9, 0xff),
                far: hex(0x26, 0x37, 0xd4),
                peak: hex(0xff, 0xf5, 0xff),
                fill: hex(0x5a, 0x27, 0xd6),
                sparkle: Vec4::ONE,
            })
            .with_line_opacity(0.9)
            .with_fill_opacity(0.11)
            .with_nodes(0.032, 0.75)
            .with_sparkles(0.085, 0.025)
            .with_far_fade(0.68)
            .with_glow_variation(0.28)
            .with_view(eye - placed_at, fov),
        placed_at,
    );

    let title = scene.add_tattva(
        Label::new("Deterministic ideas, beautifully in motion", 0.42)
            .with_color(hex(0xf8, 0xf5, 0xff)),
        Vec3::new(0.0, 1.65, 0.0),
    );
    let subtitle = scene.add_tattva(
        Label::new(
            "A reusable WaveMesh background for openings, explainers, and transitions",
            0.2,
        )
        .with_color(hex(0xbc, 0xb8, 0xff)),
        Vec3::new(0.0, 0.95, 0.0),
    );

    let mut timeline = Timeline::new();
    timeline.call_during_reversible(
        0.0,
        10.0,
        move |scene, progress| {
            if let Some(mesh) = scene.get_tattva_typed_mut::<WaveMesh>(terrain) {
                mesh.state.phase = progress * 2.0;
            }
        },
        move |scene| {
            if let Some(mesh) = scene.get_tattva_typed_mut::<WaveMesh>(terrain) {
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

fn hex(red: u8, green: u8, blue: u8) -> Vec4 {
    Vec4::new(
        red as f32 / 255.0,
        green as f32 / 255.0,
        blue as f32 / 255.0,
        1.0,
    )
}
