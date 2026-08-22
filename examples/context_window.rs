use glam::Vec3;
use murali::App;
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::animation::Ease;
use murali::frontend::layout::Direction;
use murali::frontend::sangrah::ai::{
    ContextBlock, ContextBlockRole, ContextTruncation, ContextWindow,
};
use murali::frontend::sangrah::text::label::Label;
use murali::positions::CAMERA_DEFAULT_POS;

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();
    let title = scene.add_tattva(
        Label::new("What the model can see", 0.38).with_color(WHITE),
        Vec3::ZERO,
    );
    scene.to_edge(title, Direction::Up, 0.58);

    let context = ContextWindow::try_new(
        vec![
            ContextBlock::new(
                "instructions",
                "Core instructions",
                ContextBlockRole::System,
                620,
            )
            .with_preview("Answer clearly and cite sources"),
            ContextBlock::new(
                "history",
                "Conversation history",
                ContextBlockRole::User,
                4_900,
            )
            .with_preview("Earlier turns and decisions")
            .truncated_to(2_700, ContextTruncation::FromStart),
            ContextBlock::new(
                "retrieval",
                "Retrieved documents",
                ContextBlockRole::Retrieved,
                1_850,
            )
            .with_preview("Three relevant passages"),
            ContextBlock::new("tool", "Tool result", ContextBlockRole::Tool, 760)
                .with_preview("Live weather observations"),
            ContextBlock::new("prompt", "Latest request", ContextBlockRole::User, 410)
                .with_preview("Plan tomorrow's field recording"),
        ],
        8_192,
    )?
    .with_title("ASSEMBLED MODEL CONTEXT");
    let context_id = scene.add_tattva(context, Vec3::new(0.0, -0.15, 0.0));
    scene.hide(context_id);

    let note = scene.add_tattva(
        Label::new(
            "Older history was trimmed from the start; the current request remains intact.",
            0.17,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, -3.25, 0.0),
    );

    let mut timeline = Timeline::new();
    timeline
        .animate(title)
        .at(0.0)
        .for_duration(0.9)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(context_id)
        .at(0.65)
        .for_duration(0.8)
        .ease(Ease::OutCubic)
        .appear()
        .spawn();
    timeline
        .animate(note)
        .at(1.55)
        .for_duration(1.7)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    scene.play(timeline)?;
    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    App::new()?.with_scene(scene).run_app()
}
