use glam::{Quat, Vec3};
use murali::App;
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::animation::Ease;
use murali::frontend::collection::primitives::{circle::Circle, square::Square};
use murali::frontend::collection::text::label::Label;
use murali::frontend::layout::Direction;
use murali::positions::CAMERA_DEFAULT_POS;

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    let title_id = scene.add_tattva(
        Label::new("Motion Basics", 0.38).with_color(WHITE),
        Vec3::ZERO,
    );
    scene.to_edge(title_id, Direction::Up, 0.8);

    let subtitle_id = scene.add_tattva(
        Label::new(
            "A first pass through movement, scaling, rotation, fading, and easing.",
            0.18,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, 2.95, 0.0),
    );

    let move_heading_id = scene.add_tattva(
        Label::new("Move", 0.2).with_color(GRAY_B),
        Vec3::new(-5.4, 1.8, 0.0),
    );
    let move_shape_id = scene.add_tattva(
        Square::new(0.9, RED_B).with_stroke(0.04, WHITE),
        Vec3::new(-5.4, 0.15, 0.0),
    );
    let move_caption_id = scene.add_tattva(
        Label::new("Ease::InOutQuad", 0.17).with_color(GRAY_A),
        Vec3::new(-5.4, -1.1, 0.0),
    );

    let scale_heading_id = scene.add_tattva(
        Label::new("Scale", 0.2).with_color(GRAY_B),
        Vec3::new(-1.8, 1.8, 0.0),
    );
    let scale_shape_id = scene.add_tattva(
        Circle::new(0.5, 48, TEAL_C).with_stroke(0.04, WHITE),
        Vec3::new(-1.8, 0.15, 0.0),
    );
    let scale_caption_id = scene.add_tattva(
        Label::new("Ease::OutCubic", 0.17).with_color(GRAY_A),
        Vec3::new(-1.8, -1.1, 0.0),
    );

    let rotate_heading_id = scene.add_tattva(
        Label::new("Rotate", 0.2).with_color(GRAY_B),
        Vec3::new(1.8, 1.8, 0.0),
    );
    let rotate_shape_id = scene.add_tattva(
        Square::new(0.95, BLUE_D).with_stroke(0.04, WHITE),
        Vec3::new(1.8, 0.15, 0.0),
    );
    let rotate_caption_id = scene.add_tattva(
        Label::new("Ease::InOutCubic", 0.17).with_color(GRAY_A),
        Vec3::new(1.8, -1.1, 0.0),
    );

    let fade_heading_id = scene.add_tattva(
        Label::new("Fade", 0.2).with_color(GRAY_B),
        Vec3::new(5.4, 1.8, 0.0),
    );
    let fade_shape_id = scene.add_tattva(
        Circle::new(0.5, 48, GOLD_C).with_stroke(0.04, WHITE),
        Vec3::new(5.4, 0.15, 0.0),
    );
    let fade_caption_id = scene.add_tattva(
        Label::new("Ease::Linear", 0.17).with_color(GRAY_A),
        Vec3::new(5.4, -1.1, 0.0),
    );

    let footer_id = scene.add_tattva(
        Label::new(
            "These are the core building blocks before morphs, semantic transforms, or camera choreography.",
            0.17,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, -3.05, 0.0),
    );

    let mut timeline = Timeline::new();
    timeline
        .animate(title_id)
        .at(0.0)
        .for_duration(3.0)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(subtitle_id)
        .at(1.05)
        .for_duration(4.8)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    for (index, heading_id, shape_id, caption_id) in [
        (0usize, move_heading_id, move_shape_id, move_caption_id),
        (1usize, scale_heading_id, scale_shape_id, scale_caption_id),
        (
            2usize,
            rotate_heading_id,
            rotate_shape_id,
            rotate_caption_id,
        ),
        (3usize, fade_heading_id, fade_shape_id, fade_caption_id),
    ] {
        let start = 4.5 + index as f32 * 5.1;
        timeline
            .animate(heading_id)
            .at(start)
            .for_duration(2.55)
            .ease(Ease::Linear)
            .typewrite_text()
            .spawn();
        timeline
            .animate(shape_id)
            .at(start + 1.05)
            .for_duration(3.15)
            .ease(Ease::OutCubic)
            .draw()
            .spawn();
        timeline
            .animate(caption_id)
            .at(start + 1.8)
            .for_duration(2.55)
            .ease(Ease::Linear)
            .typewrite_text()
            .spawn();
    }

    timeline
        .animate(move_shape_id)
        .at(6.6)
        .for_duration(6.0)
        .ease(Ease::InOutQuad)
        .move_to(Vec3::new(-4.4, 0.15, 0.0))
        .spawn();

    timeline
        .animate(scale_shape_id)
        .at(12.8)
        .for_duration(5.8)
        .ease(Ease::InOutCubic)
        .scale_to(Vec3::splat(1.8))
        .spawn();

    timeline
        .animate(rotate_shape_id)
        .at(18.6)
        .for_duration(5.4)
        .ease(Ease::InOutCubic)
        .rotate_to(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2))
        .spawn();

    timeline
        .animate(fade_shape_id)
        .at(24.4)
        .for_duration(4.5)
        .ease(Ease::Linear)
        .fade_to(0.18)
        .spawn();
    timeline
        .animate(fade_shape_id)
        .at(29.2)
        .for_duration(4.05)
        .ease(Ease::Linear)
        .fade_to(1.0)
        .spawn();

    timeline
        .animate(footer_id)
        .at(31.8)
        .for_duration(5.4)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    scene.play(timeline)?;
    scene.camera_mut().position = CAMERA_DEFAULT_POS;

    App::new()?.with_scene(scene).run_app()
}
