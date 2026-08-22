// murali-example-tags: linear-algebra,math,reference

use glam::{Vec3, vec2, vec3};
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::frontend::sangrah::composite::number_plane::NumberPlane;
use murali::frontend::sangrah::ganit::linear_algebra::{
    AngleArc, AngleUnit, DotProductMeter, LabeledVector2D, MeterMode, OrthogonalityMarker,
    ProjectionShadow, VectorLabelAnchor,
};
use murali::frontend::sangrah::text::label::Label;
use murali::positions::CAMERA_DEFAULT_POS;
use murali::{App, Timeline};

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    scene.add_tattva(
        Label::new("Dot Product And Projection", 0.36).with_color(WHITE),
        vec3(0.0, 3.05, 0.0),
    );
    scene.add_tattva(
        Label::new(
            "Alignment becomes visible as an angle, a shadow, and a signed similarity meter.",
            0.17,
        )
        .with_color(GRAY_B),
        vec3(0.0, 2.58, 0.0),
    );

    scene.add_tattva(
        NumberPlane::new((-3.8, 3.8), (-2.3, 2.3)).with_step(1.0),
        Vec3::ZERO,
    );

    let a = vec2(2.5, 1.2);
    let b = vec2(2.1, -0.25);
    let projection = ProjectionShadow::new(a, b).projection();
    let residual = a - projection;

    let a_id = scene.add_tattva(
        LabeledVector2D::new("a", a)
            .with_color(TEAL_C)
            .with_label_color(WHITE)
            .with_anchor(VectorLabelAnchor::Tip),
        Vec3::ZERO,
    );
    let b_id = scene.add_tattva(
        LabeledVector2D::new("b", b)
            .with_color(GOLD_C)
            .with_label_color(WHITE)
            .with_anchor(VectorLabelAnchor::Tip),
        Vec3::ZERO,
    );

    scene.add_tattva(
        AngleArc::between(b, a)
            .with_radius(0.72)
            .with_auto_label(AngleUnit::Degrees),
        Vec3::ZERO,
    );

    scene.add_tattva(ProjectionShadow::new(a, b).with_original(false), Vec3::ZERO);
    scene.add_tattva(
        OrthogonalityMarker::new(b, residual)
            .with_vertex(projection)
            .with_size(0.24),
        Vec3::ZERO,
    );
    scene.add_tattva(DotProductMeter::new(a, b), vec3(0.0, -2.75, 0.0));
    scene.add_tattva(
        DotProductMeter::new(a, b).with_mode(MeterMode::DotProduct),
        vec3(-3.15, -2.75, 0.0),
    );
    scene.add_tattva(
        DotProductMeter::new(a, -b).with_mode(MeterMode::CosineSimilarity),
        vec3(3.15, -2.75, 0.0),
    );

    let mut timeline = Timeline::new();
    timeline
        .animate(a_id)
        .at(0.3)
        .for_duration(0.8)
        .appear()
        .spawn();
    timeline
        .animate(b_id)
        .at(0.55)
        .for_duration(0.8)
        .appear()
        .spawn();
    scene.play(timeline)?;

    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    scene.camera_mut().set_view_width(9.5);

    App::new()?.with_scene(scene).run_app()
}
