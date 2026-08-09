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

fn tensor_view(snapshot: TensorSnapshot) -> anyhow::Result<TensorView> {
    let mut view = TensorView::try_new(snapshot)?;
    view.cell_size = vec2(0.9, 0.66);
    view.label_height = 0.18;
    view.value_height = 0.16;
    view.selection_color = GOLD_A;
    Ok(view)
}

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    let title_id = scene.add_tattva(
        Label::new("Semantic Tensor Operations", 0.4).with_color(WHITE),
        Vec3::ZERO,
    );
    scene.to_edge(title_id, Direction::Up, 0.75);

    let subtitle_id = scene.add_tattva(
        Label::new(
            "Named axes keep model meaning intact through broadcasting, splitting, and reshaping.",
            0.17,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, 2.75, 0.0),
    );

    let token_axis = axis(
        "token",
        "Tokens",
        &[("token.0", "AI"), ("token.1", "learns")],
    );
    let feature_axis = axis(
        "feature",
        "Features",
        &[
            ("feature.0", "x0"),
            ("feature.1", "x1"),
            ("feature.2", "x2"),
            ("feature.3", "x3"),
        ],
    );
    let activations = TensorSnapshot::try_new(
        "activations",
        vec![2, 4],
        vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0],
        vec![token_axis, feature_axis],
    )?;

    // The bias is intentionally stored out of feature order. Elementwise alignment uses IDs.
    let bias = TensorSnapshot::try_new(
        "bias",
        vec![4],
        vec![0.4, 0.1, 0.3, 0.2],
        vec![axis(
            "feature",
            "Bias",
            &[
                ("feature.3", "x3"),
                ("feature.0", "x0"),
                ("feature.2", "x2"),
                ("feature.1", "x1"),
            ],
        )],
    )?;
    let biased = activations.try_elementwise(&bias, "activations", TensorElementwiseOp::Add)?;
    let split = biased.try_split("feature", &[2, 2], &["features.left", "features.right"])?;
    let merged = TensorSnapshot::try_merge(&split, "feature", "features.merged")?;
    let reshaped = merged.try_reshape(
        "heads",
        vec![
            axis(
                "head_token",
                "Head / token",
                &[
                    ("head.0.token.0", "h0 / AI"),
                    ("head.0.token.1", "h0 / learns"),
                    ("head.1.token.0", "h1 / AI"),
                    ("head.1.token.1", "h1 / learns"),
                ],
            ),
            axis(
                "channel",
                "Channels",
                &[("channel.0", "c0"), ("channel.1", "c1")],
            ),
        ],
    )?;

    let source_id = scene.add_tattva(tensor_view(activations)?, Vec3::new(0.0, -0.1, 0.0));
    let left_id = scene.add_tattva(tensor_view(split[0].clone())?, Vec3::new(-2.65, -0.1, 0.0));
    let right_id = scene.add_tattva(tensor_view(split[1].clone())?, Vec3::new(2.65, -0.1, 0.0));
    let reshaped_id = scene.add_tattva(tensor_view(reshaped)?, Vec3::new(0.0, -0.15, 0.0));

    let broadcast_caption_id = scene.add_tattva(
        Label::new("1. Reordered feature bias broadcasts across tokens", 0.2).with_color(TEAL_A),
        Vec3::new(0.0, 2.05, 0.0),
    );
    let split_caption_id = scene.add_tattva(
        Label::new("2. Split feature IDs into two tensors", 0.2).with_color(GOLD_A),
        Vec3::new(0.0, 2.05, 0.0),
    );
    let reshape_caption_id = scene.add_tattva(
        Label::new("3. Merge losslessly, then reshape onto explicit axes", 0.2).with_color(PINK),
        Vec3::new(0.0, 2.05, 0.0),
    );
    let left_label_id = scene.add_tattva(
        Label::new("features.left", 0.17).with_color(GRAY_B),
        Vec3::new(-2.65, -1.65, 0.0),
    );
    let right_label_id = scene.add_tattva(
        Label::new("features.right", 0.17).with_color(GRAY_B),
        Vec3::new(2.65, -1.65, 0.0),
    );

    for id in [
        source_id,
        left_id,
        right_id,
        reshaped_id,
        broadcast_caption_id,
        split_caption_id,
        reshape_caption_id,
        left_label_id,
        right_label_id,
    ] {
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
    timeline
        .animate(subtitle_id)
        .at(0.25)
        .for_duration(1.4)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    for id in [source_id, broadcast_caption_id] {
        timeline
            .animate(id)
            .at(1.3)
            .for_duration(0.5)
            .ease(Ease::InOutQuad)
            .appear()
            .spawn();
    }
    timeline
        .animate(source_id)
        .at(2.4)
        .for_duration(1.1)
        .ease(Ease::InOutSmooth)
        .tensor_to(biased)
        .spawn();
    for id in [source_id, broadcast_caption_id] {
        timeline
            .animate(id)
            .at(4.1)
            .for_duration(0.4)
            .fade_to(0.0)
            .spawn();
    }
    for id in [
        left_id,
        right_id,
        split_caption_id,
        left_label_id,
        right_label_id,
    ] {
        timeline
            .animate(id)
            .at(4.5)
            .for_duration(0.5)
            .ease(Ease::InOutQuad)
            .appear()
            .spawn();
    }
    for id in [
        left_id,
        right_id,
        split_caption_id,
        left_label_id,
        right_label_id,
    ] {
        timeline
            .animate(id)
            .at(6.4)
            .for_duration(0.4)
            .fade_to(0.0)
            .spawn();
    }
    for id in [reshaped_id, reshape_caption_id] {
        timeline
            .animate(id)
            .at(6.8)
            .for_duration(0.55)
            .ease(Ease::InOutQuad)
            .appear()
            .spawn();
    }
    timeline.wait_until(9.0);

    scene.play(timeline)?;
    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    scene.camera_mut().set_view_width(14.0);

    App::new()?.with_scene(scene).run_app()
}
