use glam::{Vec3, vec2};
use murali::App;
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::animation::Ease;
use murali::frontend::collection::primitives::path::Path;
use murali::frontend::collection::text::label::Label;
use murali::frontend::layout::Direction;
use murali::positions::CAMERA_DEFAULT_POS;

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    let title_id = scene.add_tattva(
        Label::new("Text Animation", 0.38).with_color(WHITE),
        Vec3::ZERO,
    );
    scene.to_edge(title_id, Direction::Up, 0.8);

    let subtitle_id = scene.add_tattva(
        Label::new(
            "Typewriter, centered reveal, indicate, and one simple authored path write.",
            0.18,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, 2.95, 0.0),
    );

    let type_heading_id = scene.add_tattva(
        Label::new("Typewrite", 0.18).with_color(GRAY_B),
        Vec3::new(-4.9, 1.85, 0.0),
    );
    let mut type_label =
        Label::new("A narrated line grows from left to right.", 0.3).with_color(TEAL_C);
    type_label.typewriter_mode = true;
    let type_label_id = scene.add_tattva(type_label, Vec3::new(-4.1, 0.7, 0.0));
    let type_caption_id = scene.add_tattva(
        Label::new("Stable anchor, good for spoken narration.", 0.16).with_color(GRAY_A),
        Vec3::new(-4.1, -0.15, 0.0),
    );

    let reveal_heading_id = scene.add_tattva(
        Label::new("Reveal", 0.18).with_color(GRAY_B),
        Vec3::new(0.0, 1.85, 0.0),
    );
    let reveal_label_id = scene.add_tattva(
        Label::new("A centered line blooms into view.", 0.3).with_color(GOLD_C),
        Vec3::new(0.0, 0.7, 0.0),
    );
    let reveal_caption_id = scene.add_tattva(
        Label::new("Symmetric motion, good for emphasis.", 0.16).with_color(GRAY_A),
        Vec3::new(0.0, -0.15, 0.0),
    );

    let indicate_heading_id = scene.add_tattva(
        Label::new("Indicate", 0.18).with_color(GRAY_B),
        Vec3::new(4.9, 1.85, 0.0),
    );
    let indicate_label_id = scene.add_tattva(
        Label::new("This phrase matters right now.", 0.3).with_color(BLUE_B),
        Vec3::new(4.9, 0.7, 0.0),
    );
    let indicate_caption_id = scene.add_tattva(
        Label::new("A pulse for attention without relayout.", 0.16).with_color(GRAY_A),
        Vec3::new(4.9, -0.15, 0.0),
    );

    let path_heading_id = scene.add_tattva(
        Label::new("Draw And Undraw", 0.18).with_color(GRAY_B),
        Vec3::new(0.0, -1.2, 0.0),
    );
    let path_id = scene.add_tattva(
        Path::new()
            .move_to(vec2(-1.9, -2.15))
            .quad_to(vec2(-1.1, -1.25), vec2(-0.2, -2.0))
            .cubic_to(vec2(0.4, -2.45), vec2(1.0, -1.05), vec2(1.8, -1.9))
            .with_thickness(0.08)
            .with_color(PURPLE_B),
        Vec3::ZERO,
    );
    let path_caption_id = scene.add_tattva(
        Label::new(
            "Useful when text and line-work need the same authored timing.",
            0.16,
        )
        .with_color(GRAY_A),
        Vec3::new(0.0, -2.9, 0.0),
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
        .for_duration(1.5)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    timeline
        .animate(type_heading_id)
        .at(1.5)
        .for_duration(0.8)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(type_label_id)
        .at(1.9)
        .for_duration(2.0)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(type_caption_id)
        .at(2.2)
        .for_duration(1.1)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(type_label_id)
        .at(4.8)
        .for_duration(0.9)
        .ease(Ease::Linear)
        .untypewrite_text()
        .spawn();
    timeline
        .animate(type_caption_id)
        .at(4.95)
        .for_duration(0.8)
        .ease(Ease::Linear)
        .untypewrite_text()
        .spawn();
    timeline
        .animate(type_heading_id)
        .at(5.1)
        .for_duration(0.7)
        .ease(Ease::Linear)
        .untypewrite_text()
        .spawn();

    timeline
        .animate(reveal_heading_id)
        .at(5.9)
        .for_duration(0.8)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(reveal_label_id)
        .at(6.3)
        .for_duration(1.8)
        .ease(Ease::Linear)
        .reveal_text()
        .spawn();
    timeline
        .animate(reveal_caption_id)
        .at(6.6)
        .for_duration(1.0)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(reveal_label_id)
        .at(9.0)
        .for_duration(1.0)
        .ease(Ease::Linear)
        .hide_text()
        .spawn();
    timeline
        .animate(reveal_caption_id)
        .at(9.15)
        .for_duration(0.8)
        .ease(Ease::Linear)
        .untypewrite_text()
        .spawn();
    timeline
        .animate(reveal_heading_id)
        .at(9.3)
        .for_duration(0.7)
        .ease(Ease::Linear)
        .untypewrite_text()
        .spawn();

    timeline
        .animate(indicate_heading_id)
        .at(10.2)
        .for_duration(0.8)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(indicate_label_id)
        .at(10.6)
        .for_duration(1.3)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(indicate_caption_id)
        .at(10.85)
        .for_duration(1.0)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    for pulse_at in [11.9, 12.7, 13.4] {
        timeline
            .animate(indicate_label_id)
            .at(pulse_at)
            .for_duration(0.75)
            .ease(Ease::InOutCubic)
            .indicate()
            .spawn();
    }
    timeline
        .animate(indicate_label_id)
        .at(14.5)
        .for_duration(0.9)
        .ease(Ease::Linear)
        .untypewrite_text()
        .spawn();
    timeline
        .animate(indicate_caption_id)
        .at(14.65)
        .for_duration(0.8)
        .ease(Ease::Linear)
        .untypewrite_text()
        .spawn();
    timeline
        .animate(indicate_heading_id)
        .at(14.8)
        .for_duration(0.7)
        .ease(Ease::Linear)
        .untypewrite_text()
        .spawn();

    timeline
        .animate(path_heading_id)
        .at(15.5)
        .for_duration(0.9)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(path_id)
        .at(16.0)
        .for_duration(1.8)
        .ease(Ease::OutCubic)
        .draw()
        .spawn();
    timeline
        .animate(path_caption_id)
        .at(16.35)
        .for_duration(1.2)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(path_id)
        .at(19.0)
        .for_duration(1.5)
        .ease(Ease::InCubic)
        .undraw()
        .spawn();
    timeline
        .animate(path_caption_id)
        .at(19.15)
        .for_duration(0.9)
        .ease(Ease::Linear)
        .untypewrite_text()
        .spawn();
    timeline
        .animate(path_heading_id)
        .at(19.3)
        .for_duration(0.8)
        .ease(Ease::Linear)
        .untypewrite_text()
        .spawn();

    scene.play(timeline)?;
    scene.camera_mut().position = CAMERA_DEFAULT_POS;

    App::new()?.with_scene(scene).run_app()
}
