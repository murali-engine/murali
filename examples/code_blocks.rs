use glam::Vec3;
use murali::App;
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::animation::Ease;
use murali::frontend::collection::text::code_block::{CodeBlock, CodeBlockSurface, CodeBlockTheme};
use murali::frontend::collection::text::label::Label;
use murali::frontend::layout::Direction;
use murali::positions::CAMERA_DEFAULT_POS;

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();
    let rust_pos = Vec3::new(3.0, -0.15, 0.0);
    let toml_pos = Vec3::new(2.9, 0.15, 0.0);

    let title_id = scene.add_tattva(
        Label::new("Re-imagined Code Blocks", 0.38).with_color(WHITE),
        Vec3::ZERO,
    );
    scene.to_edge(title_id, Direction::Up, 0.8);

    let subtitle_id = scene.add_tattva(
        Label::new(
            "Murali now supports window decorations, line numbers, and custom themes for professional presentations.",
            0.18,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, 2.9, 0.0),
    );

    let heading_id = scene.add_tattva(
        Label::new("highlight.rs", 0.24).with_color(GRAY_B),
        Vec3::new(0.0, 2.1, 0.0),
    );

    let rust_code = r#"fn highlight(tokens: &[&str]) -> Vec<String> {
    tokens
        .iter()
        .map(|token| token.to_uppercase())
        .collect()
}"#;
    let rust_id = scene.add_tattva(
        CodeBlock::new(rust_code, "rust", 0.28)
            .with_theme(CodeBlockTheme::Dark)
            .with_surface(CodeBlockSurface::Dark)
            .with_title("highlight.rs")
            .with_line_numbers(false)
            .with_content_box_size(7.20, 2.4)
            .with_content_offset(0.0, -0.06),
        rust_pos,
    );
    scene.hide(rust_id);

    let toml_code = r#"[render]
fps = 60
background = "slate"
"#;
    let toml_id = scene.add_tattva(
        CodeBlock::new(toml_code, "toml", 0.30)
            .with_theme(CodeBlockTheme::Light)
            .with_surface(CodeBlockSurface::Light)
            .with_title("murali.toml")
            .with_line_numbers(true)
            .with_content_box_size(4.2, 1.35)
            .with_content_offset(0.0, -0.05),
        toml_pos,
    );
    scene.hide(toml_id);

    let footer_id = scene.add_tattva(
        Label::new(
            "Typst-powered line numbering and triangulated window dots ensure maximum clarity at any scale.",
            0.15,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, -2.5, 0.0),
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
        .for_duration(1.7)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    timeline
        .animate(heading_id)
        .at(1.6)
        .for_duration(0.9)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    // SHOW THE RUST CODE
    timeline
        .animate(rust_id)
        .at(2.0)
        .for_duration(0.35)
        .ease(Ease::Linear)
        .appear()
        .spawn();

    timeline
        .animate(rust_id)
        .at(2.0)
        .for_duration(1.2)
        .ease(Ease::InOutCubic)
        .move_to(rust_pos)
        .from_vec3(rust_pos + Vec3::new(0.0, -0.18, 0.0))
        .spawn();

    // Switch to TOML after a pause
    timeline
        .animate(rust_id)
        .at(5.3)
        .for_duration(0.6)
        .ease(Ease::InOutCubic)
        .fade_to(0.0)
        .spawn();

    timeline.call_at(5.8, move |scene| {
        if let Some(label) = scene.get_tattva_typed_mut::<Label>(heading_id) {
            label.state.text = "murali.toml".to_string();
            label.mark_dirty(
                murali::frontend::DirtyFlags::GEOMETRY | murali::frontend::DirtyFlags::STYLE,
            );
        }
    });

    // SHOW THE TOML CODE
    timeline
        .animate(toml_id)
        .at(6.2)
        .for_duration(0.35)
        .ease(Ease::Linear)
        .appear()
        .spawn();

    timeline
        .animate(toml_id)
        .at(6.2)
        .for_duration(1.2)
        .ease(Ease::InOutCubic)
        .move_to(toml_pos)
        .from_vec3(toml_pos + Vec3::new(0.0, -0.18, 0.0))
        .spawn();

    timeline
        .animate(footer_id)
        .at(8.5)
        .for_duration(1.8)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    scene.play(timeline)?;
    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    // scene.camera_mut().set_view_width(8.0);

    App::new()?.with_scene(scene).run_app()
}
