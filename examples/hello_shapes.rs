use glam::Vec3;
use murali::App;
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::animation::Ease;
use murali::frontend::layout::Direction;
use murali::frontend::sangrah::primitives::{
    circle::Circle, polygon::Polygon, rectangle::Rectangle, square::Square,
};
use murali::frontend::sangrah::text::label::Label;
use murali::positions::CAMERA_DEFAULT_POS;

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    let title_id = scene.add_tattva(
        Label::new("Hello Shapes", 0.38).with_color(WHITE),
        Vec3::ZERO,
    );
    scene.to_edge(title_id, Direction::Up, 0.8);

    let subtitle_id = scene.add_tattva(
        Label::new(
            "A first scene with a few core primitives placed by hand.",
            0.18,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, 2.95, 0.0),
    );

    let square_id = scene.add_tattva(
        Square::new(1.25, RED_B).with_stroke(0.04, WHITE),
        Vec3::new(-5.2, 0.4, 0.0),
    );
    let square_label_id = scene.add_tattva(
        Label::new("Square", 0.2).with_color(GRAY_A),
        Vec3::new(-5.2, -1.0, 0.0),
    );

    let circle_id = scene.add_tattva(
        Circle::new(0.7, 48, GREEN_D).with_stroke(0.04, WHITE),
        Vec3::new(-1.8, 0.4, 0.0),
    );
    let circle_label_id = scene.add_tattva(
        Label::new("Circle", 0.2).with_color(GRAY_A),
        Vec3::new(-1.8, -1.0, 0.0),
    );

    let rectangle_id = scene.add_tattva(
        Rectangle::new(1.9, 1.05, BLUE_D).with_stroke(0.04, WHITE),
        Vec3::new(1.9, 0.4, 0.0),
    );
    let rectangle_label_id = scene.add_tattva(
        Label::new("Rectangle", 0.2).with_color(GRAY_A),
        Vec3::new(1.9, -1.0, 0.0),
    );

    let polygon_id = scene.add_tattva(
        Polygon::regular(6, 0.8, GOLD_C).with_stroke(0.04, WHITE),
        Vec3::new(5.3, 0.4, 0.0),
    );
    let polygon_label_id = scene.add_tattva(
        Label::new("Polygon", 0.2).with_color(GRAY_A),
        Vec3::new(5.3, -1.0, 0.0),
    );

    let footer_id = scene.add_tattva(
        Label::new(
            "This example is intentionally simple: primitives first, composition later.",
            0.17,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, -2.8, 0.0),
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
        .for_duration(1.3)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    for (index, (shape_id, label_id)) in [
        (square_id, square_label_id),
        (circle_id, circle_label_id),
        (rectangle_id, rectangle_label_id),
        (polygon_id, polygon_label_id),
    ]
    .into_iter()
    .enumerate()
    {
        let start = 1.5 + index as f32 * 1.2;
        timeline
            .animate(shape_id)
            .at(start)
            .for_duration(0.95)
            .ease(Ease::OutCubic)
            .draw()
            .spawn();
        timeline
            .animate(label_id)
            .at(start + 0.45)
            .for_duration(0.7)
            .ease(Ease::Linear)
            .typewrite_text()
            .spawn();
    }

    timeline
        .animate(footer_id)
        .at(6.6)
        .for_duration(1.8)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    scene.play(timeline)?;

    scene.camera_mut().position = CAMERA_DEFAULT_POS;

    App::new()?.with_scene(scene).run_app()
}
