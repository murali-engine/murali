use glam::{Vec3, Vec4, vec2};
use murali::App;
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::animation::Ease;
use murali::frontend::layout::Direction;
use murali::frontend::sangrah::composite::beta::{ChatInputBox, ChatInputTipSide};
use murali::frontend::sangrah::text::label::Label;
use murali::positions::CAMERA_DEFAULT_POS;

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    let title_id = scene.add_tattva(
        Label::new("Chat Input Box", 0.38).with_color(WHITE),
        Vec3::ZERO,
    );
    scene.to_edge(title_id, Direction::Up, 0.8);

    let subtitle_id = scene.add_tattva(
        Label::new(
            "A beta composite for prompt-entry and dialogue moments in explainer videos.",
            0.18,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, 2.95, 0.0),
    );

    let user_box = ChatInputBox::new("Why is the sky blue?")
        .with_size(5.8, 0.82)
        .with_tip(ChatInputTipSide::Right, 0.42, 0.28)
        .with_tip_inset(0.72)
        .with_fill(Vec4::new(0.08, 0.11, 0.15, 0.94))
        .with_stroke(0.018, Vec4::new(0.56, 0.72, 0.90, 0.55))
        .with_text_style(0.22, Vec4::new(0.94, 0.97, 1.0, 0.96))
        .with_text_inset(vec2(0.38, 0.0))
        .with_send_button(true)
        .with_send_button_style(0.34, 0.15, BLUE_C);
    let user_ids = user_box.add_to_scene(&mut scene, Vec3::new(0.0, 0.85, 0.0));

    let assistant_box = ChatInputBox::new("The sky appears blue because sunlight is scattered...")
        .with_size(8.1, 0.82)
        .with_tip(ChatInputTipSide::Left, 0.42, 0.28)
        .with_tip_inset(0.72)
        .with_fill(Vec4::new(0.11, 0.14, 0.13, 0.94))
        .with_stroke(0.018, Vec4::new(0.50, 0.76, 0.62, 0.50))
        .with_text_style(0.2, Vec4::new(0.92, 0.98, 0.94, 0.95))
        .with_text_inset(vec2(0.38, 0.0));
    let assistant_ids = assistant_box.add_to_scene(&mut scene, Vec3::new(0.0, -0.45, 0.0));

    let note_id = scene.add_tattva(
        Label::new(
            "Use the returned text id with typewrite_text(); the box itself is ordinary geometry.",
            0.16,
        )
        .with_color(GRAY_A),
        Vec3::new(0.0, -2.3, 0.0),
    );

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

    for id in user_ids.all() {
        if id != user_ids.text {
            timeline
                .animate(id)
                .at(1.25)
                .for_duration(0.55)
                .ease(Ease::OutCubic)
                .appear()
                .spawn();
        }
    }
    timeline
        .animate(user_ids.text)
        .at(1.55)
        .for_duration(1.25)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    for id in assistant_ids.all() {
        if id != assistant_ids.text {
            timeline
                .animate(id)
                .at(3.1)
                .for_duration(0.55)
                .ease(Ease::OutCubic)
                .appear()
                .spawn();
        }
    }
    timeline
        .animate(assistant_ids.text)
        .at(3.4)
        .for_duration(2.1)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    timeline
        .animate(note_id)
        .at(5.65)
        .for_duration(1.8)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    scene.play(timeline)?;

    scene.camera_mut().position = CAMERA_DEFAULT_POS;

    App::new()?.with_scene(scene).run_app()
}
