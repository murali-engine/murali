use glam::{Vec3, vec2};
use murali::App;
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::animation::Ease;
use murali::frontend::layout::Direction;
use murali::frontend::sangrah::composite::{axes::Axes, number_plane::NumberPlane};
use murali::frontend::sangrah::ganit::data_geometry::scatter_plot::ScatterPlot;
use murali::frontend::sangrah::ganit::graph::{PlotLegend, PlotLegendEntry};
use murali::frontend::sangrah::primitives::path::Path;
use murali::frontend::sangrah::text::label::Label;
use murali::positions::CAMERA_DEFAULT_POS;
use std::f32::consts::{FRAC_PI_2, PI};

fn sine(x: f32) -> f32 {
    x.sin()
}

fn sine_path(x_range: (f32, f32), samples: usize) -> Path {
    let samples = samples.max(2);
    let step = (x_range.1 - x_range.0) / (samples - 1) as f32;
    let mut path = Path::new()
        .move_to(vec2(x_range.0, sine(x_range.0)))
        .with_thickness(0.06)
        .with_color(BLUE_B);

    for i in 1..samples {
        let x = x_range.0 + i as f32 * step;
        path = path.line_to(vec2(x, sine(x)));
    }

    path
}

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    let title_id = scene.add_tattva(Label::new("2D Graphs", 0.38).with_color(WHITE), Vec3::ZERO);
    scene.to_edge(title_id, Direction::Up, 0.8);

    let subtitle_id = scene.add_tattva(
        Label::new(
            "A simple graphing scene with a plane, axes, one sine wave, sampled points, and labels.",
            0.18,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, 2.95, 0.0),
    );

    let plane_id = scene.add_tattva(
        NumberPlane::new((-7.0, 7.0), (-1.8, 1.8)).with_step(1.0),
        Vec3::new(0.0, -0.1, 0.0),
    );
    let axes_id = scene.add_tattva(
        Axes::new((-7.0, 7.0), (-1.8, 1.8))
            .with_step(1.0)
            .with_thickness(0.028)
            .with_tick_size(0.14)
            .with_color(GRAY_A),
        Vec3::new(0.0, -0.1, 0.0),
    );

    let graph_id = scene.add_tattva(
        sine_path((-2.0 * PI, 2.0 * PI), 420),
        Vec3::new(0.0, -0.1, 0.0),
    );
    let points_id = scene.add_tattva(
        ScatterPlot::new(vec![
            vec2(-3.0 * FRAC_PI_2, sine(-3.0 * FRAC_PI_2)),
            vec2(-FRAC_PI_2, sine(-FRAC_PI_2)),
            vec2(FRAC_PI_2, sine(FRAC_PI_2)),
            vec2(3.0 * FRAC_PI_2, sine(3.0 * FRAC_PI_2)),
        ]),
        Vec3::new(0.0, -0.1, 0.0),
    );

    let equation_id = scene.add_tattva(
        Label::new("y = sin(x)", 0.22).with_color(BLUE_B),
        Vec3::new(0.0, 2.0, 0.0),
    );
    let legend_id = scene.add_tattva(
        PlotLegend::new(vec![
            PlotLegendEntry::new("sine curve", BLUE_B),
            PlotLegendEntry::new("sampled extrema", GOLD_C),
        ])
        .with_text_color(GRAY_B),
        Vec3::new(4.4, 1.45, 0.0),
    );
    let x_label_id = scene.add_tattva(
        Label::new("x", 0.22).with_color(ORANGE_B),
        Vec3::new(4.45, -0.15, 0.0),
    );
    let y_label_id = scene.add_tattva(
        Label::new("y", 0.22).with_color(BLUE_B),
        Vec3::new(0.15, 2.45, 0.0),
    );
    let footer_id = scene.add_tattva(
        Label::new(
            "Start with one function and a few observations before layering more graphing ideas.",
            0.17,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, -3.1, 0.0),
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
        .animate(plane_id)
        .at(1.4)
        .for_duration(1.0)
        .ease(Ease::Linear)
        .appear()
        .spawn();
    timeline
        .animate(axes_id)
        .at(1.7)
        .for_duration(0.9)
        .ease(Ease::Linear)
        .appear()
        .spawn();

    timeline
        .animate(equation_id)
        .at(2.6)
        .for_duration(1.0)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(legend_id)
        .at(2.75)
        .for_duration(0.55)
        .ease(Ease::OutCubic)
        .appear()
        .spawn();
    timeline
        .animate(x_label_id)
        .at(2.9)
        .for_duration(0.6)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(y_label_id)
        .at(3.1)
        .for_duration(0.6)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    timeline
        .animate(graph_id)
        .at(3.5)
        .for_duration(10.0)
        .ease(Ease::Linear)
        .draw()
        .spawn();
    timeline
        .animate(points_id)
        .at(13.0)
        .for_duration(1.0)
        .ease(Ease::Linear)
        .appear()
        .spawn();
    timeline
        .animate(footer_id)
        .at(13.6)
        .for_duration(1.6)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    scene.play(timeline)?;
    scene.camera_mut().position = CAMERA_DEFAULT_POS;

    App::new()?.with_scene(scene).run_app()
}
