// murali-example-tags: linear-algebra,math,reference

use glam::{Mat2, Vec3, vec2, vec3};
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::frontend::collection::maths::linear_algebra::{
    CoordinateReadout, CoordinateReadoutMode, LabeledVector2D, MatrixTransformPanel,
    TransformableGrid2D, VectorLabelAnchor,
};
use murali::frontend::collection::text::label::Label;
use murali::positions::CAMERA_DEFAULT_POS;
use murali::{App, Timeline};

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    scene.add_tattva(
        Label::new("Matrices Transform Space", 0.36).with_color(WHITE),
        vec3(0.0, 2.85, 0.0),
    );
    scene.add_tattva(
        Label::new(
            "Each column tells where a basis vector lands; the whole grid follows.",
            0.17,
        )
        .with_color(GRAY_B),
        vec3(0.0, 2.43, 0.0),
    );

    let matrix = Mat2::from_cols(vec2(1.4, 0.35), vec2(-0.45, 1.15));
    let input = vec2(1.6, 1.1);
    let output = matrix * input;

    let grid_id = scene.add_tattva(
        TransformableGrid2D::new(matrix)
            .with_range((-3.5, 3.5), (-2.5, 2.5))
            .with_step(0.5),
        Vec3::ZERO,
    );

    scene.add_tattva(
        MatrixTransformPanel::new(matrix).with_cell_height(0.34),
        vec3(3.85, 1.0, 0.0),
    );

    scene.add_tattva(
        LabeledVector2D::new("x", input)
            .with_color(GRAY_B)
            .with_anchor(VectorLabelAnchor::Tip)
            .with_coordinates(true),
        Vec3::ZERO,
    );

    let output_id = scene.add_tattva(
        LabeledVector2D::new("Ax", output)
            .with_color(GREEN_C)
            .with_anchor(VectorLabelAnchor::Tip)
            .with_coordinates(true),
        Vec3::ZERO,
    );

    scene.add_tattva(
        CoordinateReadout::from_vec2(output).with_mode(CoordinateReadoutMode::ColumnVector),
        vec3(3.85, -1.0, 0.0),
    );

    let mut timeline = Timeline::new();
    timeline
        .animate(grid_id)
        .at(0.25)
        .for_duration(1.0)
        .appear()
        .spawn();
    timeline
        .animate(output_id)
        .at(0.85)
        .for_duration(0.8)
        .appear()
        .spawn();
    scene.play(timeline)?;

    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    scene.camera_mut().set_view_width(11.2);

    App::new()?.with_scene(scene).run_app()
}
