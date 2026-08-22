// murali-example-tags: linear-algebra,math,reference,scene-view

use glam::{Mat2, Vec3, Vec4, vec2, vec3};
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::frontend::TattvaId;
use murali::frontend::animation::Ease;
use murali::frontend::collection::maths::linear_algebra::{
    LabeledVector2D, MatrixTransformPanel, TransformableGrid2D, VectorLabelAnchor,
};
use murali::frontend::collection::text::label::Label;
use murali::positions::CAMERA_DEFAULT_POS;
use murali::{App, SceneView, SceneViewPlayback, Timeline};

fn alpha(color: Vec4, opacity: f32) -> Vec4 {
    Vec4::new(color.x, color.y, color.z, opacity)
}

fn add_transform_layer(scene: &mut Scene, matrix: Mat2, color: Vec4) -> murali::frontend::TattvaId {
    let mut grid = TransformableGrid2D::new(matrix)
        .with_range((-2.4, 2.4), (-1.8, 1.8))
        .with_step(0.6)
        .with_source_grid(false)
        .with_basis_vectors(false);
    grid.grid_color = alpha(color, 0.42);
    grid.axis_color = alpha(color, 0.82);
    grid.grid_thickness = 0.012;
    grid.axis_thickness = 0.03;

    scene.add_tattva(grid, Vec3::ZERO)
}

fn add_transform_step(
    scene: &mut Scene,
    matrix: Mat2,
    label: &str,
    label_pos: Vec3,
    color: Vec4,
) -> (TattvaId, TattvaId) {
    let id = scene.add_tattva(Label::new(label, 0.17).with_color(color), label_pos);
    (add_transform_layer(scene, matrix, color), id)
}

fn transform_sequence_scene(
    title: &str,
    first_name: &str,
    second_name: &str,
    first: Mat2,
    second: Mat2,
) -> anyhow::Result<Scene> {
    let mut scene = Scene::new();
    let composed = second * first;
    let input = vec2(1.0, 1.0);

    scene.add_tattva(
        Label::new(title, 0.28).with_color(WHITE),
        vec3(0.0, 2.25, 0.0),
    );
    scene.add_tattva(
        Label::new(format!("{first_name} first, then {second_name}"), 0.16).with_color(GRAY_B),
        vec3(0.0, 1.9, 0.0),
    );

    let start_grid = add_transform_layer(&mut scene, Mat2::IDENTITY, GRAY_B);
    let (first_grid, first_label) = add_transform_step(
        &mut scene,
        first,
        &format!("after {first_name}"),
        vec3(-2.55, -2.25, 0.0),
        BLUE_C,
    );
    let (final_grid, final_label) = add_transform_step(
        &mut scene,
        composed,
        &format!("after {second_name}: {second_name}{first_name}"),
        vec3(1.8, -2.25, 0.0),
        GREEN_C,
    );
    let final_vector = scene.add_tattva(
        LabeledVector2D::new(&format!("{second_name}{first_name}x"), composed * input)
            .with_color(GREEN_C)
            .with_anchor(VectorLabelAnchor::Tip)
            .with_coordinates(true),
        Vec3::ZERO,
    );

    scene.add_tattva(
        LabeledVector2D::new("x", input)
            .with_color(GRAY_B)
            .with_anchor(VectorLabelAnchor::Tip),
        Vec3::ZERO,
    );
    let panel = scene.add_tattva(
        MatrixTransformPanel::new(composed).with_cell_height(0.22),
        vec3(3.1, 1.1, 0.0),
    );

    let mut timeline = Timeline::new();
    timeline
        .animate(start_grid)
        .at(0.1)
        .for_duration(0.45)
        .ease(Ease::OutCubic)
        .appear()
        .spawn();
    for id in [first_grid, first_label] {
        timeline
            .animate(id)
            .at(0.8)
            .for_duration(0.7)
            .ease(Ease::OutCubic)
            .appear()
            .spawn();
    }
    for id in [final_grid, final_label, panel] {
        timeline
            .animate(id)
            .at(1.7)
            .for_duration(0.7)
            .ease(Ease::OutCubic)
            .appear()
            .spawn();
    }
    timeline
        .animate(final_vector)
        .at(2.2)
        .for_duration(0.55)
        .ease(Ease::OutCubic)
        .appear()
        .spawn();
    scene.play(timeline)?;

    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    scene.camera_mut().set_view_width(8.4);
    Ok(scene)
}

fn main() -> anyhow::Result<()> {
    let scale_x = Mat2::from_cols(vec2(1.45, 0.0), vec2(0.0, 1.0));
    let shear = Mat2::from_cols(vec2(1.0, 0.0), vec2(0.65, 1.0));

    let first_view_scene = transform_sequence_scene("Path 1", "A", "B", scale_x, shear)?;
    let second_view_scene = transform_sequence_scene("Path 2", "B", "A", shear, scale_x)?;

    let mut scene = Scene::new();
    let path_one = scene.add_scene_view(
        SceneView::new(first_view_scene)
            .size(vec2(8.8, 5.0))
            .background(alpha(BLUE_D, 0.12))
            .border(0.035, alpha(BLUE_C, 0.75))
            .corner_radius(0.16)
            .playback(SceneViewPlayback::Once),
        Vec3::ZERO,
    );
    let path_two = scene.add_scene_view(
        SceneView::new(second_view_scene)
            .size(vec2(8.8, 5.0))
            .background(alpha(GREEN_D, 0.10))
            .border(0.035, alpha(GREEN_C, 0.75))
            .corner_radius(0.16)
            .start_at(3.15)
            .playback(SceneViewPlayback::Once),
        vec3(0.0, -7.0, 0.0),
    );

    let title = scene.add_tattva(
        Label::new("Same transforms, different order", 0.34).with_color(WHITE),
        vec3(0.0, 2.75, 0.0),
    );
    let compare = scene.add_tattva(
        Label::new("A then B lands differently from B then A", 0.2).with_color(TEAL_C),
        vec3(0.0, -2.75, 0.0),
    );

    let mut timeline = Timeline::new();
    timeline
        .animate(path_one)
        .at(2.95)
        .for_duration(0.8)
        .ease(Ease::InOutCubic)
        .move_to(vec3(-2.95, 1.15, 0.0))
        .spawn();
    timeline
        .animate(path_one)
        .at(2.95)
        .for_duration(0.8)
        .ease(Ease::InOutCubic)
        .scale_to(Vec3::new(0.42, 0.42, 1.0))
        .spawn();
    timeline
        .animate(path_two)
        .at(3.15)
        .for_duration(0.75)
        .ease(Ease::InOutCubic)
        .move_to(Vec3::ZERO)
        .spawn();
    timeline
        .animate(path_two)
        .at(5.95)
        .for_duration(0.8)
        .ease(Ease::InOutCubic)
        .move_to(vec3(2.95, 1.15, 0.0))
        .spawn();
    timeline
        .animate(path_two)
        .at(5.95)
        .for_duration(0.8)
        .ease(Ease::InOutCubic)
        .scale_to(Vec3::new(0.42, 0.42, 1.0))
        .spawn();
    timeline
        .animate(title)
        .at(6.75)
        .for_duration(0.5)
        .ease(Ease::OutCubic)
        .appear()
        .spawn();
    timeline
        .animate(compare)
        .at(7.05)
        .for_duration(0.6)
        .ease(Ease::OutCubic)
        .appear()
        .spawn();
    scene.play(timeline)?;

    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    scene.camera_mut().set_view_width(9.8);

    App::new()?.with_scene(scene).run_app()
}
