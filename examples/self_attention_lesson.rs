use glam::Vec3;
use murali::App;
use murali::colors::*;
use murali::frontend::animation::Ease;
use murali::frontend::layout::Direction;
use murali::frontend::sangrah::text::label::Label;
use murali::positions::CAMERA_DEFAULT_POS;
use murali::prelude::*;

fn view(snapshot: TensorSnapshot, cell_size: glam::Vec2) -> anyhow::Result<TensorView> {
    let mut view = TensorView::try_new(snapshot)?;
    view.cell_size = cell_size;
    view.label_height = 0.16;
    view.value_height = 0.14;
    view.selection_color = GOLD_A;
    Ok(view)
}

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();
    let trace = AiTrace::from_json_str(include_str!("data/self_attention_trace.json"))?;

    let title_id = scene.add_tattva(
        Label::new("A Complete Self-Attention Step", 0.4).with_color(WHITE),
        Vec3::ZERO,
    );
    scene.to_edge(title_id, Direction::Up, 0.7);
    let subtitle_id = scene.add_tattva(
        Label::new(
            "Every displayed value is computed from the same semantic tensor snapshots.",
            0.18,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, 3.05, 0.0),
    );

    let query_axis = trace.token_axis("query", "Tokens");
    let embeddings = trace.require_tensor("embedding.query")?.clone();
    let key_embeddings = trace.require_tensor("embedding.key")?.clone();
    let wq = trace.require_tensor("weights.q")?.clone();
    let wk = trace.require_tensor("weights.k")?.clone();
    let wv = trace.require_tensor("weights.v")?.clone();
    let output_weights = trace.require_tensor("weights.output")?.clone();
    let head_feature_axis = wq.axes[1].clone();
    let vocabulary_axis = output_weights.axes[1].clone();

    let queries = embeddings.try_matmul(&wq, "attention.q")?;
    let keys = key_embeddings.try_matmul(&wk, "attention.k")?;
    let values = key_embeddings.try_matmul(&wv, "attention.v")?;
    let scores = queries.try_matmul(
        &keys.try_transpose_2d("attention.k.transpose")?,
        "attention.weights",
    )?;
    let scaled = scores.try_scaled((queries.shape[1] as f32).sqrt())?;
    let masked = scaled.try_causal_masked(-20.0)?;
    let attention = masked.try_softmax("key")?;
    let context = attention.try_matmul(&values, "attention.context")?;
    let residual_input = embeddings.try_reshape(
        "residual.input",
        vec![query_axis.clone(), head_feature_axis.clone()],
    )?;
    let residual =
        residual_input.try_elementwise(&context, "residual.output", TensorElementwiseOp::Add)?;

    let logits = residual.try_matmul(&output_weights, "next_token")?;
    let probabilities = logits.try_softmax("vocabulary")?;
    let samples = probabilities.try_sample_categorical("vocabulary", &[0.15, 0.55, 0.78])?;
    let final_sample = samples.last().expect("three query slices");
    let sampled_vocabulary_id = &final_sample.element_id.coordinates[1].element_id;
    let sampled_index = vocabulary_axis
        .element_ids
        .iter()
        .position(|id| id == sampled_vocabulary_id)
        .expect("sampled vocabulary element exists");
    let sampled_token = &vocabulary_axis.element_labels[sampled_index];

    let tokens_id = scene.add_tattva(
        TokenSequence::try_from_axis(&query_axis, 0.22)?,
        Vec3::new(0.0, 2.45, 0.0),
    );
    let embeddings_id = scene.add_tattva(
        view(embeddings, vec2(0.8, 0.56))?,
        Vec3::new(0.0, -0.2, 0.0),
    );
    let q_id = scene.add_tattva(view(queries, vec2(0.68, 0.52))?, Vec3::new(-4.4, -0.2, 0.0));
    let k_id = scene.add_tattva(view(keys, vec2(0.68, 0.52))?, Vec3::new(0.0, -0.2, 0.0));
    let v_id = scene.add_tattva(view(values, vec2(0.68, 0.52))?, Vec3::new(4.4, -0.2, 0.0));
    let attention_id = scene.add_tattva(view(scores, vec2(0.82, 0.56))?, Vec3::new(0.0, -0.2, 0.0));
    let context_id = scene.add_tattva(view(context, vec2(0.75, 0.54))?, Vec3::new(-2.5, -0.2, 0.0));
    let residual_id =
        scene.add_tattva(view(residual, vec2(0.75, 0.54))?, Vec3::new(2.5, -0.2, 0.0));
    let output_id = scene.add_tattva(view(logits, vec2(0.92, 0.56))?, Vec3::new(0.0, -0.2, 0.0));

    let stage_texts = [
        "1. Token IDs anchor the embedding rows",
        "2. Learned projections produce Q, K, and V",
        "3. QK^T -> scale -> causal mask -> softmax",
        "4. Attention @ V, then add the residual stream",
        "5. Output projection -> softmax -> categorical sample",
    ];
    let stage_colors = [TEAL_A, BLUE_A, GOLD_A, GREEN_A, PINK];
    let mut stage_ids = Vec::new();
    for (text, color) in stage_texts.into_iter().zip(stage_colors) {
        stage_ids.push(scene.add_tattva(
            Label::new(text, 0.2).with_color(color),
            Vec3::new(0.0, 1.65, 0.0),
        ));
    }
    let q_label_id = scene.add_tattva(
        Label::new("Queries", 0.18).with_color(BLUE_A),
        Vec3::new(-4.4, -2.35, 0.0),
    );
    let k_label_id = scene.add_tattva(
        Label::new("Keys", 0.18).with_color(BLUE_A),
        Vec3::new(0.0, -2.35, 0.0),
    );
    let v_label_id = scene.add_tattva(
        Label::new("Values", 0.18).with_color(BLUE_A),
        Vec3::new(4.4, -2.35, 0.0),
    );
    let context_label_id = scene.add_tattva(
        Label::new("Context", 0.18).with_color(GREEN_A),
        Vec3::new(-2.5, -2.35, 0.0),
    );
    let residual_label_id = scene.add_tattva(
        Label::new("Context + residual", 0.18).with_color(GREEN_A),
        Vec3::new(2.5, -2.35, 0.0),
    );
    let sample_id = scene.add_tattva(
        Label::new(
            format!(
                "u = 0.78 selects '{sampled_token}' with p = {:.3}",
                final_sample.probability
            ),
            0.22,
        )
        .with_color(GOLD_A),
        Vec3::new(0.0, -3.2, 0.0),
    );

    let all_staged_ids = [
        tokens_id,
        embeddings_id,
        q_id,
        k_id,
        v_id,
        attention_id,
        context_id,
        residual_id,
        output_id,
        q_label_id,
        k_label_id,
        v_label_id,
        context_label_id,
        residual_label_id,
        sample_id,
    ];
    for id in all_staged_ids.into_iter().chain(stage_ids.iter().copied()) {
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
    timeline
        .animate(tokens_id)
        .at(1.2)
        .for_duration(0.45)
        .appear()
        .spawn();

    for id in [embeddings_id, stage_ids[0]] {
        timeline
            .animate(id)
            .at(2.0)
            .for_duration(0.45)
            .appear()
            .spawn();
        timeline
            .animate(id)
            .at(4.0)
            .for_duration(0.4)
            .fade_to(0.0)
            .spawn();
    }
    for id in [
        q_id,
        k_id,
        v_id,
        q_label_id,
        k_label_id,
        v_label_id,
        stage_ids[1],
    ] {
        timeline
            .animate(id)
            .at(4.4)
            .for_duration(0.45)
            .appear()
            .spawn();
        timeline
            .animate(id)
            .at(7.0)
            .for_duration(0.4)
            .fade_to(0.0)
            .spawn();
    }
    for id in [attention_id, stage_ids[2]] {
        timeline
            .animate(id)
            .at(7.4)
            .for_duration(0.45)
            .appear()
            .spawn();
    }
    for (start, snapshot) in [(8.6, scaled), (10.0, masked), (11.4, attention)] {
        timeline
            .animate(attention_id)
            .at(start)
            .for_duration(0.9)
            .ease(Ease::InOutSmooth)
            .tensor_to(snapshot)
            .spawn();
    }
    for id in [attention_id, stage_ids[2]] {
        timeline
            .animate(id)
            .at(12.8)
            .for_duration(0.4)
            .fade_to(0.0)
            .spawn();
    }
    for id in [
        context_id,
        residual_id,
        context_label_id,
        residual_label_id,
        stage_ids[3],
    ] {
        timeline
            .animate(id)
            .at(13.2)
            .for_duration(0.45)
            .appear()
            .spawn();
        timeline
            .animate(id)
            .at(15.8)
            .for_duration(0.4)
            .fade_to(0.0)
            .spawn();
    }
    for id in [output_id, stage_ids[4]] {
        timeline
            .animate(id)
            .at(16.2)
            .for_duration(0.45)
            .appear()
            .spawn();
    }
    timeline
        .animate(output_id)
        .at(17.6)
        .for_duration(1.0)
        .ease(Ease::InOutSmooth)
        .tensor_to(probabilities)
        .spawn();
    timeline
        .animate(sample_id)
        .at(19.0)
        .for_duration(0.6)
        .appear()
        .spawn();
    timeline.wait_until(21.0);

    scene.play(timeline)?;
    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    scene.camera_mut().set_view_width(15.0);

    App::new()?.with_scene(scene).run_app()
}
