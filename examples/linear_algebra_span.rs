// murali-example-tags: linear-algebra,math,reference

use glam::{Vec3, Vec4, vec2, vec3};
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::frontend::sangrah::composite::number_plane::NumberPlane;
use murali::frontend::sangrah::ganit::linear_algebra::{
    BasisVectors2D, CoordinateReadout, CoordinateReadoutMode, LinearCombinationView, SpanRegion2D,
    VectorAdditionView,
};
use murali::frontend::sangrah::text::label::Label;
use murali::positions::CAMERA_DEFAULT_POS;
use murali::{App, Timeline};

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    scene.add_tattva(
        Label::new("Span And Linear Combinations", 0.36).with_color(WHITE),
        vec3(0.0, 3.05, 0.0),
    );
    scene.add_tattva(
        Label::new(
            "A span is the set of vectors reachable by scaling and adding basis directions.",
            0.17,
        )
        .with_color(GRAY_B),
        vec3(0.0, 2.58, 0.0),
    );

    scene.add_tattva(
        NumberPlane::new((-4.0, 4.0), (-2.4, 2.4)).with_step(1.0),
        Vec3::ZERO,
    );

    let u = vec2(1.3, 0.35);
    let v = vec2(0.35, 1.1);

    scene.add_tattva(
        SpanRegion2D::plane(u, v)
            .with_extent(4.5)
            .with_step(0.75)
            .with_color(Vec4::new(0.34, 0.78, 0.95, 0.22)),
        Vec3::ZERO,
    );

    scene.add_tattva(
        BasisVectors2D::new(u, v)
            .with_labels("u", "v")
            .with_coordinates(true),
        Vec3::ZERO,
    );

    let combo = LinearCombinationView::new(u, v, 1.7, 1.2).with_labels("1.7u", "1.2v", "x");
    let combo_id = scene.add_tattva(combo, Vec3::ZERO);

    let addition_id = scene.add_tattva(
        VectorAdditionView::new(u * 1.7, v * 1.2).with_labels("1.7u", "1.2v", "x"),
        vec3(0.0, -0.08, 0.0),
    );

    scene.add_tattva(
        CoordinateReadout::from_vec2(u * 1.7 + v * 1.2)
            .with_mode(CoordinateReadoutMode::ColumnVector),
        vec3(4.25, 0.4, 0.0),
    );

    let mut timeline = Timeline::new();
    timeline
        .animate(combo_id)
        .at(0.35)
        .for_duration(0.9)
        .appear()
        .spawn();
    timeline
        .animate(addition_id)
        .at(0.75)
        .for_duration(0.9)
        .appear()
        .spawn();
    scene.play(timeline)?;

    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    scene.camera_mut().set_view_width(10.5);

    App::new()?.with_scene(scene).run_app()
}
