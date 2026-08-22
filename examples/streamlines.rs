use glam::{Vec2, Vec3, Vec4, vec2};
use murali::App;
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::DirtyFlags;
use murali::frontend::animation::Ease;
use murali::frontend::collection::maths::calculus::stream_lines::{
    StreamLines, circle_start_points,
};
use murali::frontend::collection::primitives::circle::Circle;
use murali::frontend::collection::text::label::Label;
use murali::positions::CAMERA_DEFAULT_POS;

fn vortex_with_drift(pos: Vec2) -> Vec2 {
    let swirl = vec2(-pos.y, pos.x) * 0.65;
    let inward = -pos * 0.18;
    let drift = vec2(0.42, 0.0);
    swirl + inward + drift
}

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    let title_id = scene.add_tattva(
        Label::new("Streamlines", 0.38).with_color(WHITE),
        Vec3::new(0.0, 3.5, 0.0),
    );
    let subtitle_id = scene.add_tattva(
        Label::new(
            "Streamlines answer a different question than arrows: not the field at a point, but the path a particle would follow.",
            0.18,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, 3.0, 0.0),
    );

    let heading_id = scene.add_tattva(
        Label::new("Seeded flow paths", 0.19).with_color(GRAY_B),
        Vec3::new(0.0, 2.5, 0.0),
    );

    let seed_ring_id = scene.add_tattva(
        Circle::new(1.15, 72, Vec4::new(0.0, 0.0, 0.0, 0.0))
            .with_stroke(0.03, Vec4::new(0.42, 0.84, 0.98, 0.18)),
        Vec3::new(-1.8, -0.15, 0.0),
    );
    let seed_dot_id = scene.add_tattva(
        Circle::new(0.07, 24, TEAL_C).with_stroke(0.02, WHITE),
        Vec3::new(-0.65, -0.15, 0.0),
    );

    let streams_id = scene.add_tattva(
        StreamLines::new(
            circle_start_points(vec2(-1.8, -0.15), 1.15, 14)?,
            vortex_with_drift,
        )
        .with_color(Vec4::new(GOLD_C.x, GOLD_C.y, GOLD_C.z, 0.0))
        .with_thickness(0.035)
        .with_step_size(0.07)
        .with_max_steps(1)
        .with_bounds(vec2(-4.8, -2.2), vec2(4.8, 2.0)),
        Vec3::ZERO,
    );

    let caption_id = scene.add_tattva(
        Label::new(
            "Start points matter: change the seeds and you change the story the flow tells.",
            0.16,
        )
        .with_color(GRAY_A),
        Vec3::new(0.0, -3.0, 0.0),
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
        .for_duration(1.6)
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
    timeline
        .animate(seed_ring_id)
        .at(1.55)
        .for_duration(0.85)
        .ease(Ease::OutCubic)
        .draw()
        .spawn();
    timeline
        .animate(seed_dot_id)
        .at(1.95)
        .for_duration(0.45)
        .ease(Ease::OutCubic)
        .appear()
        .spawn();
    timeline
        .animate(streams_id)
        .at(2.0)
        .for_duration(0.2)
        .ease(Ease::Linear)
        .appear()
        .spawn();
    timeline
        .animate(caption_id)
        .at(2.7)
        .for_duration(1.2)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    timeline.call_during(2.0, 2.8, move |scene, t| {
        if let Some(streams) = scene.get_tattva_typed_mut::<StreamLines>(streams_id) {
            let k = Ease::InOutQuad.eval(t);
            streams.state.max_steps = (1.0 + k * 169.0).round() as usize;
            streams.state.color.w = k;
            streams.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    });

    timeline
        .animate_camera()
        .at(2.4)
        .for_duration(2.8)
        .ease(Ease::InOutQuad)
        .zoom_to(1.12)
        .spawn();

    scene.play(timeline)?;
    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    scene.camera_mut().set_view_width(16.0);

    App::new()?.with_scene(scene).run_app()
}
