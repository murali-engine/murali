use glam::Vec3;
use murali::App;
use murali::colors::*;
use murali::frontend::animation::Ease;
use murali::frontend::collection::text::label::Label;
use murali::frontend::layout::Direction;
use murali::positions::CAMERA_DEFAULT_POS;
use murali::prelude::*;

fn axis(id: &str, label: &str, elements: &[(&str, &str)]) -> TensorAxis {
    TensorAxis::with_elements(id, label, elements.iter().copied())
}

fn main() -> anyhow::Result<()> {
    let activations = TensorSnapshot::try_new(
        "encoder.activations",
        vec![2, 2, 3, 4],
        (0..48)
            .map(|index| ((index as f32 * 0.43).sin() + 1.0) * 0.5)
            .collect(),
        vec![
            axis("batch", "Batch", &[("batch.0", "0"), ("batch.1", "1")]),
            axis("head", "Head", &[("head.0", "0"), ("head.1", "1")]),
            axis(
                "token",
                "Tokens",
                &[("token.0", "AI"), ("token.1", "learns"), ("token.2", "by")],
            ),
            axis(
                "feature",
                "Features",
                &[
                    ("feature.0", "f0"),
                    ("feature.1", "f1"),
                    ("feature.2", "f2"),
                    ("feature.3", "f3"),
                ],
            ),
        ],
    )?;

    let projection = |head: &str| {
        activations.try_project_2d(
            "encoder.head.view",
            "token",
            "feature",
            &TensorSlice::new().at("batch", "batch.0").at("head", head),
        )
    };
    let head_zero = projection("head.0")?;
    let head_one = projection("head.1")?;

    let mut view = TensorView::try_new(head_zero)?;
    view.cell_size = vec2(1.0, 0.72);
    view.label_height = 0.2;
    view.value_height = 0.16;
    view.selection_color = GOLD_A;
    let mut scene = Scene::new();
    let title_id = scene.add_tattva(
        Label::new("Semantic Slices of a Rank-4 Tensor", 0.4).with_color(WHITE),
        Vec3::ZERO,
    );
    scene.to_edge(title_id, Direction::Up, 0.7);
    let shape_id = scene.add_tattva(
        Label::new("[batch, head, token, feature] = [2, 2, 3, 4]", 0.2).with_color(GRAY_B),
        Vec3::new(0.0, 2.75, 0.0),
    );
    let view_id = scene.add_tattva(view, Vec3::new(0.0, -0.1, 0.0));
    let head_zero_id = scene.add_tattva(
        Label::new("batch.0 / head.0", 0.24).with_color(TEAL_A),
        Vec3::new(0.0, 1.8, 0.0),
    );
    let head_one_id = scene.add_tattva(
        Label::new("batch.0 / head.1", 0.24).with_color(GOLD_A),
        Vec3::new(0.0, 1.8, 0.0),
    );
    let note_id = scene.add_tattva(
        Label::new(
            "Token and feature IDs remain stable while the fixed head changes.",
            0.2,
        )
        .with_color(GRAY_A),
        Vec3::new(0.0, -3.1, 0.0),
    );
    for id in [shape_id, view_id, head_zero_id, head_one_id, note_id] {
        scene.hide(id);
    }

    let mut timeline = Timeline::new();
    timeline
        .animate(title_id)
        .at(0.0)
        .for_duration(0.8)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    for id in [shape_id, view_id, head_zero_id] {
        timeline
            .animate(id)
            .at(1.0)
            .for_duration(0.6)
            .appear()
            .spawn();
    }
    timeline
        .animate(head_zero_id)
        .at(3.2)
        .for_duration(0.35)
        .fade_to(0.0)
        .spawn();
    timeline
        .animate(view_id)
        .at(3.2)
        .for_duration(1.2)
        .ease(Ease::InOutSmooth)
        .tensor_to(head_one)
        .spawn();
    timeline
        .animate(head_one_id)
        .at(3.55)
        .for_duration(0.5)
        .appear()
        .spawn();
    timeline
        .animate(note_id)
        .at(4.5)
        .for_duration(0.8)
        .appear()
        .spawn();
    timeline.wait_until(7.0);

    scene.play(timeline)?;
    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    scene.camera_mut().set_view_width(14.0);
    App::new()?.with_scene(scene).run_app()
}
