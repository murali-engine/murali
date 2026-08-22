use glam::{Vec3, vec2};
use murali::App;
use murali::colors::*;
use murali::engine::scene::{DrawableProps, Scene};
use murali::engine::timeline::Timeline;
use murali::frontend::TattvaId;
use murali::frontend::animation::Ease;
use murali::frontend::layout::{Anchor, Direction};
use murali::frontend::sangrah::layout::{Group, HStack, VStack};
use murali::frontend::sangrah::primitives::{circle::Circle, square::Square};
use murali::frontend::sangrah::text::label::Label;
use murali::positions::CAMERA_DEFAULT_POS;

fn position_of(scene: &Scene, id: TattvaId) -> Vec3 {
    scene
        .get_tattva_any(id)
        .map(|t| DrawableProps::read(t.props()).position)
        .unwrap_or(Vec3::ZERO)
}

fn capture_positions(scene: &Scene, ids: &[TattvaId]) -> Vec<Vec3> {
    ids.iter().map(|&id| position_of(scene, id)).collect()
}

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    let title_id = scene.add_tattva(
        Label::new("Layout And Groups", 0.38).with_color(WHITE),
        Vec3::new(0.0, 3.0, 0.0),
    );
    let subtitle_id = scene.add_tattva(
        Label::new(
            "Good layout helpers should visibly remove guesswork: align one thing, stack many things, move a whole cluster together.",
            0.18,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, 2.42, 0.0),
    );

    let anchored_heading_id = scene.add_tattva(
        Label::new("Anchor One Label", 0.19).with_color(GRAY_B),
        Vec3::new(-4.8, 1.45, 0.0),
    );
    let anchored_square_id = scene.add_tattva(
        Square::new(1.05, RED_B).with_stroke(0.04, WHITE),
        Vec3::new(-5.0, 0.1, 0.0),
    );
    let anchored_label_id = scene.add_tattva(
        Label::new("aligned by helper", 0.2).with_color(WHITE),
        Vec3::new(-6.2, -0.95, 0.0),
    );
    scene.next_to(anchored_label_id, anchored_square_id, Direction::Up, 0.28);
    scene.align_to(anchored_label_id, anchored_square_id, Anchor::Left);
    let anchored_label_target = position_of(&scene, anchored_label_id);
    scene.set_position_2d(anchored_label_id, vec2(-6.2, -0.95));
    let anchored_caption_id = scene.add_tattva(
        Label::new(
            "Start anywhere. Then `next_to`\nand `align_to` place it cleanly.",
            0.145,
        )
        .with_color(GRAY_A),
        Vec3::new(-4.85, -1.65, 0.0),
    );

    let stack_heading_id = scene.add_tattva(
        Label::new("Resolve A Mess Into Stacks", 0.19).with_color(GRAY_B),
        Vec3::new(0.0, 1.45, 0.0),
    );
    let input_id = scene.add_tattva(Label::new("Input", 0.24).with_color(BLUE_B), Vec3::ZERO);
    let hidden_id = scene.add_tattva(Label::new("Hidden", 0.24).with_color(TEAL_C), Vec3::ZERO);
    let output_id = scene.add_tattva(Label::new("Output", 0.24).with_color(GOLD_C), Vec3::ZERO);
    HStack::new(vec![input_id, hidden_id, output_id], 0.42).apply(&mut scene);
    Group::new(vec![input_id, hidden_id, output_id]).move_to(&mut scene, vec2(-0.7, 0.35));
    let hstack_targets = capture_positions(&scene, &[input_id, hidden_id, output_id]);
    scene.set_position_2d(input_id, vec2(-1.4, 0.95));
    scene.set_position_2d(hidden_id, vec2(0.6, -0.1));
    scene.set_position_2d(output_id, vec2(1.25, 1.0));

    let observe_id = scene.add_tattva(Label::new("Observe", 0.22).with_color(GRAY_A), Vec3::ZERO);
    let reason_id = scene.add_tattva(Label::new("Reason", 0.22).with_color(GRAY_A), Vec3::ZERO);
    let act_id = scene.add_tattva(Label::new("Act", 0.22).with_color(GRAY_A), Vec3::ZERO);
    VStack::new(vec![observe_id, reason_id, act_id], 0.22).apply(&mut scene);
    Group::new(vec![observe_id, reason_id, act_id]).move_to(&mut scene, vec2(3.45, 0.15));
    let vstack_targets = capture_positions(&scene, &[observe_id, reason_id, act_id]);
    scene.set_position_2d(observe_id, vec2(2.55, 0.95));
    scene.set_position_2d(reason_id, vec2(4.15, 0.25));
    scene.set_position_2d(act_id, vec2(2.35, -0.55));

    let stack_caption_id = scene.add_tattva(
        Label::new(
            "`HStack` and `VStack`\nturn clutter into reading order.",
            0.145,
        )
        .with_color(GRAY_A),
        Vec3::new(1.35, -1.65, 0.0),
    );

    let group_heading_id = scene.add_tattva(
        Label::new("Move A Whole Cluster", 0.19).with_color(GRAY_B),
        Vec3::new(0.0, -2.05, 0.0),
    );
    let group_square_id = scene.add_tattva(
        Square::new(0.72, ORANGE_B).with_stroke(0.035, WHITE),
        Vec3::new(-4.45, -3.05, 0.0),
    );
    let group_circle_id = scene.add_tattva(
        Circle::new(0.34, 40, GREEN_D).with_stroke(0.035, WHITE),
        Vec3::new(-3.35, -3.05, 0.0),
    );
    let group_small_square_id = scene.add_tattva(
        Square::new(0.52, PURPLE_B).with_stroke(0.035, WHITE),
        Vec3::new(-2.35, -3.05, 0.0),
    );
    let moving_group = Group::new(vec![
        group_square_id,
        group_circle_id,
        group_small_square_id,
    ]);
    let group_start_positions = capture_positions(
        &scene,
        &[group_square_id, group_circle_id, group_small_square_id],
    );
    moving_group.move_to(&mut scene, vec2(3.15, -3.05));
    let group_targets = capture_positions(
        &scene,
        &[group_square_id, group_circle_id, group_small_square_id],
    );
    for (&id, start) in [group_square_id, group_circle_id, group_small_square_id]
        .iter()
        .zip(group_start_positions.iter())
    {
        scene.set_position_2d(id, start.truncate());
    }
    let group_caption_id = scene.add_tattva(
        Label::new(
            "One `Group.move_to` keeps spacing intact\nwhile the whole cluster travels.",
            0.145,
        )
        .with_color(GRAY_A),
        Vec3::new(0.0, -4.15, 0.0),
    );

    let mut timeline = Timeline::new();
    timeline
        .animate(title_id)
        .at(0.0)
        .for_duration(1.2)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(subtitle_id)
        .at(0.35)
        .for_duration(2.1)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    timeline
        .animate(anchored_heading_id)
        .at(1.8)
        .for_duration(1.0)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(anchored_square_id)
        .at(2.35)
        .for_duration(1.0)
        .ease(Ease::OutCubic)
        .draw()
        .spawn();
    timeline
        .animate(anchored_label_id)
        .at(2.8)
        .for_duration(0.95)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(anchored_label_id)
        .at(3.95)
        .for_duration(1.8)
        .ease(Ease::InOutCubic)
        .move_to(anchored_label_target)
        .spawn();
    timeline
        .animate(anchored_caption_id)
        .at(4.55)
        .for_duration(1.45)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    timeline
        .animate(stack_heading_id)
        .at(5.8)
        .for_duration(1.0)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    for (id, at) in [(input_id, 6.45), (hidden_id, 6.8), (output_id, 7.15)] {
        timeline
            .animate(id)
            .at(at)
            .for_duration(0.75)
            .ease(Ease::Linear)
            .typewrite_text()
            .spawn();
    }
    for (id, at) in [(observe_id, 6.95), (reason_id, 7.3), (act_id, 7.65)] {
        timeline
            .animate(id)
            .at(at)
            .for_duration(0.75)
            .ease(Ease::Linear)
            .typewrite_text()
            .spawn();
    }
    for (id, target) in [input_id, hidden_id, output_id]
        .into_iter()
        .zip(hstack_targets.into_iter())
    {
        timeline
            .animate(id)
            .at(8.7)
            .for_duration(1.8)
            .ease(Ease::InOutCubic)
            .move_to(target)
            .spawn();
    }
    for (id, target) in [observe_id, reason_id, act_id]
        .into_iter()
        .zip(vstack_targets.into_iter())
    {
        timeline
            .animate(id)
            .at(9.2)
            .for_duration(1.8)
            .ease(Ease::InOutCubic)
            .move_to(target)
            .spawn();
    }
    timeline
        .animate(stack_caption_id)
        .at(10.2)
        .for_duration(1.5)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    timeline
        .animate(group_heading_id)
        .at(11.85)
        .for_duration(1.0)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    for (index, id) in [group_square_id, group_circle_id, group_small_square_id]
        .into_iter()
        .enumerate()
    {
        timeline
            .animate(id)
            .at(12.5 + index as f32 * 0.28)
            .for_duration(0.8)
            .ease(Ease::OutCubic)
            .draw()
            .spawn();
    }
    for (id, target) in [group_square_id, group_circle_id, group_small_square_id]
        .into_iter()
        .zip(group_targets.into_iter())
    {
        timeline
            .animate(id)
            .at(13.9)
            .for_duration(2.0)
            .ease(Ease::InOutCubic)
            .move_to(target)
            .spawn();
    }
    timeline
        .animate(group_caption_id)
        .at(14.75)
        .for_duration(1.55)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    scene.play(timeline)?;
    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    scene.camera_mut().set_view_width(16.0);

    App::new()?.with_scene(scene).run_app()
}
