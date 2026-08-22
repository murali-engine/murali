use glam::{Vec3, vec2};
use murali::App;
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::animation::Ease;
use murali::frontend::collection::primitives::{
    arrow::Arrow, circle::Circle, line::Line, path::Path, rectangle::Rectangle,
};
use murali::frontend::collection::text::label::Label;
use murali::frontend::style::{ColorSource, StrokeParams, Style};
use murali::positions::CAMERA_DEFAULT_POS;

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    let title_id = scene.add_tattva(
        Label::new("Style And Paths", 0.38).with_color(WHITE),
        Vec3::new(0.0, 3.0, 0.0),
    );
    let subtitle_id = scene.add_tattva(
        Label::new(
            "Fill, stroke, dashes, arrows, and one authored path without mixing in unrelated animation ideas.",
            0.18,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, 2.45, 0.0),
    );

    let style_heading_id = scene.add_tattva(
        Label::new("Fill And Stroke", 0.19).with_color(GRAY_B),
        Vec3::new(-5.0, 1.65, 0.0),
    );
    let style_card_id = scene.add_tattva(
        Rectangle::new(2.35, 1.5, WHITE).with_style(
            Style::new()
                .with_fill(ColorSource::linear_gradient(
                    vec2(-1.15, -0.75),
                    vec2(1.15, 0.75),
                    vec![(0.0, BLUE_D), (0.55, TEAL_C), (1.0, GREEN_D)],
                ))
                .with_stroke(StrokeParams {
                    thickness: 0.05,
                    color: WHITE,
                    ..Default::default()
                }),
        ),
        Vec3::new(-5.0, 0.35, 0.0),
    );
    let style_badge_id = scene.add_tattva(
        Circle::new(0.28, 36, GOLD_C).with_stroke(0.03, WHITE),
        Vec3::new(-4.2, -0.05, 0.0),
    );
    let style_caption_id = scene.add_tattva(
        Label::new(
            "A surface can carry both a fill story and a clear edge.",
            0.16,
        )
        .with_color(GRAY_A),
        Vec3::new(-5.0, -1.45, 0.0),
    );

    let dash_heading_id = scene.add_tattva(
        Label::new("Dashes And Direction", 0.19).with_color(GRAY_B),
        Vec3::new(0.0, 1.65, 0.0),
    );
    let dashed_line_id = scene.add_tattva(
        Line::new(
            Vec3::new(-1.35, 0.65, 0.0),
            Vec3::new(1.2, -0.55, 0.0),
            0.06,
            TEAL_C,
        )
        .with_dash(0.18, 0.10),
        Vec3::ZERO,
    );
    let arrow_id = scene.add_tattva(
        Arrow::with_default_tip(vec2(-1.15, -0.85), vec2(1.25, 0.8), 0.055, GOLD_C),
        Vec3::ZERO,
    );
    let dash_caption_id = scene.add_tattva(
        Label::new(
            "Use dashes for guides and arrows when motion needs a destination.",
            0.16,
        )
        .with_color(GRAY_A),
        Vec3::new(0.0, -2.05, 0.0),
    );

    let path_heading_id = scene.add_tattva(
        Label::new("Authored Path", 0.19).with_color(GRAY_B),
        Vec3::new(5.0, 1.65, 0.0),
    );
    let path_id = scene.add_tattva(
        Path::new()
            .move_to(vec2(3.55, -0.65))
            .line_to(vec2(4.05, 0.45))
            .quad_to(vec2(4.55, 1.05), vec2(5.2, 0.25))
            .cubic_to(vec2(5.55, -0.25), vec2(6.05, -0.95), vec2(6.4, 0.15))
            .with_thickness(0.085)
            .with_color(PURPLE_B),
        Vec3::ZERO,
    );
    let path_dot_a_id = scene.add_tattva(
        Circle::new(0.09, 28, BLUE_B).with_stroke(0.02, WHITE),
        Vec3::new(3.55, -0.65, 0.0),
    );
    let path_dot_b_id = scene.add_tattva(
        Circle::new(0.09, 28, TEAL_C).with_stroke(0.02, WHITE),
        Vec3::new(5.2, 0.25, 0.0),
    );
    let path_dot_c_id = scene.add_tattva(
        Circle::new(0.09, 28, GOLD_C).with_stroke(0.02, WHITE),
        Vec3::new(6.4, 0.15, 0.0),
    );
    let path_caption_id = scene.add_tattva(
        Label::new(
            "Lines, quadratics, and cubics let you author one continuous gesture.",
            0.16,
        )
        .with_color(GRAY_A),
        Vec3::new(5.0, -1.45, 0.0),
    );

    let footer_id = scene.add_tattva(
        Label::new(
            "This example is about visual language: how marks feel before they start moving.",
            0.17,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, -3.0, 0.0),
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
        .for_duration(1.5)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    timeline
        .animate(style_heading_id)
        .at(1.4)
        .for_duration(0.8)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(style_card_id)
        .at(1.85)
        .for_duration(0.7)
        .ease(Ease::OutCubic)
        .appear()
        .spawn();
    timeline
        .animate(style_badge_id)
        .at(2.2)
        .for_duration(0.55)
        .ease(Ease::OutCubic)
        .appear()
        .spawn();
    timeline
        .animate(style_caption_id)
        .at(2.35)
        .for_duration(1.0)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    timeline
        .animate(dash_heading_id)
        .at(3.25)
        .for_duration(0.8)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(dashed_line_id)
        .at(3.7)
        .for_duration(0.9)
        .ease(Ease::OutCubic)
        .draw()
        .spawn();
    timeline
        .animate(arrow_id)
        .at(4.05)
        .for_duration(0.55)
        .ease(Ease::OutCubic)
        .appear()
        .spawn();
    timeline
        .animate(dash_caption_id)
        .at(4.15)
        .for_duration(1.1)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    timeline
        .animate(path_heading_id)
        .at(5.1)
        .for_duration(0.8)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(path_id)
        .at(5.55)
        .for_duration(1.4)
        .ease(Ease::OutCubic)
        .draw()
        .spawn();
    for (dot_id, start) in [
        (path_dot_a_id, 5.85),
        (path_dot_b_id, 6.25),
        (path_dot_c_id, 6.6),
    ] {
        timeline
            .animate(dot_id)
            .at(start)
            .for_duration(0.4)
            .ease(Ease::OutCubic)
            .appear()
            .spawn();
    }
    timeline
        .animate(path_caption_id)
        .at(6.0)
        .for_duration(1.15)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    timeline
        .animate(footer_id)
        .at(7.1)
        .for_duration(1.5)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    scene.play(timeline)?;
    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    scene.camera_mut().set_view_width(16.0);

    App::new()?.with_scene(scene).run_app()
}
