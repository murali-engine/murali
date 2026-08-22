use glam::Vec3;
use murali::App;
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::engine::timeline::{SignalPlayback, Timeline};
use murali::frontend::animation::Ease;
use murali::frontend::layout::Direction;
use murali::frontend::sangrah::ai::ml_components::SignalFlow;
use murali::frontend::sangrah::ai::transformers_llms::{
    AttentionMatrix, TokenSequence, TransformerBlockDiagram,
};
use murali::frontend::sangrah::text::label::Label;
use murali::positions::CAMERA_DEFAULT_POS;

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    let title_id = scene.add_tattva(
        Label::new("Transformer Attention", 0.38).with_color(WHITE),
        Vec3::ZERO,
    );
    scene.to_edge(title_id, Direction::Up, 0.8);

    let subtitle_id = scene.add_tattva(
        Label::new(
            "Tokens form relationships in an attention map, then continue into one transformer block.",
            0.18,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, 2.9, 0.0),
    );

    let tokens_heading_id = scene.add_tattva(
        Label::new("Token sequence", 0.2).with_color(GRAY_B),
        Vec3::new(-4.6, 2.15, 0.0),
    );
    let matrix_heading_id = scene.add_tattva(
        Label::new("Attention matrix", 0.2).with_color(GRAY_B),
        Vec3::new(-4.6, -2.25, 0.0),
    );
    let block_heading_id = scene.add_tattva(
        Label::new("Semantic transformer block", 0.2).with_color(GRAY_B),
        Vec3::new(3.25, 2.3, 0.0),
    );

    let tokens_id = scene.add_tattva(
        TokenSequence::new(vec!["The", "model", "reads", "context"], 0.24),
        Vec3::new(-4.6, 1.65, 0.0),
    );
    scene.hide(tokens_id);

    let matrix_id = scene.add_tattva(
        AttentionMatrix::new(
            vec![
                vec![1.0, 0.28, 0.12, 0.08],
                vec![0.22, 0.95, 0.33, 0.20],
                vec![0.18, 0.46, 0.88, 0.42],
                vec![0.11, 0.31, 0.57, 0.92],
            ],
            Some(vec![
                "The".into(),
                "model".into(),
                "reads".into(),
                "context".into(),
            ]),
        ),
        Vec3::new(-4.6, -0.65, 0.0),
    );
    scene.hide(matrix_id);

    let mut block = TransformerBlockDiagram::new();
    block.width = 3.6;
    block.block_height = 0.48;
    block.gap = 0.13;
    block.accent_color = TEAL_C;
    block.frame_color = GRAY_A;
    block.input_label = "Input stream".to_string();
    block.output_label = "Output stream".to_string();

    let block_id = scene.add_tattva(block, Vec3::new(3.25, -0.25, 0.0));
    scene.hide(block_id);

    let tokens_to_matrix_flow_id = scene.add_tattva(
        {
            let mut flow =
                SignalFlow::new(vec![Vec3::new(-4.6, 1.42, 0.0), Vec3::new(-4.6, 0.48, 0.0)])
                    .with_progress(0.0)
                    .with_edge_color(GOLD_C)
                    .with_pulse_color(GOLD_A);
            flow.highlight_nodes = false;
            flow.node_radius = 0.0;
            flow.edge_thickness = 0.04;
            flow.pulse_radius = 0.08;
            flow
        },
        Vec3::ZERO,
    );
    scene.hide(tokens_to_matrix_flow_id);

    let matrix_to_block_flow_id = scene.add_tattva(
        {
            let mut flow = SignalFlow::new(vec![
                Vec3::new(-3.45, -0.65, 0.0),
                Vec3::new(1.58, -0.65, 0.0),
            ])
            .with_progress(0.0)
            .with_edge_color(TEAL_C)
            .with_pulse_color(TEAL_A);
            flow.highlight_nodes = false;
            flow.node_radius = 0.0;
            flow.edge_thickness = 0.04;
            flow.pulse_radius = 0.08;
            flow
        },
        Vec3::ZERO,
    );
    scene.hide(matrix_to_block_flow_id);

    let footer_id = scene.add_tattva(
        Label::new(
            "This family is for model structure, token relationships, and visible signal motion.",
            0.17,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, -3.15, 0.0),
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
        .for_duration(1.7)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    timeline
        .animate(tokens_heading_id)
        .at(1.5)
        .for_duration(0.85)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(tokens_id)
        .at(1.9)
        .for_duration(0.3)
        .ease(Ease::Linear)
        .appear()
        .spawn();

    timeline
        .animate(matrix_heading_id)
        .at(2.8)
        .for_duration(0.85)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(matrix_id)
        .at(3.2)
        .for_duration(0.35)
        .ease(Ease::Linear)
        .appear()
        .spawn();
    timeline
        .animate(tokens_to_matrix_flow_id)
        .at(3.55)
        .for_duration(0.2)
        .ease(Ease::Linear)
        .appear()
        .spawn();
    timeline.play_signal(
        tokens_to_matrix_flow_id,
        SignalPlayback::once(3.6, 1.8, Ease::InOutQuad),
    );

    timeline
        .animate(block_heading_id)
        .at(5.5)
        .for_duration(0.85)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(block_id)
        .at(5.9)
        .for_duration(0.35)
        .ease(Ease::Linear)
        .appear()
        .spawn();
    timeline
        .animate(matrix_to_block_flow_id)
        .at(6.2)
        .for_duration(0.2)
        .ease(Ease::Linear)
        .appear()
        .spawn();
    timeline.play_signal(
        matrix_to_block_flow_id,
        SignalPlayback::once(6.3, 1.7, Ease::InOutQuad),
    );

    for (start, stage_id) in [
        (6.4, "self_attention"),
        (7.6, "attention_residual"),
        (8.8, "mlp"),
        (10.0, "mlp_residual"),
    ] {
        timeline
            .animate(block_id)
            .at(start)
            .for_duration(0.45)
            .ease(Ease::InOutQuad)
            .transformer_focus(stage_id)
            .spawn();
    }
    timeline
        .animate(block_id)
        .at(11.1)
        .for_duration(0.45)
        .ease(Ease::InOutQuad)
        .transformer_clear_focus()
        .spawn();

    timeline
        .animate(footer_id)
        .at(10.0)
        .for_duration(1.7)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    timeline.wait_until(12.2);

    scene.play(timeline)?;
    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    scene.camera_mut().set_view_width(17.0);

    App::new()?.with_scene(scene).run_app()
}
