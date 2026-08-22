use glam::Vec3;
use murali::App;
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::animation::Ease;
use murali::frontend::collection::ai::{KvCacheView, TensorAxis, TensorSnapshot};
use murali::frontend::collection::text::label::Label;
use murali::frontend::layout::Direction;
use murali::positions::CAMERA_DEFAULT_POS;

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();
    let title = scene.add_tattva(
        Label::new("Why generation gets faster", 0.38).with_color(WHITE),
        Vec3::ZERO,
    );
    scene.to_edge(title, Direction::Up, 0.58);

    let token_axis = TensorAxis::with_elements(
        "token",
        "Cached positions",
        [
            ("token.the", "The"),
            ("token.model", "model"),
            ("token.reuses", "reuses"),
            ("token.past", "past"),
            ("token.keys", "keys"),
            ("token.values", "values"),
        ],
    );
    let feature_axis = TensorAxis::new("feature", "Head features", vec!["f0", "f1", "f2", "f3"]);
    let keys = TensorSnapshot::try_new(
        "layer.7.head.2.keys",
        vec![6, 4],
        vec![
            0.8, -0.2, 0.4, 0.1, 0.5, 0.7, -0.3, 0.2, -0.4, 0.9, 0.6, -0.1, 0.3, -0.6, 0.8, 0.5,
            0.7, 0.2, -0.5, 0.9, -0.2, 0.4, 0.6, 0.1,
        ],
        vec![token_axis.clone(), feature_axis.clone()],
    )?;
    let values = TensorSnapshot::try_new(
        "layer.7.head.2.values",
        vec![6, 4],
        vec![
            -0.1, 0.6, 0.3, 0.7, 0.8, -0.5, 0.2, 0.4, 0.4, 0.7, -0.3, 0.1, 0.2, -0.8, 0.5, 0.6,
            -0.4, 0.9, 0.2, -0.2, 0.5, 0.8, 0.3, -0.6,
        ],
        vec![token_axis, feature_axis],
    )?;
    let cache = KvCacheView::try_new(keys, values, "token", "feature", 0)?;
    let cache_id = scene.add_tattva(cache, Vec3::new(0.0, -0.1, 0.0));

    let note = scene.add_tattva(
        Label::new(
            "Each generated token appends one key row and one value row; earlier rows are reused.",
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
    for occupied in 1..=6 {
        timeline
            .animate(cache_id)
            .at(0.55 + occupied as f32 * 0.48)
            .for_duration(0.38)
            .ease(Ease::OutCubic)
            .kv_cache_fill_to(occupied)
            .spawn();
    }
    timeline
        .animate(note)
        .at(3.75)
        .for_duration(1.8)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    scene.play(timeline)?;
    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    App::new()?.with_scene(scene).run_app()
}
