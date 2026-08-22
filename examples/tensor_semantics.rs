use glam::Vec3;
use murali::App;
use murali::colors::*;
use murali::frontend::animation::Ease;
use murali::frontend::layout::Direction;
use murali::frontend::sangrah::text::label::Label;
use murali::positions::CAMERA_DEFAULT_POS;
use murali::prelude::*;

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    let title_id = scene.add_tattva(
        Label::new("Inside Self-Attention", 0.38).with_color(WHITE),
        Vec3::ZERO,
    );
    scene.to_edge(title_id, Direction::Up, 0.8);

    let subtitle_id = scene.add_tattva(
        Label::new(
            "Q and K produce every animated value, from dot products to attention weights.",
            0.18,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, 2.9, 0.0),
    );

    let tokens = [
        ("token.0", "The"),
        ("token.1", "model"),
        ("token.2", "reads"),
        ("token.3", "context"),
    ];
    let features = [("feature.0", "x"), ("feature.1", "y")];
    let queries = TensorSnapshot::try_new(
        "layer.3.head.1.q",
        vec![4, 2],
        vec![1.0, 0.2, 0.1, 1.0, -0.4, 0.8, -0.5, 0.6],
        vec![
            TensorAxis::with_elements("query", "Query tokens", tokens),
            TensorAxis::with_elements("feature", "Features", features),
        ],
    )?;
    let keys = TensorSnapshot::try_new(
        "layer.3.head.1.k",
        vec![4, 2],
        vec![1.0, 0.3, 0.2, 1.0, -0.3, 0.7, 0.1, 0.5],
        vec![
            TensorAxis::with_elements("key", "Key tokens", tokens),
            TensorAxis::with_elements("feature", "Features", features),
        ],
    )?;

    let keys_transposed = keys.try_transpose_2d("layer.3.head.1.k_transposed")?;
    let dot_products = queries.try_matmul(&keys_transposed, "layer.3.head.1.attention")?;
    let scaled = dot_products.try_scaled((queries.shape[1] as f32).sqrt())?;
    let masked = scaled.try_causal_masked(-4.0)?;
    let weights = masked.try_softmax("key")?;

    let mut view = TensorView::try_new(dot_products)?;
    view.cell_size = vec2(1.05, 0.72);
    view.label_height = 0.2;
    view.value_height = 0.17;
    view.selection_color = GOLD_A;

    let tensor_id = scene.add_tattva(view, Vec3::new(0.0, -0.15, 0.0));
    scene.hide(tensor_id);

    let stages_id = scene.add_tattva(
        Label::new(
            "QK^T   ->   / sqrt(d_k)   ->   + causal mask   ->   softmax",
            0.2,
        )
        .with_color(TEAL_A),
        Vec3::new(0.0, 2.15, 0.0),
    );

    let caption_id = scene.add_tattva(
        Label::new(
            "The highlighted token keeps its identity while real tensor operations change the values.",
            0.17,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, -3.05, 0.0),
    );

    let mut timeline = Timeline::new();
    timeline
        .animate(title_id)
        .at(0.0)
        .for_duration(0.9)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(subtitle_id)
        .at(0.3)
        .for_duration(1.6)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(tensor_id)
        .at(1.5)
        .for_duration(0.5)
        .ease(Ease::InOutQuad)
        .appear()
        .spawn();
    timeline
        .animate(stages_id)
        .at(1.7)
        .for_duration(1.5)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(tensor_id)
        .at(2.1)
        .for_duration(0.7)
        .ease(Ease::InOutQuad)
        .tensor_select(vec![TensorSelector::axis_element("query", "token.2")])
        .spawn();
    timeline
        .animate(caption_id)
        .at(2.3)
        .for_duration(1.3)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(tensor_id)
        .at(3.2)
        .for_duration(1.1)
        .ease(Ease::InOutSmooth)
        .tensor_to(scaled)
        .spawn();
    timeline
        .animate(tensor_id)
        .at(4.8)
        .for_duration(1.1)
        .ease(Ease::InOutSmooth)
        .tensor_to(masked)
        .spawn();
    timeline
        .animate(tensor_id)
        .at(6.4)
        .for_duration(1.1)
        .ease(Ease::InOutSmooth)
        .tensor_to(weights)
        .spawn();
    timeline.wait_until(9.0);

    scene.play(timeline)?;
    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    scene.camera_mut().set_view_width(14.0);

    App::new()?.with_scene(scene).run_app()
}
