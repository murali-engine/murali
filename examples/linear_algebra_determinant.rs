// murali-example-tags: linear-algebra,math,reference

use glam::{Mat2, vec2, vec3};
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::frontend::collection::composite::number_plane::NumberPlane;
use murali::frontend::collection::maths::linear_algebra::{
    DeterminantAreaView, MatrixTransformPanel, TransformableGrid2D,
};
use murali::frontend::collection::text::label::Label;
use murali::positions::CAMERA_DEFAULT_POS;
use murali::{App, Timeline};

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    scene.add_tattva(
        Label::new("Determinant As Area Scaling", 0.34).with_color(WHITE),
        vec3(0.0, 3.0, 0.0),
    );
    scene.add_tattva(
        Label::new(
            "The unit square becomes a parallelogram; its signed area is det(A).",
            0.16,
        )
        .with_color(GRAY_B),
        vec3(0.0, 2.58, 0.0),
    );

    let stretch = Mat2::from_cols(vec2(1.6, 0.25), vec2(-0.35, 1.2));
    let flip = Mat2::from_cols(vec2(0.2, 1.1), vec2(1.2, 0.15));
    let collapse = Mat2::from_cols(vec2(1.2, 0.65), vec2(2.4, 1.3));

    add_case(&mut scene, "scales area", stretch, vec3(-3.25, 0.25, 0.0));
    add_case(&mut scene, "flips orientation", flip, vec3(0.0, 0.25, 0.0));
    add_case(&mut scene, "collapses", collapse, vec3(3.25, 0.25, 0.0));

    scene.play(Timeline::new())?;

    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    scene.camera_mut().set_view_width(10.5);

    App::new()?.with_scene(scene).run_app()
}

fn add_case(scene: &mut Scene, title: &str, matrix: Mat2, offset: glam::Vec3) {
    scene.add_tattva(
        Label::new(title, 0.2).with_color(WHITE),
        offset + vec3(0.0, 1.55, 0.0),
    );
    scene.add_tattva(
        NumberPlane::new((-1.2, 2.2), (-0.8, 2.0)).with_step(1.0),
        offset,
    );
    scene.add_tattva(
        TransformableGrid2D::new(matrix)
            .with_range((-1.2, 2.2), (-0.8, 2.0))
            .with_step(1.0)
            .with_source_grid(false)
            .with_basis_vectors(false),
        offset,
    );
    scene.add_tattva(DeterminantAreaView::new(matrix), offset);
    scene.add_tattva(
        MatrixTransformPanel::new(matrix).with_cell_height(0.2),
        offset + vec3(0.0, -1.55, 0.0),
    );
}
