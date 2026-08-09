use glam::Vec3;
use murali::App;
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::animation::Ease;
use murali::frontend::collection::math::equation::{EquationLayout, EquationPart};
use murali::frontend::collection::math::matrix::Matrix;
use murali::frontend::collection::text::label::Label;
use murali::positions::CAMERA_DEFAULT_POS;
use murali::positions::*;

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    let title_id = scene.add_tattva(
        Label::new("Equation And Matrix Animation", 0.38).with_color(WHITE),
        Vec3::new(0.0, 3.0, 0.0),
    );
    let subtitle_id = scene.add_tattva(
        Label::new(
            "Math animation becomes easier to read when terms keep their identity and matrix focus steps stay explicit.",
            0.18,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, 2.45, 0.0),
    );

    let equation_heading_id = scene.add_tattva(
        Label::new("Equation Continuity", 0.19).with_color(GRAY_B),
        Vec3::new(0.0, 1.55, 0.0),
    );

    let source_equation_id = scene.add_tattva(
        EquationLayout::new(
            vec![
                EquationPart::new("x").with_key("x").with_color(TEAL_C),
                EquationPart::new("+").with_key("plus").with_color(GRAY_A),
                EquationPart::new("2").with_key("two").with_color(GOLD_C),
                EquationPart::new("=").with_key("eq").with_color(GRAY_A),
                EquationPart::new("5").with_key("five").with_color(BLUE_B),
            ],
            0.42,
        ),
        Vec3::new(0.0, 0.65, 0.0),
    );
    let target_equation_id = scene.add_tattva(
        EquationLayout::new(
            vec![
                EquationPart::new("x").with_key("x").with_color(TEAL_C),
                EquationPart::new("=").with_key("eq").with_color(GRAY_A),
                EquationPart::new("5").with_key("five").with_color(BLUE_B),
                EquationPart::new("-").with_key("minus").with_color(GRAY_A),
                EquationPart::new("2").with_key("two").with_color(GOLD_C),
            ],
            0.42,
        ),
        Vec3::new(0.0, 0.65, 0.0),
    );
    scene.hide(target_equation_id);

    let equation_caption_id = scene.add_tattva(
        Label::new(
            "The shared terms keep their place in the viewer's memory while only the moved term changes role.",
            0.16,
        )
        .with_color(GRAY_A),
        0.15 * DOWN,
    );

    let matrix_heading_id = scene.add_tattva(
        Label::new("Matrix Steps", 0.19).with_color(GRAY_B),
        DOWN * 0.9,
    );

    let matrix_id = scene.add_tattva(
        Matrix::new(
            vec![
                vec!["2", "-1", "0"],
                vec!["-1", "2", "-1"],
                vec!["0", "-1", "2"],
            ],
            0.44,
        ),
        DOWN * 2.1,
    );

    let matrix_caption_id = scene.add_tattva(
        Label::new(
            "Row, column, and cell highlights turn a static array into a guided explanation.",
            0.16,
        )
        .with_color(GRAY_A),
        DOWN * 4.0,
    );

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
        .animate(equation_heading_id)
        .at(1.25)
        .for_duration(0.85)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(source_equation_id)
        .at(1.7)
        .for_duration(0.5)
        .ease(Ease::OutCubic)
        .appear()
        .spawn();
    timeline
        .animate(equation_caption_id)
        .at(2.1)
        .for_duration(1.2)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(target_equation_id)
        .at(3.15)
        .for_duration(1.5)
        .ease(Ease::InOutCubic)
        .equation_continuity_from(source_equation_id)
        .spawn();
    timeline
        .animate(matrix_heading_id)
        .at(4.8)
        .for_duration(0.85)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(matrix_id)
        .at(5.15)
        .for_duration(0.5)
        .ease(Ease::OutCubic)
        .appear()
        .spawn();
    timeline
        .animate(matrix_caption_id)
        .at(5.45)
        .for_duration(1.1)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(matrix_id)
        .at(6.0)
        .for_duration(0.95)
        .ease(Ease::InOutCubic)
        .matrix_step_row(1, TEAL_C, 0.28)
        .spawn();
    timeline
        .animate(matrix_id)
        .at(7.1)
        .for_duration(0.95)
        .ease(Ease::InOutCubic)
        .matrix_step_column(1, BLUE_B, 0.24)
        .spawn();
    timeline
        .animate(matrix_id)
        .at(8.2)
        .for_duration(1.0)
        .ease(Ease::InOutCubic)
        .matrix_step_cells(vec![(0, 0), (1, 1), (2, 2)], GOLD_C, 0.24)
        .spawn();

    scene.play(timeline)?;
    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    scene.camera_mut().set_view_width(16.0);

    App::new()?.with_scene(scene).run_app()
}
