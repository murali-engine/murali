use glam::Vec3;
use murali::App;
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::engine::timeline::{SignalPlayback, Timeline};
use murali::frontend::animation::Ease;
use murali::frontend::collection::ai::{
    neural_network_diagram::{IndicationStyle, NeuralNetworkDiagram},
    signal_flow::SignalFlow,
};
use murali::frontend::collection::text::label::Label;
use murali::frontend::layout::Direction;
use murali::positions::CAMERA_DEFAULT_POS;

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    let title_id = scene.add_tattva(
        Label::new("Neural Networks", 0.38).with_color(WHITE),
        Vec3::ZERO,
    );
    scene.to_edge(title_id, Direction::Up, 0.8);

    let subtitle_id = scene.add_tattva(
        Label::new(
            "One network, one clean forward pass, then a second playback pattern over the same structure.",
            0.18,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, 2.95, 0.0),
    );

    let heading_id = scene.add_tattva(
        Label::new("Signal flow through layers", 0.2).with_color(GRAY_B),
        Vec3::new(0.0, 2.3, 0.0),
    );

    let diagram = NeuralNetworkDiagram::new(vec![3, 5, 4, 2])
        .with_layer_spacing(1.7)
        .with_node_spacing(0.58)
        .with_node_radius(0.11)
        .with_labels(vec!["Input", "Hidden", "Hidden", "Output"])
        .with_indication_style(IndicationStyle::Single)
        .deactivate_node(1, 4)
        .deactivate_node(2, 0);
    let flow_paths = diagram.all_path_points();

    let diagram_id = scene.add_tattva(diagram, Vec3::new(0.0, 0.15, 0.0));
    let flow_id = scene.add_tattva(
        {
            let mut flow = SignalFlow::from_paths(flow_paths)
                .with_progress(0.0)
                .with_edge_color(GOLD_C)
                .with_pulse_color(GOLD_A);
            flow.highlight_nodes = false;
            flow.node_radius = 0.0;
            flow.edge_thickness = 0.04;
            flow.pulse_radius = 0.09;
            flow
        },
        Vec3::new(0.0, 0.15, 0.0),
    );
    scene.hide(flow_id);

    let caption_id = scene.add_tattva(
        Label::new(
            "Inactive nodes stay dim while the active routes carry the signal.",
            0.17,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, -2.55, 0.0),
    );

    let footer_id = scene.add_tattva(
        Label::new(
            "Start with structure and playback before introducing training or model internals.",
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
        .animate(heading_id)
        .at(1.5)
        .for_duration(0.8)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(diagram_id)
        .at(1.9)
        .for_duration(0.8)
        .ease(Ease::Linear)
        .appear()
        .spawn();
    timeline
        .animate(caption_id)
        .at(2.4)
        .for_duration(1.1)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(flow_id)
        .at(2.75)
        .for_duration(0.2)
        .ease(Ease::Linear)
        .appear()
        .spawn();

    timeline.play_signal(flow_id, SignalPlayback::once(2.8, 2.0, Ease::InOutQuad));
    timeline.play_signal(
        flow_id,
        SignalPlayback::round_trip(5.4, 1.5, Ease::InOutQuad),
    );

    timeline
        .animate(footer_id)
        .at(6.6)
        .for_duration(1.6)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    scene.play(timeline)?;
    scene.camera_mut().position = CAMERA_DEFAULT_POS;

    App::new()?.with_scene(scene).run_app()
}
