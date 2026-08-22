// murali-example-tags: linear-algebra,math,reference

use glam::{Mat2, vec2, vec3};
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::frontend::collection::math::linear_algebra::MatrixTransformPanel;
use murali::frontend::collection::text::label::Label;
use murali::positions::CAMERA_DEFAULT_POS;
use murali::{App, Timeline};

fn add_matrix_case(scene: &mut Scene, label: &str, matrix: Mat2, offset: glam::Vec3) {
    scene.add_tattva(
        Label::new(label, 0.24).with_color(WHITE),
        offset + vec3(0.0, 0.7, 0.0),
    );
    scene.add_tattva(
        MatrixTransformPanel::new(matrix).with_cell_height(0.24),
        offset + vec3(0.0, 0.0, 0.0),
    );
}

fn add_operator_labels(scene: &mut Scene, y: f32) {
    scene.add_tattva(
        Label::new("then", 0.18).with_color(GRAY_B),
        vec3(-1.45, y, 0.0),
    );
    scene.add_tattva(Label::new("=", 0.26).with_color(GRAY_B), vec3(1.45, y, 0.0));
}

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    scene.add_tattva(
        Label::new("Transform Composition", 0.34).with_color(WHITE),
        vec3(0.0, 2.55, 0.0),
    );
    scene.add_tattva(
        Label::new(
            "For column vectors: applying A, then B gives BA. Reversing the order gives AB.",
            0.16,
        )
        .with_color(GRAY_B),
        vec3(0.0, 2.18, 0.0),
    );

    let scale_x = Mat2::from_cols(vec2(1.45, 0.0), vec2(0.0, 1.0));
    let shear = Mat2::from_cols(vec2(1.0, 0.0), vec2(0.65, 1.0));
    scene.add_tattva(Label::new("A then B", 0.2).with_color(GRAY_B), vec3(-3.7, 0.9, 0.0));
    add_matrix_case(&mut scene, "A", scale_x, vec3(-2.2, 0.55, 0.0));
    add_matrix_case(&mut scene, "B", shear, vec3(0.0, 0.55, 0.0));
    add_matrix_case(&mut scene, "BA", shear * scale_x, vec3(2.2, 0.55, 0.0));
    add_operator_labels(&mut scene, 0.55);

    scene.add_tattva(Label::new("B then A", 0.2).with_color(GRAY_B), vec3(-3.7, -1.15, 0.0));
    add_matrix_case(&mut scene, "B", shear, vec3(-2.2, -1.5, 0.0));
    add_matrix_case(&mut scene, "A", scale_x, vec3(0.0, -1.5, 0.0));
    add_matrix_case(&mut scene, "AB", scale_x * shear, vec3(2.2, -1.5, 0.0));
    add_operator_labels(&mut scene, -1.5);

    scene.add_tattva(
        Label::new("BA and AB are different here, so transform order matters.", 0.18)
            .with_color(TEAL_C),
        vec3(0.0, -2.55, 0.0),
    );

    scene.play(Timeline::new())?;

    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    scene.camera_mut().set_view_width(8.8);

    App::new()?.with_scene(scene).run_app()
}
