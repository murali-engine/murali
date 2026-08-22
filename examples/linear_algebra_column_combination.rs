// murali-example-tags: linear-algebra,math,reference

use glam::{Mat2, Vec3, vec2, vec3};
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::frontend::sangrah::composite::number_plane::NumberPlane;
use murali::frontend::sangrah::ganit::linear_algebra::{
    ColumnCombinationView, DimensionBadge, MatrixTransformPanel, QuantityBadge,
};
use murali::frontend::sangrah::text::label::Label;
use murali::positions::CAMERA_DEFAULT_POS;
use murali::{App, Timeline};

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    scene.add_tattva(
        Label::new("Matrix Columns As Building Blocks", 0.34).with_color(WHITE),
        vec3(0.0, 3.0, 0.0),
    );
    scene.add_tattva(
        Label::new("Ax is a weighted sum of the columns of A.", 0.16).with_color(GRAY_B),
        vec3(0.0, 2.58, 0.0),
    );

    let matrix = Mat2::from_cols(vec2(1.45, 0.55), vec2(-0.55, 1.25));
    let coefficients = vec2(1.45, 1.1);
    let target = vec2(1.25, 2.55);

    scene.add_tattva(
        NumberPlane::new((-3.0, 3.2), (-1.5, 3.1)).with_step(1.0),
        Vec3::ZERO,
    );
    scene.add_tattva(
        ColumnCombinationView::new(matrix, coefficients)
            .with_labels("a1", "a2", "Ax")
            .with_target(target, "b"),
        vec3(-1.35, 0.15, 0.0),
    );
    scene.add_tattva(
        MatrixTransformPanel::new(matrix).with_cell_height(0.28),
        vec3(2.95, 0.85, 0.0),
    );
    scene.add_tattva(
        Label::new("x = [1.45, 1.10]", 0.18).with_color(GRAY_B),
        vec3(2.95, 0.15, 0.0),
    );
    scene.add_tattva(
        Label::new("b - Ax is the residual", 0.16).with_color(RED_C),
        vec3(2.95, -0.35, 0.0),
    );
    scene.add_tattva(
        DimensionBadge::new("A", 2, 2).with_text_color(TEAL_C),
        vec3(2.35, -0.9, 0.0),
    );
    scene.add_tattva(
        DimensionBadge::vector("x", 2).with_text_color(GOLD_C),
        vec3(3.2, -0.9, 0.0),
    );
    scene.add_tattva(
        QuantityBadge::new("rank", "2").with_text_color(GREEN_C),
        vec3(2.78, -1.35, 0.0),
    );

    scene.play(Timeline::new())?;

    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    scene.camera_mut().set_view_width(8.8);

    App::new()?.with_scene(scene).run_app()
}
