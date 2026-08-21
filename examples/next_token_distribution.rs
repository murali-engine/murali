use glam::Vec3;
use murali::App;
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::animation::Ease;
use murali::frontend::collection::ai::{
    NextTokenDistribution, NextTokenSampling, TensorAxis, TensorSnapshot,
};
use murali::frontend::collection::text::label::Label;
use murali::frontend::layout::Direction;
use murali::positions::CAMERA_DEFAULT_POS;

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();
    let title = scene.add_tattva(
        Label::new("How the next token is chosen", 0.38).with_color(WHITE),
        Vec3::ZERO,
    );
    scene.to_edge(title, Direction::Up, 0.58);

    let logits = TensorSnapshot::try_new(
        "decoder.step.12.logits",
        vec![7],
        vec![2.8, 2.25, 1.7, 1.05, 0.4, -0.1, -0.8],
        vec![TensorAxis::with_elements(
            "vocabulary",
            "Candidate tokens",
            [
                ("token.scattered", "scattered"),
                ("token.blue", "blue"),
                ("token.across", "across"),
                ("token.through", "through"),
                ("token.softly", "softly"),
                ("token.above", "above"),
                ("token.dark", "dark"),
            ],
        )],
    )?;
    let distribution = NextTokenDistribution::try_from_logits(
        &logits,
        "vocabulary",
        NextTokenSampling::new(0.61)
            .with_temperature(0.85)
            .with_top_k(5)
            .with_top_p(0.90),
    )?;
    let selected = distribution.selected().token.clone();
    let distribution_id = scene.add_tattva(distribution, Vec3::new(0.0, -0.2, 0.0));
    scene.hide(distribution_id);

    let sentence = scene.add_tattva(
        Label::new(format!("The light was  +  {selected}"), 0.23).with_color(GOLD_A),
        Vec3::new(0.0, -3.28, 0.0),
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
        .animate(distribution_id)
        .at(0.55)
        .for_duration(0.75)
        .ease(Ease::OutCubic)
        .appear()
        .spawn();
    timeline
        .animate(sentence)
        .at(1.45)
        .for_duration(1.5)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    scene.play(timeline)?;
    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    App::new()?.with_scene(scene).run_app()
}
