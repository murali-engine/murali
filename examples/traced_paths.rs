use glam::{Quat, Vec3, Vec4, vec2, vec3};
use murali::App;
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::animation::Ease;
use murali::frontend::sangrah::primitives::{circle::Circle, line::Line};
use murali::frontend::sangrah::text::label::Label;
use murali::frontend::sangrah::utility::TracedPath;
use murali::positions::CAMERA_DEFAULT_POS;

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    let title_id = scene.add_tattva(
        Label::new("Traced Paths", 0.38).with_color(WHITE),
        Vec3::new(0.0, 3.0, 0.0),
    );
    let subtitle_id = scene.add_tattva(
        Label::new(
            "A traced path becomes meaningful when the underlying motion is simple enough to understand first.",
            0.18,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, 2.45, 0.0),
    );

    let heading_id = scene.add_tattva(
        Label::new("Rolling point on a wheel", 0.19).with_color(GRAY_B),
        Vec3::new(0.0, 1.7, 0.0),
    );

    let ground_y = -1.15;
    let radius = 0.55;
    let start_time = 2.2;
    let duration = 5.4;
    let start_x = -3.45;
    let total_theta = std::f32::consts::TAU * 2.0;

    let ground_id = scene.add_tattva(
        Line::new(
            Vec3::new(-4.85, ground_y, 0.0),
            Vec3::new(4.85, ground_y, 0.0),
            0.04,
            Vec4::new(0.38, 0.80, 0.96, 0.35),
        ),
        Vec3::ZERO,
    );

    let wheel_id = scene.add_tattva(
        Circle::new(radius, 72, Vec4::new(0.0, 0.0, 0.0, 0.0)).with_stroke(0.05, TEAL_C),
        Vec3::new(start_x, ground_y + radius, 0.0),
    );
    let hub_id = scene.add_tattva(
        Circle::new(0.06, 28, GOLD_C).with_stroke(0.02, WHITE),
        Vec3::new(start_x, ground_y + radius, 0.0),
    );
    let tracer_dot_id = scene.add_tattva(
        Circle::new(0.08, 28, BLUE_B).with_stroke(0.02, WHITE),
        Vec3::new(start_x, ground_y, 0.0),
    );

    scene.add_tattva(
        TracedPath::new(
            wheel_id,
            move |obj_pos, obj_rot| obj_pos + obj_rot * vec3(0.0, -radius, 0.0),
            GOLD_C,
            0.06,
        )
        .with_min_distance(0.02)
        .with_max_points(6000),
        Vec3::ZERO,
    );

    let caption_id = scene.add_tattva(
        Label::new(
            "The path is not separate decoration: it is the history of one chosen point as the wheel rolls forward.",
            0.16,
        )
        .with_color(GRAY_A),
        Vec3::new(0.0, -2.95, 0.0),
    );

    scene.add_updater(wheel_id, move |scene, _, _dt| {
        let elapsed = (scene.scene_time - start_time).clamp(0.0, duration);
        let progress = if duration <= 0.0 {
            1.0
        } else {
            elapsed / duration
        };
        let theta = progress * total_theta;
        let center = vec2(start_x + radius * theta, ground_y + radius);
        let rotation = Quat::from_rotation_z(-theta);
        let traced_point = center + (rotation * vec3(0.0, -radius, 0.0)).truncate();

        scene.set_position_2d(wheel_id, center);
        scene.set_rotation(wheel_id, rotation);
        scene.set_position_2d(hub_id, center);
        scene.set_position_2d(tracer_dot_id, traced_point);
    });

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
        .animate(heading_id)
        .at(1.15)
        .for_duration(0.8)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(ground_id)
        .at(1.45)
        .for_duration(0.9)
        .ease(Ease::OutCubic)
        .draw()
        .spawn();
    timeline
        .animate(wheel_id)
        .at(1.8)
        .for_duration(0.55)
        .ease(Ease::OutCubic)
        .appear()
        .spawn();
    timeline
        .animate(hub_id)
        .at(1.95)
        .for_duration(0.45)
        .ease(Ease::OutCubic)
        .appear()
        .spawn();
    timeline
        .animate(tracer_dot_id)
        .at(2.05)
        .for_duration(0.45)
        .ease(Ease::OutCubic)
        .appear()
        .spawn();
    timeline
        .animate(caption_id)
        .at(2.55)
        .for_duration(1.2)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate_camera()
        .at(2.2)
        .for_duration(2.8)
        .ease(Ease::InOutQuad)
        .zoom_to(1.1)
        .spawn();
    timeline
        .animate_camera()
        .at(5.2)
        .for_duration(2.2)
        .ease(Ease::InOutQuad)
        .zoom_to(1.22)
        .spawn();

    scene.play(timeline)?;
    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    scene.camera_mut().set_view_width(16.0);

    App::new()?.with_scene(scene).run_app()
}
