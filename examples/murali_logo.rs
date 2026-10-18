use glam::{Vec3, Vec4, vec2};
use murali::App;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::animation::Ease;
use murali::frontend::collection::primitives::path::Path;
use murali::frontend::style::Style;
use murali::positions::CAMERA_DEFAULT_POS;

const MURALI_CORAL: Vec4 = Vec4::new(1.0, 107.0 / 255.0, 95.0 / 255.0, 1.0);

fn murali_mark() -> Path {
    Path::new()
        .move_to(vec2(-2.60, -1.70))
        .line_to(vec2(-2.60, 1.45))
        .quad_to(vec2(-2.60, 1.80), vec2(-2.25, 1.80))
        .line_to(vec2(-1.85, 1.80))
        .quad_to(vec2(-1.65, 1.80), vec2(-1.50, 1.62))
        .line_to(vec2(0.00, 0.05))
        .line_to(vec2(1.50, 1.62))
        .quad_to(vec2(1.65, 1.80), vec2(1.85, 1.80))
        .line_to(vec2(2.25, 1.80))
        .quad_to(vec2(2.60, 1.80), vec2(2.60, 1.45))
        .line_to(vec2(2.60, -1.70))
        .line_to(vec2(1.62, -1.70))
        .line_to(vec2(1.62, 0.14))
        .line_to(vec2(0.42, -1.12))
        .quad_to(vec2(0.00, -1.56), vec2(-0.42, -1.12))
        .line_to(vec2(-1.62, 0.14))
        .line_to(vec2(-1.62, -1.70))
        .close()
        .with_style(Style::new().with_fill(MURALI_CORAL))
}

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();
    let mark = scene.add_tattva(murali_mark(), Vec3::ZERO);

    scene.set_opacity(mark, 0.0);
    let mut timeline = Timeline::new();
    timeline
        .animate(mark)
        .at(0.15)
        .for_duration(0.7)
        .ease(Ease::OutCubic)
        .appear()
        .spawn();
    timeline.wait_until(1.0);
    scene.play(timeline)?;

    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    scene.camera_mut().set_view_width(7.2);
    scene.capture_screenshots_named([(0.95, Some("murali_default_logo.png"))]);

    App::new()?.with_scene(scene).run_app()
}
