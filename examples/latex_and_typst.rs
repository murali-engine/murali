use glam::Vec3;
use murali::App;
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::animation::Ease;
use murali::frontend::layout::Direction;
use murali::frontend::sangrah::ganit::notation::equation::{
    VectorLatexEquation, VectorTypstEquation,
};
use murali::frontend::sangrah::text::label::Label;
use murali::frontend::sangrah::text::latex::Latex;
use murali::frontend::sangrah::text::typst::Typst;
use murali::positions::CAMERA_DEFAULT_POS;

fn place_equation(scene: &mut Scene, ids: &[usize], position: Vec3) {
    for &id in ids {
        scene.set_position_2d(id, position.truncate());
    }
}

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    let title_id = scene.add_tattva(
        Label::new("LaTeX And Typst", 0.38).with_color(WHITE),
        Vec3::ZERO,
    );
    scene.to_edge(title_id, Direction::Up, 0.8);

    let subtitle_id = scene.add_tattva(
        Label::new(
            "Static rendering first, then one vector morph sequence for authored math continuity.",
            0.18,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, 2.95, 0.0),
    );

    let latex_heading_id = scene.add_tattva(
        Label::new("LaTeX", 0.2).with_color(GOLD_B),
        Vec3::new(-4.5, 1.85, 0.0),
    );
    let latex_id = scene.add_tattva(
        Latex::new(r"\int_0^1 x^2 \, dx = \frac{1}{3}", 0.72).with_color(GOLD_B),
        Vec3::new(-4.5, 0.75, 0.0),
    );
    let latex_caption_id = scene.add_tattva(
        Label::new("Good for familiar TeX-style math input.", 0.16).with_color(GRAY_A),
        Vec3::new(-4.5, -0.15, 0.0),
    );

    let typst_heading_id = scene.add_tattva(
        Label::new("Typst", 0.2).with_color(TEAL_C),
        Vec3::new(4.5, 1.85, 0.0),
    );
    let typst_id = scene.add_tattva(
        Typst::new(r#"$f(x) = x^2 + 2 x + 1$"#, 0.46).with_color(TEAL_C),
        Vec3::new(4.5, 0.8, 0.0),
    );
    let typst_caption_id = scene.add_tattva(
        Label::new("Good for modern document-native math authoring.", 0.16).with_color(GRAY_A),
        Vec3::new(4.5, -0.15, 0.0),
    );

    let morph_heading_id = scene.add_tattva(
        Label::new("Vector Morph", 0.2).with_color(GRAY_B),
        Vec3::new(0.0, -1.25, 0.0),
    );
    let morph_caption_id = scene.add_tattva(
        Label::new(
            "A compact example: Typst source morphs into a LaTeX result.",
            0.16,
        )
        .with_color(GRAY_A),
        Vec3::new(0.0, -2.9, 0.0),
    );

    let source = VectorTypstEquation::new("$(a + b)^2$", 0.95).with_color(BLUE_B);
    let target = VectorLatexEquation::new(r"a^2 + 2ab + b^2", 0.95).with_color(GOLD_C);
    let source_handle = scene.add_vector_typst(source);
    let target_handle = scene.add_vector_latex(target);
    place_equation(&mut scene, source_handle.ids(), Vec3::new(0.0, -2.0, 0.0));
    place_equation(&mut scene, target_handle.ids(), Vec3::new(0.0, -2.0, 0.0));

    let mut timeline = Timeline::new();
    timeline
        .animate(title_id)
        .at(0.0)
        .for_duration(1.0)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(subtitle_id)
        .at(0.35)
        .for_duration(1.6)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    timeline
        .animate(latex_heading_id)
        .at(1.5)
        .for_duration(0.8)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(latex_id)
        .at(1.9)
        .for_duration(1.0)
        .ease(Ease::Linear)
        .appear()
        .spawn();
    timeline
        .animate(latex_caption_id)
        .at(2.2)
        .for_duration(1.0)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    timeline
        .animate(typst_heading_id)
        .at(2.8)
        .for_duration(0.8)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(typst_id)
        .at(3.2)
        .for_duration(1.0)
        .ease(Ease::Linear)
        .appear()
        .spawn();
    timeline
        .animate(typst_caption_id)
        .at(3.5)
        .for_duration(1.0)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    timeline
        .animate(morph_heading_id)
        .at(4.5)
        .for_duration(0.8)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(morph_caption_id)
        .at(4.9)
        .for_duration(1.0)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    timeline.morph_vector_equations(
        &source_handle,
        &target_handle,
        &mut scene,
        5.5,
        2.6,
        Ease::InOutCubic,
    );

    scene.play(timeline)?;
    scene.camera_mut().position = CAMERA_DEFAULT_POS;

    App::new()?.with_scene(scene).run_app()
}
