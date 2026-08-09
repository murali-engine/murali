use glam::{Vec2, Vec3, vec2};
use murali::App;
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::DirtyFlags;
use murali::frontend::animation::Ease;
use murali::frontend::collection::graph::vector_field::VectorField;
use murali::frontend::collection::primitives::circle::Circle;
use murali::frontend::collection::text::label::Label;
use murali::frontend::layout::Direction;
use murali::positions::CAMERA_DEFAULT_POS;
use std::sync::{Arc, Mutex};

fn charge_positions(t: f32) -> (Vec2, Vec2) {
    let positive = vec2(-0.9 + 0.75 * (0.9 * t).sin(), 0.45 * (0.6 * t).cos());
    let negative = vec2(0.9 + 0.55 * (0.7 * t + 1.2).cos(), -0.4 * (0.8 * t).sin());
    (positive, negative)
}

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    let title_id = scene.add_tattva(
        Label::new("Force Fields", 0.38).with_color(WHITE),
        Vec3::ZERO,
    );
    scene.to_edge(title_id, Direction::Up, 0.8);

    let subtitle_id = scene.add_tattva(
        Label::new(
            "Move the charges and the field responds continuously, instead of staying a fixed picture.",
            0.18,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, 3.05, 0.0),
    );

    let heading_id = scene.add_tattva(
        Label::new("Live charge motion", 0.2).with_color(GRAY_B),
        Vec3::new(0.0, 2.4, 0.0),
    );

    let shared_positions = Arc::new(Mutex::new(charge_positions(0.0)));
    let field_positions = Arc::clone(&shared_positions);
    let field_id = scene.add_tattva(
        VectorField::new((-3.4, 3.4), (-2.0, 2.0), 15, 9, move |pos| {
            let (positive, negative) = *field_positions.lock().expect("charge positions");

            let to_positive = pos - positive;
            let to_negative = pos - negative;

            let dist_pos = to_positive.length().max(0.24);
            let dist_neg = to_negative.length().max(0.24);

            let force_pos = to_positive.normalize() / (dist_pos * dist_pos);
            let force_neg = -to_negative.normalize() / (dist_neg * dist_neg);

            (force_pos + force_neg) * 0.6
        })
        .with_color(BLUE_B)
        .with_length_scale(0.34)
        .with_arrow_style(0.02, 0.08, 0.06),
        Vec3::new(0.0, -0.05, 0.0),
    );

    let (initial_positive, initial_negative) = charge_positions(0.0);
    let positive_charge_id = scene.add_tattva(
        Circle::new(0.16, 28, RED_B).with_stroke(0.03, WHITE),
        Vec3::new(initial_positive.x, initial_positive.y - 0.05, 0.0),
    );
    let negative_charge_id = scene.add_tattva(
        Circle::new(0.16, 28, TEAL_C).with_stroke(0.03, WHITE),
        Vec3::new(initial_negative.x, initial_negative.y - 0.05, 0.0),
    );
    let positive_label_id = scene.add_tattva(
        Label::new("+", 0.18).with_color(WHITE),
        Vec3::new(initial_positive.x, initial_positive.y - 0.05, 0.0),
    );
    let negative_label_id = scene.add_tattva(
        Label::new("-", 0.18).with_color(WHITE),
        Vec3::new(initial_negative.x, initial_negative.y - 0.05, 0.0),
    );

    let footer_id = scene.add_tattva(
        Label::new(
            "This is the right mental model for force fields: charges move, and the arrows update with them.",
            0.17,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, -3.1, 0.0),
    );

    let updater_positions = Arc::clone(&shared_positions);
    scene.add_updater(field_id, move |scene, _, _dt| {
        let (positive, negative) = charge_positions(scene.scene_time);
        *updater_positions.lock().expect("charge positions") = (positive, negative);

        scene.set_position_2d(positive_charge_id, positive + vec2(0.0, -0.05));
        scene.set_position_2d(negative_charge_id, negative + vec2(0.0, -0.05));
        scene.set_position_2d(positive_label_id, positive + vec2(0.0, -0.05));
        scene.set_position_2d(negative_label_id, negative + vec2(0.0, -0.05));

        if let Some(field) = scene.get_tattva_typed_mut::<VectorField>(field_id) {
            field.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
        }
    });

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
        .at(1.4)
        .for_duration(0.8)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(field_id)
        .at(1.8)
        .for_duration(1.0)
        .ease(Ease::Linear)
        .appear()
        .spawn();
    timeline
        .animate(positive_charge_id)
        .at(2.0)
        .for_duration(0.5)
        .ease(Ease::Linear)
        .appear()
        .spawn();
    timeline
        .animate(negative_charge_id)
        .at(2.1)
        .for_duration(0.5)
        .ease(Ease::Linear)
        .appear()
        .spawn();
    timeline
        .animate(positive_label_id)
        .at(2.15)
        .for_duration(0.4)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(negative_label_id)
        .at(2.25)
        .for_duration(0.4)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(footer_id)
        .at(3.0)
        .for_duration(1.8)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    scene.play(timeline)?;
    scene.camera_mut().position = CAMERA_DEFAULT_POS;

    App::new()?.with_scene(scene).run_app()
}
