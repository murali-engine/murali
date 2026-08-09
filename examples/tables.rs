use glam::Vec3;
use murali::App;
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::animation::Ease;
use murali::frontend::collection::table::Table;
use murali::frontend::collection::text::label::Label;
use murali::frontend::layout::Direction;
use murali::positions::CAMERA_DEFAULT_POS;

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    let title_id = scene.add_tattva(Label::new("Tables", 0.38).with_color(WHITE), Vec3::ZERO);
    scene.to_edge(title_id, Direction::Up, 0.8);

    let subtitle_id = scene.add_tattva(
        Label::new(
            "One table, written in cleanly, held long enough to read, then unwritten.",
            0.18,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, 2.95, 0.0),
    );

    let table_id = scene.add_tattva(
        Table::new(vec![
            vec!["Alice", "28", "NYC"],
            vec!["Bob", "34", "LA"],
            vec!["Charlie", "25", "Chicago"],
        ])
        .with_col_labels(vec!["Name", "Age", "City"])
        .with_row_labels(vec!["Person 1", "Person 2", "Person 3"])
        .with_title("Person Data")
        .with_line_color(TEAL_C)
        .with_text_color(WHITE)
        .with_text_height(0.25)
        .with_h_buff(0.3)
        .with_v_buff(0.2)
        .with_outer_lines(true)
        .with_write_progress(0.0),
        Vec3::new(0.0, -0.05, 0.0),
    );

    let footer_id = scene.add_tattva(
        Label::new(
            "Start with one readable table before exploring multiple table styles or transitions.",
            0.17,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, -3.05, 0.0),
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
        .animate(table_id)
        .at(1.8)
        .for_duration(3.2)
        .ease(Ease::InOutQuad)
        .write_table()
        .spawn();
    timeline
        .animate(footer_id)
        .at(2.4)
        .for_duration(1.4)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(table_id)
        .at(6.8)
        .for_duration(2.2)
        .ease(Ease::InOutQuad)
        .unwrite_table()
        .spawn();

    scene.play(timeline)?;
    scene.camera_mut().position = CAMERA_DEFAULT_POS;

    App::new()?.with_scene(scene).run_app()
}
