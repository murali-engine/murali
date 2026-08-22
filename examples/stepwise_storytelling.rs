use glam::Vec3;
use murali::App;
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::animation::Ease;
use murali::frontend::sangrah::storytelling::stepwise::{
    Stepwise, StepwiseLayout, StepwiseStyle, model::Direction,
};
use murali::frontend::sangrah::text::label::Label;
use murali::positions::CAMERA_DEFAULT_POS;

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    let title_id = scene.add_tattva(
        Label::new("Stepwise Storytelling", 0.38).with_color(WHITE),
        Vec3::new(0.0, 3.0, 0.0),
    );
    let subtitle_id = scene.add_tattva(
        Label::new(
            "A narrative flow can reveal step by step, then replay the journey through the same path.",
            0.18,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, 2.45, 0.0),
    );

    let heading_id = scene.add_tattva(
        Label::new("Observe -> Reason -> Revise -> Publish", 0.19).with_color(GRAY_B),
        Vec3::new(0.0, 1.7, 0.0),
    );

    let stepwise_id = scene.add_tattva(
        Stepwise::from_script(|s| {
            let observe = s.step("Observe");
            let reason = s.step("Reason");
            let revise = s.step("Revise");
            let publish = s.step("Publish");

            s.connect(observe, reason);
            s.connect(reason, revise);
            s.connect(revise, publish);
            s.connect(revise, reason)
                .route(vec![Direction::Down, Direction::Left]);

            s.with_sequence(vec![observe, reason, revise, reason, revise, publish]);
        })
        .with_layout(StepwiseLayout::horizontal(1.45))
        .with_style({
            let mut style = StepwiseStyle::default();
            style.signal_color = TEAL_C;
            style
        })
        .with_reveal_progress(0.0)
        .with_signal_progress(0.0),
        Vec3::new(-5.0, 0.1, 0.0),
    );

    let caption_id = scene.add_tattva(
        Label::new(
            "The feedback loop is part of the story, not a separate diagram.",
            0.17,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, -2.5, 0.0),
    );
    let footer_id = scene.add_tattva(
        Label::new(
            "Use this when the audience should feel the sequence, not just see the final structure.",
            0.17,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, -3.05, 0.0),
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
        .at(1.4)
        .for_duration(0.9)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(stepwise_id)
        .at(1.9)
        .for_duration(2.8)
        .ease(Ease::InOutQuad)
        .propagate_to(1.0)
        .spawn();
    timeline
        .animate(caption_id)
        .at(2.5)
        .for_duration(1.1)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(stepwise_id)
        .at(5.0)
        .for_duration(3.0)
        .ease(Ease::Linear)
        .signal_to(1.0)
        .spawn();
    timeline
        .animate(footer_id)
        .at(6.1)
        .for_duration(1.6)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    scene.play(timeline)?;
    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    scene.camera_mut().set_view_width(16.0);

    App::new()?.with_scene(scene).run_app()
}
