use glam::{Vec3, Vec4, vec2};
use murali::App;
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::TattvaId;
use murali::frontend::animation::Ease;
use murali::frontend::collection::primitives::{circle::Circle, line::Line, path::Path};
use murali::frontend::collection::text::label::Label;
use murali::positions::CAMERA_DEFAULT_POS;

fn add_line(
    scene: &mut Scene,
    start: (f32, f32),
    end: (f32, f32),
    thickness: f32,
    color: glam::Vec4,
) -> TattvaId {
    scene.add_tattva(
        Line::new(
            Vec3::new(start.0, start.1, 0.0),
            Vec3::new(end.0, end.1, 0.0),
            thickness,
            color,
        ),
        Vec3::ZERO,
    )
}

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    // let title_id = scene.add_tattva(
    //     Label::new("Murali Logo Study", 0.42).with_color(WHITE),
    //     Vec3::new(0.0, 3.5, 0.0),
    // );
    // let subtitle_id = scene.add_tattva(
    //     Label::new(
    //         "An experimental bezier M inside a mathematical frame, kept intentionally simple.",
    //         0.18,
    //     )
    //     .with_color(GRAY_B),
    //     Vec3::new(0.0, 2.8, 0.0),
    // );

    let frame_color = Vec4::new(0.20, 0.72, 0.98, 0.52);
    let grid_color = Vec4::new(0.20, 0.72, 0.98, 0.24);
    let axis_color = Vec4::new(0.28, 0.84, 0.95, 0.86);
    let support_color = Vec4::new(0.18, 0.58, 0.98, 0.88);
    let mark_color = Vec4::new(0.18, 0.96, 0.76, 0.96);
    let handle_color = Vec4::new(0.92, 0.98, 1.0, 0.18);

    let left = -3.2;
    let right = 3.2;
    let bottom = -2.4;
    let top = 2.4;

    let frame_ids = vec![
        add_line(&mut scene, (left, bottom), (left, top), 0.035, frame_color),
        add_line(&mut scene, (left, top), (right, top), 0.035, frame_color),
        add_line(
            &mut scene,
            (right, top),
            (right, bottom),
            0.035,
            frame_color,
        ),
        add_line(
            &mut scene,
            (right, bottom),
            (left, bottom),
            0.035,
            frame_color,
        ),
    ];

    let grid_ids = vec![
        add_line(&mut scene, (-2.4, bottom), (-2.4, top), 0.018, grid_color),
        add_line(&mut scene, (-1.6, bottom), (-1.6, top), 0.020, grid_color),
        add_line(&mut scene, (-0.8, bottom), (-0.8, top), 0.018, grid_color),
        add_line(&mut scene, (0.0, bottom), (0.0, top), 0.024, grid_color),
        add_line(&mut scene, (0.8, bottom), (0.8, top), 0.018, grid_color),
        add_line(&mut scene, (1.6, bottom), (1.6, top), 0.020, grid_color),
        add_line(&mut scene, (2.4, bottom), (2.4, top), 0.018, grid_color),
        add_line(&mut scene, (left, -1.8), (right, -1.8), 0.018, grid_color),
        add_line(&mut scene, (left, -1.2), (right, -1.2), 0.020, grid_color),
        add_line(&mut scene, (left, -0.6), (right, -0.6), 0.018, grid_color),
        add_line(&mut scene, (left, 0.0), (right, 0.0), 0.024, grid_color),
        add_line(&mut scene, (left, 0.6), (right, 0.6), 0.018, grid_color),
        add_line(&mut scene, (left, 1.2), (right, 1.2), 0.020, grid_color),
        add_line(&mut scene, (left, 1.8), (right, 1.8), 0.018, grid_color),
    ];

    let axis_ids = vec![
        add_line(&mut scene, (left, 0.0), (right, 0.0), 0.045, axis_color),
        add_line(&mut scene, (0.0, bottom), (0.0, top), 0.045, axis_color),
    ];

    let support_path_id = scene.add_tattva(
        Path::new()
            .move_to(vec2(-2.45, -1.62))
            .cubic_to(vec2(-1.85, -1.05), vec2(-0.92, -0.72), vec2(0.0, -0.72))
            .cubic_to(vec2(0.92, -0.72), vec2(1.85, -1.05), vec2(2.45, -1.62))
            .with_thickness(0.085)
            .with_color(support_color),
        Vec3::ZERO,
    );

    let support_handle_ids = vec![
        add_line(
            &mut scene,
            (-2.45, -1.62),
            (-1.85, -1.05),
            0.018,
            handle_color,
        ),
        add_line(
            &mut scene,
            (0.0, -0.72),
            (-0.92, -0.72),
            0.018,
            handle_color,
        ),
        add_line(&mut scene, (0.0, -0.72), (0.92, -0.72), 0.018, handle_color),
        add_line(
            &mut scene,
            (2.45, -1.62),
            (1.85, -1.05),
            0.018,
            handle_color,
        ),
    ];

    let mark_path_id = scene.add_tattva(
        Path::new()
            .move_to(vec2(-2.55, -1.52))
            .cubic_to(vec2(-2.62, -0.18), vec2(-2.30, 1.52), vec2(-1.72, 1.86))
            .cubic_to(vec2(-1.18, 2.12), vec2(-0.58, 0.92), vec2(0.0, 0.08))
            .cubic_to(vec2(0.58, 0.92), vec2(1.18, 2.12), vec2(1.72, 1.86))
            .cubic_to(vec2(2.30, 1.52), vec2(2.62, -0.18), vec2(2.55, -1.52))
            .with_thickness(0.115)
            .with_color(mark_color),
        Vec3::ZERO,
    );

    let mark_handle_ids = vec![
        add_line(
            &mut scene,
            (-2.55, -1.52),
            (-2.62, -0.18),
            0.02,
            handle_color,
        ),
        add_line(&mut scene, (-1.72, 1.86), (-2.30, 1.52), 0.02, handle_color),
        add_line(&mut scene, (-1.72, 1.86), (-1.18, 2.12), 0.02, handle_color),
        add_line(&mut scene, (0.0, 0.08), (-0.58, 0.92), 0.02, handle_color),
        add_line(&mut scene, (0.0, 0.08), (0.58, 0.92), 0.02, handle_color),
        add_line(&mut scene, (1.72, 1.86), (1.18, 2.12), 0.02, handle_color),
        add_line(&mut scene, (1.72, 1.86), (2.30, 1.52), 0.02, handle_color),
        add_line(&mut scene, (2.55, -1.52), (2.62, -0.18), 0.02, handle_color),
    ];

    let guide_diagonal_id = add_line(
        &mut scene,
        (-1.6, 2.4),
        (0.0, 0.0),
        0.030,
        Vec4::new(0.30, 0.70, 0.98, 0.56),
    );
    let guide_diagonal_two_id = add_line(
        &mut scene,
        (0.0, 0.0),
        (1.6, 2.4),
        0.030,
        Vec4::new(0.16, 0.94, 0.80, 0.56),
    );

    let dot_ids = vec![
        scene.add_tattva(
            Circle::new(0.11, 28, support_color).with_stroke(0.02, WHITE),
            Vec3::new(-2.45, -1.62, 0.0),
        ),
        scene.add_tattva(
            Circle::new(0.11, 28, BLUE_B).with_stroke(0.02, WHITE),
            Vec3::new(-1.6, 1.9, 0.0),
        ),
        scene.add_tattva(
            Circle::new(0.11, 28, mark_color).with_stroke(0.02, WHITE),
            Vec3::new(1.6, 1.9, 0.0),
        ),
        scene.add_tattva(
            Circle::new(0.11, 28, support_color).with_stroke(0.02, WHITE),
            Vec3::new(-1.55, -0.98, 0.0),
        ),
        scene.add_tattva(
            Circle::new(0.11, 28, support_color).with_stroke(0.02, WHITE),
            Vec3::new(1.55, -0.98, 0.0),
        ),
    ];

    let handle_dot_ids = vec![
        scene.add_tattva(
            Circle::new(0.055, 24, Vec4::new(0.92, 0.98, 1.0, 0.88)).with_stroke(0.012, WHITE),
            Vec3::new(-2.62, -0.18, 0.0),
        ),
        scene.add_tattva(
            Circle::new(0.055, 24, Vec4::new(0.92, 0.98, 1.0, 0.88)).with_stroke(0.012, WHITE),
            Vec3::new(-2.30, 1.52, 0.0),
        ),
        scene.add_tattva(
            Circle::new(0.055, 24, Vec4::new(0.92, 0.98, 1.0, 0.88)).with_stroke(0.012, WHITE),
            Vec3::new(-1.18, 2.12, 0.0),
        ),
        scene.add_tattva(
            Circle::new(0.055, 24, Vec4::new(0.92, 0.98, 1.0, 0.88)).with_stroke(0.012, WHITE),
            Vec3::new(-0.58, 0.92, 0.0),
        ),
        scene.add_tattva(
            Circle::new(0.055, 24, Vec4::new(0.92, 0.98, 1.0, 0.88)).with_stroke(0.012, WHITE),
            Vec3::new(0.58, 0.92, 0.0),
        ),
        scene.add_tattva(
            Circle::new(0.055, 24, Vec4::new(0.92, 0.98, 1.0, 0.88)).with_stroke(0.012, WHITE),
            Vec3::new(1.18, 2.12, 0.0),
        ),
        scene.add_tattva(
            Circle::new(0.055, 24, Vec4::new(0.92, 0.98, 1.0, 0.88)).with_stroke(0.012, WHITE),
            Vec3::new(2.30, 1.52, 0.0),
        ),
        scene.add_tattva(
            Circle::new(0.055, 24, Vec4::new(0.92, 0.98, 1.0, 0.88)).with_stroke(0.012, WHITE),
            Vec3::new(2.62, -0.18, 0.0),
        ),
    ];

    // let footer_id = scene.add_tattva(
    //     Label::new(
    //         "This is an exploration piece: reduce the mark until the frame and curve feel iconic.",
    //         0.17,
    //     )
    //     .with_color(GRAY_B),
    //     Vec3::new(0.0, -3.02, 0.0),
    // );

    let mut hidden_ids = Vec::new();
    hidden_ids.extend(frame_ids.iter().copied());
    hidden_ids.extend(grid_ids.iter().copied());
    hidden_ids.extend(axis_ids.iter().copied());
    hidden_ids.extend([
        support_path_id,
        mark_path_id,
        guide_diagonal_id,
        guide_diagonal_two_id,
    ]);
    hidden_ids.extend(support_handle_ids.iter().copied());
    hidden_ids.extend(mark_handle_ids.iter().copied());
    hidden_ids.extend(dot_ids.iter().copied());
    hidden_ids.extend(handle_dot_ids.iter().copied());

    for id in hidden_ids {
        scene.hide(id);
    }

    let mut timeline = Timeline::new();
    // timeline
    //     .animate(title_id)
    //     .at(0.0)
    //     .for_duration(0.95)
    //     .ease(Ease::Linear)
    //     .typewrite_text()
    //     .spawn();
    // timeline
    //     .animate(subtitle_id)
    //     .at(0.3)
    //     .for_duration(1.45)
    //     .ease(Ease::Linear)
    //     .typewrite_text()
    //     .spawn();

    for (idx, id) in frame_ids.iter().enumerate() {
        timeline
            .animate(*id)
            .at(1.15 + idx as f32 * 0.12)
            .for_duration(0.18)
            .ease(Ease::Linear)
            .appear()
            .spawn();
    }

    for (idx, id) in grid_ids.iter().enumerate() {
        timeline
            .animate(*id)
            .at(1.8 + idx as f32 * 0.1)
            .for_duration(0.16)
            .ease(Ease::Linear)
            .appear()
            .spawn();
    }

    for (idx, id) in axis_ids.iter().enumerate() {
        timeline
            .animate(*id)
            .at(2.45 + idx as f32 * 0.12)
            .for_duration(0.18)
            .ease(Ease::Linear)
            .appear()
            .spawn();
    }

    timeline
        .animate(guide_diagonal_id)
        .at(2.9)
        .for_duration(0.18)
        .ease(Ease::Linear)
        .appear()
        .spawn();
    timeline
        .animate(guide_diagonal_two_id)
        .at(3.05)
        .for_duration(0.18)
        .ease(Ease::Linear)
        .appear()
        .spawn();

    for (idx, id) in support_handle_ids.iter().enumerate() {
        timeline
            .animate(*id)
            .at(3.0 + idx as f32 * 0.08)
            .for_duration(0.16)
            .ease(Ease::Linear)
            .appear()
            .spawn();
    }

    for (idx, id) in mark_handle_ids.iter().enumerate() {
        timeline
            .animate(*id)
            .at(3.25 + idx as f32 * 0.06)
            .for_duration(0.16)
            .ease(Ease::Linear)
            .appear()
            .spawn();
    }

    timeline
        .animate(support_path_id)
        .at(3.2)
        .for_duration(1.2)
        .ease(Ease::OutCubic)
        .draw()
        .spawn();
    timeline
        .animate(mark_path_id)
        .at(3.55)
        .for_duration(1.85)
        .ease(Ease::OutCubic)
        .draw()
        .spawn();

    for (idx, id) in dot_ids.iter().enumerate() {
        timeline
            .animate(*id)
            .at(4.0 + idx as f32 * 0.18)
            .for_duration(0.2)
            .ease(Ease::Linear)
            .appear()
            .spawn();
    }

    for (idx, id) in handle_dot_ids.iter().enumerate() {
        timeline
            .animate(*id)
            .at(4.2 + idx as f32 * 0.08)
            .for_duration(0.16)
            .ease(Ease::Linear)
            .appear()
            .spawn();
    }

    // timeline
    //     .animate(footer_id)
    //     .at(5.1)
    //     .for_duration(1.55)
    //     .ease(Ease::Linear)
    //     .typewrite_text()
    //     .spawn();

    timeline.wait_until(6.6);
    scene.play(timeline)?;
    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    scene.camera_mut().set_view_width(16.0);

    scene.capture_screenshots_named([(6.5, Some("murali_default_logo.png"))]);

    App::new()?.with_scene(scene).run_app()
}
