use glam::{Vec3, Vec4};
use murali::App;
use murali::colors::{BLUE_D, GREEN_D, WHITE};
use murali::engine::frame::Frame;
use murali::engine::render::RenderOptions;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::animation::Ease;
use murali::frontend::collection::primitives::circle::Circle;
use murali::frontend::collection::text::label::Label;
use murali::frontend::layout::Direction;

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new().with_frame(Frame::portrait());

    let title = scene.add_tattva(
        Label::new("Portrait Video", 0.55).with_color(WHITE),
        Vec3::ZERO,
    );
    scene.to_edge(title, Direction::Up, 0.8);

    let first = scene.add_tattva(
        Circle::new(1.25, 64, BLUE_D).with_stroke(0.05, WHITE),
        Vec3::new(0.0, 3.0, 0.0),
    );
    let second = scene.add_tattva(
        Circle::new(1.25, 64, GREEN_D).with_stroke(0.05, WHITE),
        Vec3::new(0.0, -1.0, 0.0),
    );
    let footer = scene.add_tattva(
        Label::new("9:16 composition, ordinary world coordinates", 0.28)
            .with_color(Vec4::new(0.78, 0.82, 0.88, 1.0)),
        Vec3::ZERO,
    );
    scene.to_edge(footer, Direction::Down, 0.8);

    let mut timeline = Timeline::new();
    timeline
        .animate(title)
        .at(0.0)
        .for_duration(0.8)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(first)
        .at(0.7)
        .for_duration(1.0)
        .ease(Ease::OutCubic)
        .draw()
        .spawn();
    timeline
        .animate(second)
        .at(1.5)
        .for_duration(1.0)
        .ease(Ease::OutCubic)
        .draw()
        .spawn();
    timeline
        .animate(footer)
        .at(2.2)
        .for_duration(1.0)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    scene.play(timeline)?;

    App::new()?
        .with_scene(scene)
        .with_render_options(RenderOptions {
            width: Some(1080),
            ..RenderOptions::default()
        })
        .run_app()
}
