use glam::{Vec3, Vec4};
use murali::App;
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::animation::Ease;
use murali::frontend::collection::primitives::{circle::Circle, particle_belt::ParticleBelt};
use murali::frontend::collection::text::label::Label;
use murali::positions::CAMERA_DEFAULT_POS;

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    let title_id = scene.add_tattva(
        Label::new("Particles", 0.38).with_color(WHITE),
        Vec3::new(0.0, 3.0, 0.0),
    );
    let subtitle_id = scene.add_tattva(
        Label::new(
            "A particle system reads best when the motion is simple enough for the eye to follow.",
            0.18,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, 2.45, 0.0),
    );

    let heading_id = scene.add_tattva(
        Label::new("Orbital Belt", 0.19).with_color(GRAY_B),
        Vec3::new(0.0, 1.7, 0.0),
    );

    // let guide_ring_id = scene.add_tattva(
    //     Circle::new(2.25, 96, Vec4::new(0.0, 0.0, 0.0, 0.0))
    //         .with_stroke(0.03, Vec4::new(0.42, 0.84, 0.98, 0.20)),
    //     Vec3::new(0.0, -0.15, 0.0),
    // );

    let core_id = scene.add_tattva(
        Circle::new(0.26, 48, Vec4::new(1.0, 0.84, 0.40, 0.95))
            .with_stroke(0.03, Vec4::new(1.0, 0.95, 0.72, 0.65)),
        Vec3::new(0.0, -0.15, 0.0),
    );

    let belt_id = scene.add_tattva(
        ParticleBelt::new(2.25)
            .with_band_width(0.68)
            .with_particle_count(220)
            .with_particle_size_range(0.016, 0.055)
            .with_palette(vec![TEAL_C, BLUE_B, PURPLE_B, GOLD_C])
            .with_orbit_speed(1.0)
            .with_clockwise_ratio(0.35)
            .with_band_breathing(0.10, 1.3)
            .with_radial_jitter(0.13, 2.7)
            .with_seed(7.0),
        Vec3::new(0.0, -0.15, 0.0),
    );

    let caption_id = scene.add_tattva(
        Label::new(
            "One evolving belt is enough to teach density, drift, and cinematic texture without turning into visual noise.",
            0.16,
        )
        .with_color(GRAY_A),
        Vec3::new(0.0, -2.95, 0.0),
    );

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
        .for_duration(1.5)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(heading_id)
        .at(1.15)
        .for_duration(0.8)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    // timeline
    //     .animate(guide_ring_id)
    //     .at(1.55)
    //     .for_duration(0.9)
    //     .ease(Ease::OutCubic)
    //     .draw()
    //     .spawn();
    timeline
        .animate(core_id)
        .at(1.85)
        .for_duration(0.55)
        .ease(Ease::OutCubic)
        .appear()
        .spawn();
    timeline
        .animate(belt_id)
        .at(2.0)
        .for_duration(0.65)
        .ease(Ease::OutCubic)
        .appear()
        .spawn();
    timeline
        .animate(caption_id)
        .at(2.45)
        .for_duration(1.2)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(belt_id)
        .at(2.6)
        .for_duration(5.6)
        .ease(Ease::Linear)
        .belt_evolve_with_speed(1.05)
        .spawn();
    timeline
        .animate_camera()
        .at(2.2)
        .for_duration(3.0)
        .ease(Ease::InOutQuad)
        .zoom_to(1.12)
        .spawn();
    timeline
        .animate_camera()
        .at(5.4)
        .for_duration(2.4)
        .ease(Ease::InOutQuad)
        .zoom_to(1.28)
        .spawn();

    scene.play(timeline)?;
    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    scene.camera_mut().set_view_width(16.0);

    App::new()?.with_scene(scene).run_app()
}
