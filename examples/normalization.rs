use glam::Vec3;
use murali::App;
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::animation::Ease;
use murali::frontend::layout::Direction;
use murali::frontend::sangrah::ai::{
    NormalizationView, TensorAxis, TensorNormalization, TensorSnapshot,
};
use murali::frontend::sangrah::text::label::Label;
use murali::positions::CAMERA_DEFAULT_POS;

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();
    let title = scene.add_tattva(
        Label::new("LayerNorm stabilizes each token", 0.38).with_color(WHITE),
        Vec3::ZERO,
    );
    scene.to_edge(title, Direction::Up, 0.58);

    let residual = TensorSnapshot::try_new(
        "layer.7.residual.input",
        vec![4, 5],
        vec![
            1.0, 2.0, 4.0, 5.0, 8.0, -3.0, -1.0, 0.0, 2.0, 7.0, 0.5, 0.8, 1.4, 2.2, 4.8, -5.0,
            -2.0, 1.0, 4.0, 10.0,
        ],
        vec![
            TensorAxis::with_elements(
                "token",
                "Tokens",
                [
                    ("token.the", "The"),
                    ("token.model", "model"),
                    ("token.learns", "learns"),
                    ("token.patterns", "patterns"),
                ],
            ),
            TensorAxis::new(
                "feature",
                "Residual features",
                vec!["f0", "f1", "f2", "f3", "f4"],
            ),
        ],
    )?;
    let view = NormalizationView::try_new(
        residual,
        "layer.7.attention.normalized",
        "feature",
        TensorNormalization::LayerNorm,
        1e-5,
    )?;
    let view_id = scene.add_tattva(view, Vec3::new(0.0, -0.15, 0.0));
    scene.hide(view_id);

    let note = scene.add_tattva(
        Label::new(
            "Every token row gets its own mean and variance; feature identity is preserved.",
            0.17,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, -3.2, 0.0),
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
        .animate(view_id)
        .at(0.55)
        .for_duration(0.8)
        .ease(Ease::OutCubic)
        .appear()
        .spawn();
    timeline
        .animate(note)
        .at(1.55)
        .for_duration(1.8)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    scene.play(timeline)?;
    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    App::new()?.with_scene(scene).run_app()
}
