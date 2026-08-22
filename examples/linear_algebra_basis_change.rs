// murali-example-tags: linear-algebra,math,reference

use glam::{Vec3, Vec4, vec2, vec3};
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::frontend::collection::composite::number_plane::NumberPlane;
use murali::frontend::collection::maths::linear_algebra::{
    BasisGrid2D, BasisVectors2D, CoordinateReadout, CoordinateReadoutMode, DimensionBadge,
    LabeledVector2D, VectorLabelAnchor,
};
use murali::frontend::collection::text::label::Label;
use murali::positions::CAMERA_DEFAULT_POS;
use murali::{App, Timeline};

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    scene.add_tattva(
        Label::new("Same Vector, Different Coordinates", 0.34).with_color(WHITE),
        vec3(0.0, 2.72, 0.0),
    );
    scene.add_tattva(
        Label::new(
            "A basis is a coordinate system: the arrow stays fixed, the numbers change.",
            0.16,
        )
        .with_color(GRAY_B),
        vec3(0.0, 2.34, 0.0),
    );

    let basis = BasisVectors2D::new(vec2(1.35, 0.45), vec2(-0.45, 1.25))
        .with_labels("b1", "b2")
        .with_label_offsets(vec2(0.16, -0.28), vec2(-0.34, 0.1))
        .with_coordinates(false);
    let vector = vec2(2.25, 1.7);
    let basis_coordinates = basis.coordinates_of(vector).unwrap_or_default();

    let mut standard_grid = NumberPlane::new((-4.0, 4.0), (-3.0, 3.0)).with_step(1.0);
    standard_grid.grid_color = Vec4::new(0.65, 0.72, 0.80, 0.22);
    standard_grid.axis_color = Vec4::new(0.84, 0.88, 0.92, 0.38);
    standard_grid.grid_thickness = 0.008;
    standard_grid.axis_thickness = 0.018;
    scene.add_tattva(standard_grid, Vec3::ZERO);
    scene.add_tattva(
        BasisGrid2D::new(basis.clone())
            .with_range((-2.0, 3.0), (-2.0, 3.0))
            .with_step(1.0)
            .with_color(Vec4::new(0.55, 0.75, 0.95, 0.18))
            .with_axis_color(Vec4::new(0.72, 0.86, 1.0, 0.34))
            .with_thickness(0.012)
            .with_axis_thickness(0.024),
        Vec3::ZERO,
    );
    scene.add_tattva(basis, Vec3::ZERO);
    scene.add_tattva(
        LabeledVector2D::new("v", vector)
            .with_color(GREEN_C)
            .with_label_color(WHITE)
            .with_anchor(VectorLabelAnchor::Tip),
        Vec3::ZERO,
    );

    scene.add_tattva(
        Label::new("standard", 0.16).with_color(GRAY_B),
        vec3(-3.45, 1.25, 0.0),
    );
    scene.add_tattva(
        CoordinateReadout::from_vec2(vector).with_mode(CoordinateReadoutMode::ColumnVector),
        vec3(-3.45, 0.75, 0.0),
    );
    scene.add_tattva(
        Label::new("basis B", 0.16).with_color(GRAY_B),
        vec3(3.35, 1.25, 0.0),
    );
    scene.add_tattva(
        CoordinateReadout::from_vec2(basis_coordinates)
            .with_mode(CoordinateReadoutMode::ColumnVector),
        vec3(3.35, 0.75, 0.0),
    );
    scene.add_tattva(
        DimensionBadge::new("B", 2, 2).with_text_color(TEAL_C),
        vec3(3.35, -0.15, 0.0),
    );

    scene.play(Timeline::new())?;

    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    scene.camera_mut().set_view_width(11.2);

    App::new()?.with_scene(scene).run_app()
}
