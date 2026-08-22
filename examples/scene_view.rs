use glam::{Quat, Vec3, Vec4, vec2, vec3};
use murali::colors::{
    BLUE_B, BLUE_D, GOLD_C, GRAY_A, GRAY_B, GRAY_C, GREEN_C, ORANGE_C, PINK_C, PURPLE_B, TEAL_C,
    WHITE,
};
use murali::engine::export::{ExportSettings, PngCompressionMode};
use murali::frontend::TattvaId;
use murali::frontend::animation::Ease;
use murali::frontend::collection::primitives::{
    circle::Circle, line::Line, rounded_rectangle::RoundedRectangle,
};
use murali::frontend::collection::text::label::Label;
use murali::frontend::props::layers;
use murali::{App, Scene, SceneView, SceneViewPlayback, Timeline};

fn alpha(color: Vec4, opacity: f32) -> Vec4 {
    Vec4::new(color.x, color.y, color.z, opacity)
}

fn add_layer_panel(scene: &mut Scene, x: f32, color: Vec4) {
    let panel = scene.add_tattva(
        RoundedRectangle::new(2.05, 5.55, 0.24, alpha(color, 0.08))
            .with_stroke(0.025, alpha(color, 0.34)),
        vec3(x, -0.15, -0.2),
    );
    scene.set_layer(panel, layers::BACKGROUND);
}

fn connect_layers(scene: &mut Scene, from: &[Vec3], to: &[Vec3], color: Vec4) {
    for (from_index, start) in from.iter().enumerate() {
        for offset in 0..2.min(to.len()) {
            let end = to[(from_index + offset) % to.len()];
            let line = scene.add_tattva(
                Line::new(*start, end, 0.024, alpha(color, 0.28)),
                Vec3::ZERO,
            );
            scene.set_layer(line, layers::BACKGROUND + 1);
        }
    }
}

fn add_network_node(scene: &mut Scene, position: Vec3, color: Vec4) {
    scene.add_tattva(Circle::new(0.34, 40, alpha(color, 0.13)), position);
    scene.add_tattva(
        Circle::new(0.22, 36, color).with_stroke(0.035, alpha(WHITE, 0.9)),
        vec3(position.x, position.y, position.z + 0.02),
    );
}

fn animate_activation(timeline: &mut Timeline, pulse: TattvaId, route: &[Vec3], start_time: f32) {
    timeline
        .animate(pulse)
        .at(start_time)
        .for_duration(0.12)
        .ease(Ease::OutCubic)
        .appear()
        .spawn();
    for (hop, target) in route.iter().skip(1).enumerate() {
        timeline
            .animate(pulse)
            .at(start_time + hop as f32 * 0.55)
            .for_duration(0.55)
            .ease(Ease::InOutCubic)
            .move_to(*target + vec3(0.0, 0.0, 0.25))
            .spawn();
    }
    timeline
        .animate(pulse)
        .at(start_time + (route.len() - 1) as f32 * 0.55 - 0.18)
        .for_duration(0.3)
        .ease(Ease::OutCubic)
        .fade_to(0.0)
        .spawn();
}

fn hand_built_transformer() -> anyhow::Result<Scene> {
    let mut scene = Scene::new();
    let columns = [
        vec![
            vec3(-5.5, -1.45, 0.0),
            vec3(-5.5, 0.0, 0.0),
            vec3(-5.5, 1.45, 0.0),
        ],
        vec![
            vec3(-3.0, -2.0, 0.0),
            vec3(-3.0, -0.68, 0.0),
            vec3(-3.0, 0.68, 0.0),
            vec3(-3.0, 2.0, 0.0),
        ],
        vec![
            vec3(-0.45, -2.1, 0.0),
            vec3(-0.45, -1.05, 0.0),
            vec3(-0.45, 0.0, 0.0),
            vec3(-0.45, 1.05, 0.0),
            vec3(-0.45, 2.1, 0.0),
        ],
        vec![
            vec3(2.15, -2.0, 0.0),
            vec3(2.15, -0.68, 0.0),
            vec3(2.15, 0.68, 0.0),
            vec3(2.15, 2.0, 0.0),
        ],
        vec![
            vec3(5.05, -1.35, 0.0),
            vec3(5.05, 0.0, 0.0),
            vec3(5.05, 1.35, 0.0),
        ],
    ];
    let colors = [BLUE_D, TEAL_C, PURPLE_B, ORANGE_C, GREEN_C];
    let headings = ["TOKENS", "EMBED", "ATTENTION", "MLP", "NEXT TOKEN"];

    for (column, color) in columns.iter().zip(colors) {
        add_layer_panel(&mut scene, column[0].x, color);
    }
    for index in 0..columns.len() - 1 {
        connect_layers(
            &mut scene,
            &columns[index],
            &columns[index + 1],
            colors[index + 1],
        );
    }
    for ((column, color), heading) in columns.iter().zip(colors).zip(headings) {
        scene.add_tattva(
            Label::new(heading, 0.17).with_color(alpha(color, 0.95)),
            vec3(column[0].x, 2.95, 0.1),
        );
        for position in column {
            add_network_node(&mut scene, *position, color);
        }
    }

    for (token, y) in [("scene", 1.45), ("views", 0.0), ("work", -1.45)] {
        scene.add_tattva(
            Label::new(token, 0.13).with_color(WHITE),
            vec3(-5.5, y, 0.12),
        );
    }
    for (label, probability, y, color) in [
        ("beautiful", "0.62", 1.35, GREEN_C),
        ("together", "0.24", 0.0, BLUE_B),
        ("smoothly", "0.09", -1.35, GOLD_C),
    ] {
        scene.add_tattva(
            Label::new(label, 0.13).with_color(WHITE),
            vec3(5.05, y + 0.12, 0.12),
        );
        scene.add_tattva(
            Label::new(probability, 0.11).with_color(color),
            vec3(5.05, y - 0.13, 0.12),
        );
    }

    scene.add_tattva(
        RoundedRectangle::new(4.3, 0.46, 0.22, alpha(BLUE_D, 0.12))
            .with_stroke(0.025, alpha(BLUE_B, 0.5)),
        vec3(0.0, -3.45, 0.0),
    );
    scene.add_tattva(
        Label::new("LIVE FORWARD PASS  /  12.4 ms", 0.15).with_color(BLUE_B),
        vec3(0.0, -3.45, 0.1),
    );

    let routes = [
        [
            columns[0][0],
            columns[1][2],
            columns[2][3],
            columns[3][2],
            columns[4][0],
        ],
        [
            columns[0][1],
            columns[1][1],
            columns[2][2],
            columns[3][1],
            columns[4][1],
        ],
        [
            columns[0][2],
            columns[1][3],
            columns[2][1],
            columns[3][0],
            columns[4][2],
        ],
    ];
    let pulse_colors = [PINK_C, GOLD_C, BLUE_B];
    let mut timeline = Timeline::new();
    for ((route, color), start_time) in routes.iter().zip(pulse_colors).zip([0.0, 0.55, 1.1]) {
        let pulse = scene.add_tattva(
            Circle::new(0.12, 28, color).with_stroke(0.035, WHITE),
            route[0] + vec3(0.0, 0.0, 0.25),
        );
        animate_activation(&mut timeline, pulse, route, start_time);
    }
    timeline.wait_until(3.5);
    scene.play(timeline)?;
    Ok(scene)
}

fn add_info_card(
    scene: &mut Scene,
    position: Vec3,
    number: &str,
    title: &str,
    detail: &str,
    color: Vec4,
) -> Vec<TattvaId> {
    let background = scene.add_tattva(
        RoundedRectangle::new(2.75, 1.72, 0.18, alpha(color, 0.09))
            .with_stroke(0.035, alpha(color, 0.65)),
        position,
    );
    let number = scene.add_tattva(
        Label::new(number, 0.18).with_color(color),
        position + vec3(-1.03, 0.52, 0.08),
    );
    let title = scene.add_tattva(
        Label::new(title, 0.22).with_color(WHITE),
        position + vec3(0.0, 0.18, 0.08),
    );
    let detail = scene.add_tattva(
        Label::new(detail, 0.135).with_color(GRAY_B),
        position + vec3(0.0, -0.38, 0.08),
    );
    vec![background, number, title, detail]
}

fn schedule_card(timeline: &mut Timeline, ids: &[TattvaId], start_time: f32) {
    timeline
        .animate(ids[0])
        .at(start_time)
        .for_duration(0.45)
        .ease(Ease::OutCubic)
        .appear()
        .spawn();
    for (offset, id) in ids.iter().skip(1).enumerate() {
        timeline
            .animate(*id)
            .at(start_time + 0.12 + offset as f32 * 0.08)
            .for_duration(0.5)
            .ease(Ease::Linear)
            .typewrite_text()
            .spawn();
    }
}

fn main() -> anyhow::Result<()> {
    let child = hand_built_transformer()?;
    let mut scene = Scene::new();
    let view = scene.add_scene_view(
        SceneView::new(child)
            .size(vec2(14.7, 8.15))
            .background(Vec4::new(0.018, 0.027, 0.045, 1.0))
            .corner_radius(0.32)
            .border(0.055, alpha(BLUE_B, 0.85))
            .playback(SceneViewPlayback::Loop { duration: 3.5 }),
        Vec3::ZERO,
    );

    let heading = scene.add_tattva(
        Label::new("Inside one forward pass", 0.48).with_color(WHITE),
        vec3(-3.55, 3.25, 0.0),
    );
    let subtitle = scene.add_tattva(
        Label::new(
            "The live SceneView keeps running while the parent explains it.",
            0.18,
        )
        .with_color(GRAY_A),
        vec3(-3.55, 2.7, 0.0),
    );
    let accent = scene.add_tattva(
        Line::new(vec3(-6.7, 2.42, 0.0), vec3(-0.4, 2.42, 0.0), 0.035, BLUE_D),
        Vec3::ZERO,
    );

    let cards = [
        add_info_card(
            &mut scene,
            vec3(-5.05, 1.25, 0.0),
            "01",
            "Tokenize",
            "Words become stable\nvector identities.",
            BLUE_D,
        ),
        add_info_card(
            &mut scene,
            vec3(-1.95, 1.25, 0.0),
            "02",
            "Attend",
            "Context decides which\nsignals matter now.",
            PURPLE_B,
        ),
        add_info_card(
            &mut scene,
            vec3(-5.05, -0.85, 0.0),
            "03",
            "Transform",
            "Features mix through\nlearned nonlinear paths.",
            ORANGE_C,
        ),
        add_info_card(
            &mut scene,
            vec3(-1.95, -0.85, 0.0),
            "04",
            "Decode",
            "The final state becomes\na probability distribution.",
            GREEN_C,
        ),
    ];

    let output_panel = scene.add_tattva(
        RoundedRectangle::new(4.35, 2.45, 0.22, alpha(GREEN_C, 0.075))
            .with_stroke(0.035, alpha(GREEN_C, 0.55)),
        vec3(3.65, -1.45, 0.0),
    );
    let output_title = scene.add_tattva(
        Label::new("NEXT TOKEN", 0.17).with_color(GREEN_C),
        vec3(3.65, -0.65, 0.08),
    );
    let output_word = scene.add_tattva(
        Label::new("beautiful", 0.44).with_color(WHITE),
        vec3(3.15, -1.35, 0.08),
    );
    let output_probability = scene.add_tattva(
        Label::new("62%", 0.32).with_color(GOLD_C),
        vec3(5.05, -1.35, 0.08),
    );
    let output_note = scene.add_tattva(
        Label::new("P(next | scene, views, work)", 0.15).with_color(GRAY_C),
        vec3(3.65, -2.15, 0.08),
    );

    let mut main_content = vec![heading, subtitle, accent];
    for card in &cards {
        main_content.extend(card.iter().copied());
    }
    main_content.extend([
        output_panel,
        output_title,
        output_word,
        output_probability,
        output_note,
    ]);

    let mut timeline = Timeline::new();
    timeline
        .animate(view)
        .at(1.55)
        .for_duration(0.95)
        .ease(Ease::InOutCubic)
        .move_to(vec3(5.05, 2.65, 0.0))
        .spawn();
    timeline
        .animate(view)
        .at(1.55)
        .for_duration(0.95)
        .ease(Ease::InOutCubic)
        .scale_to(Vec3::new(0.35, 0.35, 1.0))
        .spawn();
    timeline
        .animate(view)
        .at(1.55)
        .for_duration(0.95)
        .ease(Ease::InOutCubic)
        .rotate_to(Quat::from_rotation_z(-0.035))
        .spawn();

    timeline
        .animate(heading)
        .at(2.05)
        .for_duration(0.8)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(subtitle)
        .at(2.3)
        .for_duration(0.65)
        .ease(Ease::OutCubic)
        .appear()
        .spawn();
    timeline
        .animate(accent)
        .at(2.35)
        .for_duration(0.7)
        .ease(Ease::OutCubic)
        .draw()
        .spawn();
    for (index, card) in cards.iter().enumerate() {
        schedule_card(&mut timeline, card, 2.75 + index as f32 * 0.42);
    }

    timeline
        .animate(output_panel)
        .at(4.5)
        .for_duration(0.5)
        .ease(Ease::OutCubic)
        .appear()
        .spawn();
    for (index, id) in [output_title, output_word, output_probability, output_note]
        .iter()
        .enumerate()
    {
        timeline
            .animate(*id)
            .at(4.62 + index as f32 * 0.16)
            .for_duration(0.55)
            .ease(Ease::Linear)
            .typewrite_text()
            .spawn();
    }

    for id in main_content {
        timeline
            .animate(id)
            .at(6.55)
            .for_duration(0.45)
            .ease(Ease::InOutCubic)
            .fade_to(0.0)
            .spawn();
    }
    timeline
        .animate(view)
        .at(6.85)
        .for_duration(1.0)
        .ease(Ease::InOutCubic)
        .move_to(Vec3::ZERO)
        .spawn();
    timeline
        .animate(view)
        .at(6.85)
        .for_duration(1.0)
        .ease(Ease::InOutCubic)
        .scale_to(Vec3::ONE)
        .spawn();
    timeline
        .animate(view)
        .at(6.85)
        .for_duration(1.0)
        .ease(Ease::InOutCubic)
        .rotate_to(Quat::IDENTITY)
        .spawn();
    scene.play(timeline)?;

    let settings = ExportSettings {
        width: 960,
        fps: 30,
        duration_seconds: 8.0,
        video_enabled: false,
        preserve_frame_exports: false,
        png_compression: PngCompressionMode::Fast,
        ..ExportSettings::from_scene(&scene)
    };
    App::new()?
        .with_scene(scene)
        .with_export_settings(settings)
        .run_app()
}
