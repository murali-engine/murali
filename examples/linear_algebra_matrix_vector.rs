// murali-example-tags: linear-algebra,math,reference

use glam::{Mat2, Vec3, vec2, vec3};
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::frontend::collection::composite::number_plane::NumberPlane;
use murali::frontend::collection::math::linear_algebra::{
    LabeledVector2D, MatrixVectorFlow, TransformableGrid2D, VectorLabelAnchor,
};
use murali::frontend::collection::text::label::Label;
use murali::positions::CAMERA_DEFAULT_POS;
use murali::{App, Timeline};

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    scene.add_tattva(
        Label::new("Matrix Vector Multiplication", 0.34).with_color(WHITE),
        vec3(0.0, 3.0, 0.0),
    );
    scene.add_tattva(
        Label::new(
            "The input vector moves by following the same rule that moves every grid point.",
            0.16,
        )
        .with_color(GRAY_B),
        vec3(0.0, 2.58, 0.0),
    );

    let matrix = Mat2::from_cols(vec2(1.4, 0.35), vec2(-0.45, 1.15));
    let input = vec2(1.6, 1.1);
    let output = matrix * input;

    scene.add_tattva(
        NumberPlane::new((-3.3, 3.3), (-2.2, 2.2)).with_step(1.0),
        Vec3::ZERO,
    );
    scene.add_tattva(
        TransformableGrid2D::new(matrix)
            .with_range((-3.3, 3.3), (-2.2, 2.2))
            .with_step(0.55)
            .with_basis_vectors(false),
        Vec3::ZERO,
    );

    let input_id = scene.add_tattva(
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
        MatrixVectorFlow::new(matrix, input)
            .with_labels("A", "x", "b = Ax")
            .with_positions(
                vec2(-1.95, 0.2),
                vec2(-0.65, 0.2),
                vec2(0.35, 0.2),
                vec2(1.3, 0.2),
            )
            .with_expansion_position(vec2(-0.2, -0.78)),
        vec3(0.0, -2.55, 0.0),
    );
    scene.add_tattva(
        MatrixVectorFlow::try_from_rows(
            vec![vec![1.0, -0.5, 2.0], vec![0.0, 1.5, 0.75]],
            vec![2.0, 1.0, -0.5],
        )
        .expect("rectangular matrix/vector dimensions should match")
        .with_labels("R", "z", "Rz")
        .with_row_expansion(false)
        .with_positions(
            vec2(-1.7, 0.0),
            vec2(-0.35, 0.0),
            vec2(0.72, 0.0),
            vec2(1.55, 0.0),
        )
        .with_text_height(0.16)
        .with_matrix_cell_height(0.2),
        vec3(2.6, -1.45, 0.0),
    );

    let mut timeline = Timeline::new();
    timeline
        .animate(input_id)
        .at(0.25)
        .for_duration(0.7)
        .appear()
        .spawn();
    timeline
        .animate(output_id)
        .at(0.75)
        .for_duration(0.8)
        .appear()
        .spawn();
    scene.play(timeline)?;

    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    scene.camera_mut().set_view_width(9.8);

    App::new()?.with_scene(scene).run_app()
}
