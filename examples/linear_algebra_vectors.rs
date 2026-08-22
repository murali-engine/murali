// murali-example-tags: linear-algebra,math,reference

use glam::{Vec3, vec2, vec3};
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::frontend::collection::composite::number_plane::NumberPlane;
use murali::frontend::collection::math::linear_algebra::{
    CoordinateReadout, CoordinateReadoutMode, LabeledVector2D, ScalarMultiplicationView,
    VectorArrow2D, VectorLabelAnchor,
};
use murali::frontend::collection::text::label::Label;
use murali::positions::CAMERA_DEFAULT_POS;
use murali::{App, Timeline};

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    scene.add_tattva(
        Label::new("Linear Algebra: Vectors", 0.36).with_color(WHITE),
        vec3(0.0, 3.05, 0.0),
    );
    scene.add_tattva(
        Label::new(
            "The same vector can be read geometrically as an arrow and numerically as coordinates.",
            0.17,
        )
        .with_color(GRAY_B),
        vec3(0.0, 2.58, 0.0),
    );

    scene.add_tattva(
        NumberPlane::new((-4.0, 4.0), (-2.4, 2.4)).with_step(1.0),
        Vec3::ZERO,
    );

    let vector = vec2(2.6, 1.4);
    let vector_id = scene.add_tattva(
        LabeledVector2D::new("v", vector)
            .with_color(TEAL_C)
            .with_label_color(WHITE)
            .with_anchor(VectorLabelAnchor::Tip)
            .with_coordinates(true),
        Vec3::ZERO,
    );

    scene.add_tattva(
        CoordinateReadout::from_vec2(vector).with_mode(CoordinateReadoutMode::ColumnVector),
        vec3(4.25, 0.75, 0.0),
    );
    scene.add_tattva(
        CoordinateReadout::from_vec2(vector).with_mode(CoordinateReadoutMode::Tuple),
        vec3(4.25, 1.55, 0.0),
    );
    scene.add_tattva(
        CoordinateReadout::from_vec2(vector).with_mode(CoordinateReadoutMode::RowVector),
        vec3(4.25, 1.18, 0.0),
    );
    scene.add_tattva(
        CoordinateReadout::new(vec![0.42, 0.81, 0.18, 0.63])
            .with_labels(vec!["topic", "style", "depth", "tone"])
            .with_mode(CoordinateReadoutMode::FeatureList)
            .with_highlights(vec![1]),
        vec3(-4.45, -0.65, 0.0),
    );
    scene.add_tattva(
        VectorArrow2D::new(vec2(-3.5, -1.85), vec2(-2.1, -1.3))
            .with_color(GOLD_C)
            .with_thickness(0.035),
        Vec3::ZERO,
    );
    scene.add_tattva(
        ScalarMultiplicationView::new(vec2(0.7, 0.35), 2.4).with_labels("u", "2.4u"),
        vec3(1.75, -1.8, 0.0),
    );

    let mut timeline = Timeline::new();
    timeline
        .animate(vector_id)
        .at(0.4)
        .for_duration(1.0)
        .appear()
        .spawn();
    scene.play(timeline)?;

    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    scene.camera_mut().set_view_width(10.5);

    App::new()?.with_scene(scene).run_app()
}
