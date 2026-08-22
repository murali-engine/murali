use glam::{Vec3, Vec4, vec2, vec3};
use murali::colors::{BLUE_D, GOLD_C, GRAY_A, GREEN_C, PINK_C, TEAL_C, WHITE};
use murali::engine::camera::Projection;
use murali::engine::export::{ExportSettings, PngCompressionMode};
use murali::frontend::TattvaId;
use murali::frontend::animation::Ease;
use murali::frontend::props::layers;
use murali::frontend::sangrah::composite::beta::opening::{Opening, OpeningStyle};
use murali::frontend::sangrah::primitives::{
    circle::Circle, line::Line, rounded_rectangle::RoundedRectangle,
};
use murali::frontend::sangrah::text::label::Label;
use murali::{
    App, BuiltinTexture, Clip, Scene, SceneView, SceneViewPlayback, TextureImage, Timeline,
};

const BACKGROUND: Vec4 = Vec4::new(0.039, 0.071, 0.11, 1.0);

fn alpha(color: Vec4, opacity: f32) -> Vec4 {
    Vec4::new(color.x, color.y, color.z, opacity)
}

fn build_opening_scene() -> anyhow::Result<(Scene, f32)> {
    let mut scene = Scene::new();
    scene.camera_mut().projection = Projection::Perspective {
        fov_y_rad: 43.0_f32.to_radians(),
        aspect: 16.0 / 9.0,
        near: 0.1,
        far: 80.0,
    };
    scene.camera_mut().position = vec3(0.0, 2.15, 10.8);
    scene.camera_mut().target = vec3(0.0, -0.35, 0.0);

    let opening = Opening::new("MURALI", "PROGRAMMATIC VISUALS")
        .with_texture(TextureImage::builtin(BuiltinTexture::BlackMarble))
        .with_style(OpeningStyle {
            letter_height: 2.15,
            letter_depth: 0.78,
            letter_gap: 0.3,
            particle_count: 360,
            particle_size: 0.025,
            tagline_height: 0.4,
            tagline_color: alpha(WHITE, 0.94),
            ..OpeningStyle::default()
        });
    let ids = opening.add_to_scene(&mut scene, Vec3::ZERO)?;
    let mut opening_clip = Clip::new();
    ids.animate(&mut opening_clip);
    let duration = opening_clip.duration();

    let mut timeline = Timeline::new();
    timeline.append(opening_clip);
    scene.play(timeline)?;
    Ok((scene, duration))
}

#[derive(Clone)]
struct StageIds {
    panel: TattvaId,
    title: TattvaId,
    detail: TattvaId,
    nodes: Vec<TattvaId>,
}

impl StageIds {
    fn all(&self) -> Vec<TattvaId> {
        let mut ids = vec![self.panel, self.title, self.detail];
        ids.extend(self.nodes.iter().copied());
        ids
    }
}

fn add_stage(scene: &mut Scene, x: f32, title: &str, detail: &str, color: Vec4) -> StageIds {
    let panel = scene.add_tattva(
        RoundedRectangle::new(3.25, 3.6, 0.22, alpha(color, 0.08))
            .with_stroke(0.035, alpha(color, 0.72)),
        vec3(x, -0.35, 0.0),
    );
    let title = scene.add_tattva(
        Label::new(title, 0.25).with_color(color),
        vec3(x, 0.82, 0.08),
    );
    let detail = scene.add_tattva(
        Label::new(detail, 0.16).with_color(alpha(GRAY_A, 0.82)),
        vec3(x, -1.45, 0.08),
    );
    let mut nodes = Vec::new();
    for (index, y) in [0.35_f32, -0.25, -0.85].into_iter().enumerate() {
        let radius = 0.18 + index as f32 * 0.025;
        nodes.push(scene.add_tattva(
            Circle::new(radius + 0.12, 36, alpha(color, 0.11)),
            vec3(x, y, 0.06),
        ));
        nodes.push(scene.add_tattva(
            Circle::new(radius, 36, alpha(color, 0.92)).with_stroke(0.025, alpha(WHITE, 0.85)),
            vec3(x, y, 0.1),
        ));
    }
    StageIds {
        panel,
        title,
        detail,
        nodes,
    }
}

fn schedule_stage(timeline: &mut Timeline, stage: &StageIds, start: f32) {
    timeline
        .animate(stage.panel)
        .at(start)
        .for_duration(0.5)
        .ease(Ease::OutCubic)
        .appear()
        .spawn();
    timeline
        .animate(stage.title)
        .at(start + 0.12)
        .for_duration(0.46)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(stage.detail)
        .at(start + 0.3)
        .for_duration(0.5)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    for (index, node) in stage.nodes.iter().copied().enumerate() {
        timeline
            .animate(node)
            .at(start + 0.24 + index as f32 * 0.055)
            .for_duration(0.34)
            .ease(Ease::OutCubic)
            .appear()
            .spawn();
    }
}

fn build_parent_scene(opening_scene: Scene, opening_duration: f32) -> anyhow::Result<Scene> {
    let mut scene = Scene::new();

    let heading = scene.add_tattva(
        Label::new("A live inference path", 0.48).with_color(WHITE),
        vec3(0.0, 3.35, 0.0),
    );
    let subtitle = scene.add_tattva(
        Label::new("Signals continue after the ident", 0.19).with_color(alpha(GRAY_A, 0.78)),
        vec3(0.0, 2.82, 0.0),
    );
    let stages = [
        add_stage(&mut scene, -4.55, "OBSERVE", "structured input", TEAL_C),
        add_stage(&mut scene, 0.0, "REASON", "latent computation", PINK_C),
        add_stage(&mut scene, 4.55, "EXPLAIN", "grounded output", GREEN_C),
    ];

    let connectors = [
        scene.add_tattva(
            Line::new(
                vec3(-2.92, -0.35, 0.02),
                vec3(-1.63, -0.35, 0.02),
                0.04,
                alpha(BLUE_D, 0.62),
            ),
            Vec3::ZERO,
        ),
        scene.add_tattva(
            Line::new(
                vec3(1.63, -0.35, 0.02),
                vec3(2.92, -0.35, 0.02),
                0.04,
                alpha(GOLD_C, 0.68),
            ),
            Vec3::ZERO,
        ),
    ];

    let mut pulses = Vec::new();
    for (index, color) in [TEAL_C, GOLD_C, GREEN_C].into_iter().enumerate() {
        let pulse = scene.add_tattva(
            Circle::new(0.12, 28, color).with_stroke(0.025, WHITE),
            vec3(-6.65, -0.35 + (index as f32 - 1.0) * 0.16, 0.2),
        );
        scene.set_layer(pulse, layers::OVERLAY);
        pulses.push(pulse);
    }

    let mut main_ids = vec![heading, subtitle];
    for stage in &stages {
        main_ids.extend(stage.all());
    }
    main_ids.extend(connectors);
    main_ids.extend(pulses.iter().copied());
    for id in main_ids {
        scene.hide(id);
    }

    let (frame_width, frame_height) = scene.frame().logical_size();
    let opening_view = scene.add_scene_view(
        SceneView::new(opening_scene)
            .size(vec2(frame_width, frame_height))
            .background(BACKGROUND)
            .playback(SceneViewPlayback::Once)
            .resolution(1280, 720),
        Vec3::ZERO,
    );

    let reveal_start = opening_duration - 0.15;
    let mut timeline = Timeline::new();
    timeline
        .animate(opening_view)
        .at(opening_duration - 0.3)
        .for_duration(0.72)
        .ease(Ease::InOutCubic)
        .fade_to(0.0)
        .spawn();
    timeline
        .animate(heading)
        .at(reveal_start)
        .for_duration(0.65)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(subtitle)
        .at(reveal_start + 0.28)
        .for_duration(0.58)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    for (index, stage) in stages.iter().enumerate() {
        schedule_stage(
            &mut timeline,
            stage,
            reveal_start + 0.55 + index as f32 * 0.24,
        );
    }
    for (index, connector) in connectors.into_iter().enumerate() {
        timeline
            .animate(connector)
            .at(reveal_start + 1.02 + index as f32 * 0.2)
            .for_duration(0.52)
            .ease(Ease::OutCubic)
            .draw()
            .spawn();
    }
    for (index, pulse) in pulses.into_iter().enumerate() {
        let start = reveal_start + 1.35 + index as f32 * 0.42;
        timeline
            .animate(pulse)
            .at(start)
            .for_duration(0.16)
            .ease(Ease::OutCubic)
            .appear()
            .spawn();
        timeline
            .animate(pulse)
            .at(start)
            .for_duration(2.45)
            .ease(Ease::InOutCubic)
            .move_to(vec3(6.65, -0.35 + (index as f32 - 1.0) * 0.16, 0.2))
            .spawn();
        timeline
            .animate(pulse)
            .at(start + 2.2)
            .for_duration(0.25)
            .ease(Ease::InCubic)
            .fade_to(0.0)
            .spawn();
    }
    timeline.wait_until(opening_duration + 5.0);
    scene.play(timeline)?;
    Ok(scene)
}

fn main() -> anyhow::Result<()> {
    let (opening_scene, opening_duration) = build_opening_scene()?;
    let scene = build_parent_scene(opening_scene, opening_duration)?;
    let settings = ExportSettings {
        width: 1280,
        fps: 30,
        duration_seconds: opening_duration + 5.0,
        video_enabled: true,
        preserve_frame_exports: false,
        png_compression: PngCompressionMode::Fast,
        clear_color: BACKGROUND,
        ..ExportSettings::from_scene(&scene)
    };

    App::new()?
        .with_scene(scene)
        .with_export_settings(settings)
        .run_app()
}
