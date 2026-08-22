use glam::Vec3;
use murali::colors::*;
use murali::engine::camera::Projection;
use murali::engine::scene::Scene;
use murali::engine::timeline::{SignalPlayback, Timeline};
use murali::frontend::animation::Ease;
use murali::frontend::collection::ai::ml_components::signal_flow::SignalFlow;
use murali::frontend::collection::composite::axes3d::Axes3D;
use murali::frontend::collection::maths::calculus::parametric_curve3d::ParametricCurve3D;
use murali::frontend::collection::text::label::Label;
use murali::frontend::layout::Direction;
use murali::{App, DepthMode};

fn space_curve(t: f32) -> Vec3 {
    Vec3::new(
        1.7 * (0.9 * t).cos(),
        0.85 * (1.4 * t).sin(),
        -1.5 + 0.48 * t + 0.22 * (1.1 * t).cos(),
    )
}

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    let title_id = scene.add_tattva(Label::new("3D Curves", 0.38).with_color(WHITE), Vec3::ZERO);
    scene.to_edge(title_id, Direction::Up, 0.8);

    let subtitle_id = scene.add_tattva(
        Label::new(
            "A single parametric space curve with 3D axes and a few camera frames to reveal its shape.",
            0.18,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, 2.95, 0.0),
    );

    let axes_id = scene.add_tattva(
        Axes3D::new((-2.8, 2.8), (-2.2, 2.2), (-2.0, 2.0))
            .with_step(1.0)
            .with_axis_thickness(0.03)
            .with_tick_size(0.14),
        Vec3::ZERO,
    );

    let curve = ParametricCurve3D::new((0.0, 6.4), space_curve).with_samples(240);
    let curve_points = curve.sample_points();
    let trace_id = scene.add_tattva(
        {
            let mut flow = SignalFlow::new(curve_points)
                .with_progress(0.0)
                .with_edge_color(GOLD_C)
                .with_pulse_color(GOLD_A);
            flow.highlight_nodes = false;
            flow.node_radius = 0.0;
            flow.edge_thickness = 0.04;
            flow.pulse_radius = 0.09;
            flow
        },
        Vec3::ZERO,
    );

    let x_label_id = scene.add_tattva(
        Label::new("x", 0.2).with_color(ORANGE_B),
        Vec3::new(3.25, 0.0, 0.0),
    );
    let y_label_id = scene.add_tattva(
        Label::new("y", 0.2).with_color(BLUE_B),
        Vec3::new(0.0, 2.55, 0.0),
    );
    let z_label_id = scene.add_tattva(
        Label::new("z", 0.2).with_color(GOLD_C),
        Vec3::new(0.0, 0.0, 2.55),
    );
    let footer_id = scene.add_tattva(
        Label::new(
            "Use camera framing to help the eye understand depth before introducing surfaces.",
            0.17,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, -3.05, 0.0),
    );

    scene.camera_mut().projection = Projection::Perspective {
        fov_y_rad: 45.0_f32.to_radians(),
        aspect: 16.0 / 9.0,
        near: 0.1,
        far: 100.0,
    };
    scene.camera_mut().position = Vec3::new(0.0, 0.8, 12.8);
    scene.camera_mut().target = Vec3::new(0.0, 0.1, 0.0);
    for id in [title_id, subtitle_id, footer_id] {
        scene.set_depth_mode(id, DepthMode::Overlay);
    }

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
        .animate(axes_id)
        .at(1.5)
        .for_duration(1.0)
        .ease(Ease::Linear)
        .appear()
        .spawn();
    timeline
        .animate(x_label_id)
        .at(1.95)
        .for_duration(0.5)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(y_label_id)
        .at(2.1)
        .for_duration(0.5)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(z_label_id)
        .at(2.25)
        .for_duration(0.5)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    timeline.play_signal(trace_id, SignalPlayback::once(2.5, 3.2, Ease::InOutCubic));

    timeline
        .animate_camera()
        .at(0.0)
        .for_duration(2.4)
        .ease(Ease::InOutQuad)
        .frame_to(Vec3::new(-1.8, 2.6, 11.8), Vec3::new(0.0, 0.1, 0.0))
        .spawn();
    timeline
        .animate_camera()
        .at(2.4)
        .for_duration(2.6)
        .ease(Ease::InOutQuad)
        .frame_to(Vec3::new(2.2, 1.3, 10.8), Vec3::new(0.1, 0.1, 0.2))
        .spawn();
    timeline
        .animate_camera()
        .at(5.0)
        .for_duration(2.0)
        .ease(Ease::OutQuad)
        .frame_to(Vec3::new(0.4, 3.8, 11.6), Vec3::new(0.0, 0.2, 0.2))
        .spawn();

    timeline
        .animate(footer_id)
        .at(6.2)
        .for_duration(1.5)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    scene.play(timeline)?;

    App::new()?.with_scene(scene).run_app()
}
