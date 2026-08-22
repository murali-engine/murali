use crate::frontend::layout::{Bounded, Bounds};
use crate::frontend::sangrah::text::label::Label;
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};
use glam::{Vec2, Vec3, Vec4};

/// Represents a single cell in a table
#[derive(Clone, Debug)]
pub struct TableCell {
    pub content: String,
    pub width: f32,
    pub height: f32,
}

impl TableCell {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            width: 1.0,
            height: 0.5,
        }
    }

    pub fn with_size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }
}

/// Configuration for table styling
#[derive(Clone, Debug)]
pub struct TableConfig {
    pub h_buff: f32,
    pub v_buff: f32,
    pub line_color: Vec4,
    pub line_thickness: f32,
    pub text_color: Vec4,
    pub text_height: f32,
    pub include_outer_lines: bool,
    pub background_color: Option<Vec4>,
    pub labels_inside: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableTitlePosition {
    Top,
    Bottom,
}

impl Default for TableConfig {
    fn default() -> Self {
        Self {
            h_buff: 0.2,
            v_buff: 0.2,
            line_color: Vec4::new(0.8, 0.8, 0.8, 1.0),
            line_thickness: 0.02,
            text_color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            text_height: 0.3,
            include_outer_lines: true,
            background_color: None,
            labels_inside: false,
        }
    }
}

#[derive(Clone, Debug)]
struct TableState {
    rows: Vec<Vec<TableCell>>,
    row_labels: Option<Vec<String>>,
    col_labels: Option<Vec<String>>,
    title: Option<String>,
    config: TableConfig,
    write_progress: f32,
    title_color: Option<Vec4>,
    title_height: Option<f32>,
    title_position: TableTitlePosition,
}

impl TableState {
    fn new(data: Vec<Vec<impl Into<String>>>) -> Self {
        let rows = data
            .into_iter()
            .map(|row| row.into_iter().map(TableCell::new).collect())
            .collect();

        Self {
            rows,
            row_labels: None,
            col_labels: None,
            title: None,
            config: TableConfig::default(),
            write_progress: 1.0,
            title_color: None,
            title_height: None,
            title_position: TableTitlePosition::Bottom,
        }
    }

    fn num_rows(&self) -> usize {
        self.rows.len()
    }

    fn num_cols(&self) -> usize {
        self.rows.first().map(|r| r.len()).unwrap_or(0)
    }

    fn get_cell(&self, row: usize, col: usize) -> Option<&TableCell> {
        if row > 0 && col > 0 {
            self.rows.get(row - 1)?.get(col - 1)
        } else {
            None
        }
    }

    fn calculate_dimensions(&self) -> (f32, f32) {
        let num_rows = self.num_rows();
        let num_cols = self.num_cols();
        let cell_width = 1.2;
        let cell_height = 0.6;

        let mut total_width =
            num_cols as f32 * cell_width + (num_cols as f32 - 1.0).max(0.0) * self.config.h_buff;
        if self.row_labels.is_some() {
            total_width += 1.5 + self.config.h_buff;
        }

        let mut total_height =
            num_rows as f32 * cell_height + (num_rows as f32 - 1.0).max(0.0) * self.config.v_buff;
        if self.col_labels.is_some() {
            total_height += cell_height + self.config.v_buff;
        }

        if self.title.is_some() {
            total_height += self.title_gap() + self.title_height();
        }

        (total_width, total_height)
    }

    fn title_height(&self) -> f32 {
        self.title_height.unwrap_or(self.config.text_height * 0.9)
    }

    fn title_color(&self) -> Vec4 {
        self.title_color
            .unwrap_or(Vec4::new(0.79, 0.83, 0.88, self.config.text_color.w))
    }

    fn title_gap(&self) -> f32 {
        self.config.text_height * 1.2
    }
}

#[derive(Clone, Debug)]
struct TextEntry {
    content: String,
    offset: Vec3,
    height: f32,
}

fn grid_metrics(state: &TableState) -> (usize, usize, f32, f32, f32, f32) {
    let cell_width = 1.2;
    let cell_height = 0.6;
    let num_rows = state.num_rows();
    let num_cols = state.num_cols();
    let (grid_cols, grid_rows) = if state.config.labels_inside {
        let cols = num_cols + usize::from(state.row_labels.is_some());
        let rows = num_rows + usize::from(state.col_labels.is_some());
        (cols, rows)
    } else {
        (num_cols, num_rows)
    };
    let grid_width = grid_cols as f32 * (cell_width + state.config.h_buff);
    let grid_height = grid_rows as f32 * (cell_height + state.config.v_buff);
    let start_x = -grid_width / 2.0;
    let start_y = grid_height / 2.0;
    (
        grid_cols,
        grid_rows,
        cell_width,
        cell_height,
        start_x,
        start_y,
    )
}

fn visible_line_indices(count: usize, include_outer_lines: bool) -> Vec<usize> {
    if include_outer_lines {
        (0..=count).collect()
    } else if count > 0 {
        (1..count).collect()
    } else {
        Vec::new()
    }
}

fn draw_progress(progress: f32, index: usize, total: usize) -> f32 {
    if total == 0 {
        return 1.0;
    }
    let start = index as f32 / total as f32;
    let end = (index + 1) as f32 / total as f32;
    ((progress - start) / (end - start)).clamp(0.0, 1.0)
}

fn staggered_text_progress(progress: f32, index: usize, total: usize, window_factor: f32) -> f32 {
    if total == 0 {
        return 1.0;
    }
    let step = 1.0 / ((total.saturating_sub(1)) as f32 + window_factor);
    let start = index as f32 * step;
    let end = start + step * window_factor;
    ((progress - start) / (end - start)).clamp(0.0, 1.0)
}

fn draw_lines(state: &TableState, ctx: &mut ProjectionCtx, line_progress: f32) {
    let (grid_cols, grid_rows, cell_width, cell_height, start_x, start_y) = grid_metrics(state);
    let grid_width = grid_cols as f32 * (cell_width + state.config.h_buff);
    let grid_height = grid_rows as f32 * (cell_height + state.config.v_buff);
    let h_lines = visible_line_indices(grid_rows, state.config.include_outer_lines);
    let v_lines = visible_line_indices(grid_cols, state.config.include_outer_lines);
    let total_lines = h_lines.len() + v_lines.len();

    for (order, row) in h_lines.iter().enumerate() {
        let draw = draw_progress(line_progress, order, total_lines);
        if draw <= 0.0 {
            continue;
        }
        let y = start_y - (*row as f32) * (cell_height + state.config.v_buff);
        ctx.emit(RenderPrimitive::Line {
            start: Vec3::new(start_x, y, 0.0),
            end: Vec3::new(start_x + grid_width * draw, y, 0.0),
            thickness: state.config.line_thickness,
            color: state.config.line_color,
            dash_length: 0.0,
            gap_length: 0.0,
            dash_offset: 0.0,
        });
    }

    for (local_order, col) in v_lines.iter().enumerate() {
        let draw = draw_progress(line_progress, h_lines.len() + local_order, total_lines);
        if draw <= 0.0 {
            continue;
        }
        let x = start_x + (*col as f32) * (cell_width + state.config.h_buff);
        ctx.emit(RenderPrimitive::Line {
            start: Vec3::new(x, start_y, 0.0),
            end: Vec3::new(x, start_y - grid_height * draw, 0.0),
            thickness: state.config.line_thickness,
            color: state.config.line_color,
            dash_length: 0.0,
            gap_length: 0.0,
            dash_offset: 0.0,
        });
    }
}

fn collect_text_entries(state: &TableState) -> Vec<TextEntry> {
    let (_, _, cell_width, cell_height, start_x, start_y) = grid_metrics(state);
    let mut entries = Vec::new();

    if state.config.labels_inside {
        let row_offset = usize::from(state.col_labels.is_some());
        let col_offset = usize::from(state.row_labels.is_some());

        if let Some(labels) = &state.col_labels {
            for (idx, label) in labels.iter().enumerate() {
                let x = start_x
                    + ((idx + col_offset) as f32) * (cell_width + state.config.h_buff)
                    + (cell_width + state.config.h_buff) / 2.0;
                let y = start_y - (cell_height + state.config.v_buff) / 2.0;
                entries.push(TextEntry {
                    content: label.clone(),
                    offset: Vec3::new(x, y, 0.0),
                    height: state.config.text_height * 0.8,
                });
            }
        }

        if let Some(labels) = &state.row_labels {
            for (idx, label) in labels.iter().enumerate() {
                let x = start_x + (cell_width + state.config.h_buff) / 2.0;
                let y = start_y
                    - ((idx + row_offset) as f32) * (cell_height + state.config.v_buff)
                    - (cell_height + state.config.v_buff) / 2.0;
                entries.push(TextEntry {
                    content: label.clone(),
                    offset: Vec3::new(x, y, 0.0),
                    height: state.config.text_height * 0.8,
                });
            }
        }

        for (row_idx, row) in state.rows.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                let x = start_x
                    + ((col_idx + col_offset) as f32) * (cell_width + state.config.h_buff)
                    + (cell_width + state.config.h_buff) / 2.0;
                let y = start_y
                    - ((row_idx + row_offset) as f32) * (cell_height + state.config.v_buff)
                    - (cell_height + state.config.v_buff) / 2.0;
                entries.push(TextEntry {
                    content: cell.content.clone(),
                    offset: Vec3::new(x, y, 0.0),
                    height: state.config.text_height * 0.8,
                });
            }
        }
    } else {
        if let Some(labels) = &state.col_labels {
            for (idx, label) in labels.iter().enumerate() {
                let x = start_x
                    + (idx as f32) * (cell_width + state.config.h_buff)
                    + (cell_width + state.config.h_buff) / 2.0;
                let y = start_y + (cell_height + state.config.v_buff) / 2.0;
                entries.push(TextEntry {
                    content: label.clone(),
                    offset: Vec3::new(x, y, 0.0),
                    height: state.config.text_height * 0.8,
                });
            }
        }

        if let Some(labels) = &state.row_labels {
            for (idx, label) in labels.iter().enumerate() {
                let x = start_x - (cell_width + state.config.h_buff) / 2.0;
                let y = start_y
                    - (idx as f32) * (cell_height + state.config.v_buff)
                    - (cell_height + state.config.v_buff) / 2.0;
                entries.push(TextEntry {
                    content: label.clone(),
                    offset: Vec3::new(x, y, 0.0),
                    height: state.config.text_height * 0.8,
                });
            }
        }

        for (row_idx, row) in state.rows.iter().enumerate() {
            for (col_idx, cell) in row.iter().enumerate() {
                let x = start_x
                    + (col_idx as f32) * (cell_width + state.config.h_buff)
                    + (cell_width + state.config.h_buff) / 2.0;
                let y = start_y
                    - (row_idx as f32) * (cell_height + state.config.v_buff)
                    - (cell_height + state.config.v_buff) / 2.0;
                entries.push(TextEntry {
                    content: cell.content.clone(),
                    offset: Vec3::new(x, y, 0.0),
                    height: state.config.text_height * 0.8,
                });
            }
        }
    }

    entries
}

fn title_entry(state: &TableState) -> Option<TextEntry> {
    let (_, height) = state.calculate_dimensions();
    let title = state.title.as_ref()?;
    let title_y = match state.title_position {
        TableTitlePosition::Top => height * 0.5 - state.title_height() * 0.5,
        TableTitlePosition::Bottom => -height * 0.5 + state.title_height() * 0.5,
    };
    Some(TextEntry {
        content: title.clone(),
        offset: Vec3::new(0.0, title_y, 0.0),
        height: state.title_height(),
    })
}

fn project_v1_text(state: &TableState, ctx: &mut ProjectionCtx, text_progress: f32) {
    if let Some(title) = title_entry(state) {
        let title_alpha = (text_progress / 0.35).clamp(0.0, 1.0);
        if title_alpha > 0.0 {
            let mut color = state.title_color();
            color.w *= title_alpha;
            ctx.emit(RenderPrimitive::Text {
                content: title.content,
                height: title.height,
                color,
                font_name: None,
                offset: title.offset,
                rotation: 0.0,
            });
        }
    }

    let entries = collect_text_entries(state);
    let total = entries.len();
    for (idx, entry) in entries.into_iter().enumerate() {
        let alpha = staggered_text_progress(text_progress, idx, total, 1.35);
        if alpha <= 0.0 {
            break;
        }
        let mut color = state.config.text_color;
        color.w *= alpha;
        ctx.emit(RenderPrimitive::Text {
            content: entry.content,
            height: entry.height,
            color,
            font_name: None,
            offset: entry.offset,
            rotation: 0.0,
        });
    }
}

fn project_written_text(state: &TableState, ctx: &mut ProjectionCtx, text_progress: f32) {
    if let Some(title) = title_entry(state) {
        let title_progress = (text_progress / 0.35).clamp(0.0, 1.0);
        if title_progress > 0.0 {
            let mut label = Label::new(title.content, title.height)
                .with_color(state.title_color())
                .with_char_reveal(title_progress);
            label.typewriter_mode = true;
            ctx.with_offset(title.offset, |ctx| label.project(ctx));
        }
    }

    let entries = collect_text_entries(state);
    let total = entries.len();
    for (idx, entry) in entries.into_iter().enumerate() {
        let reveal = staggered_text_progress(text_progress, idx, total, 1.2);
        if reveal <= 0.0 {
            break;
        }
        let mut label = Label::new(entry.content, entry.height)
            .with_color(state.config.text_color)
            .with_char_reveal(reveal);
        label.typewriter_mode = true;
        ctx.with_offset(entry.offset, |ctx| label.project(ctx));
    }
}

macro_rules! impl_table_api {
    ($name:ident) => {
        impl $name {
            pub fn new(data: Vec<Vec<impl Into<String>>>) -> Self {
                Self {
                    state: TableState::new(data),
                }
            }

            pub fn with_row_labels(mut self, labels: Vec<impl Into<String>>) -> Self {
                self.state.row_labels = Some(labels.into_iter().map(Into::into).collect());
                self
            }

            pub fn with_col_labels(mut self, labels: Vec<impl Into<String>>) -> Self {
                self.state.col_labels = Some(labels.into_iter().map(Into::into).collect());
                self
            }

            pub fn with_config(mut self, config: TableConfig) -> Self {
                self.state.config = config;
                self
            }

            pub fn with_title(mut self, title: impl Into<String>) -> Self {
                self.state.title = Some(title.into());
                self
            }

            pub fn with_title_color(mut self, color: Vec4) -> Self {
                self.state.title_color = Some(color);
                self
            }

            pub fn with_title_height(mut self, height: f32) -> Self {
                self.state.title_height = Some(height.max(0.01));
                self
            }

            pub fn with_title_position(mut self, position: TableTitlePosition) -> Self {
                self.state.title_position = position;
                self
            }

            pub fn with_h_buff(mut self, h_buff: f32) -> Self {
                self.state.config.h_buff = h_buff;
                self
            }

            pub fn with_v_buff(mut self, v_buff: f32) -> Self {
                self.state.config.v_buff = v_buff;
                self
            }

            pub fn with_line_color(mut self, color: Vec4) -> Self {
                self.state.config.line_color = color;
                self
            }

            pub fn with_line_thickness(mut self, thickness: f32) -> Self {
                self.state.config.line_thickness = thickness;
                self
            }

            pub fn with_text_color(mut self, color: Vec4) -> Self {
                self.state.config.text_color = color;
                self
            }

            pub fn with_text_height(mut self, height: f32) -> Self {
                self.state.config.text_height = height;
                self
            }

            pub fn with_outer_lines(mut self, include: bool) -> Self {
                self.state.config.include_outer_lines = include;
                self
            }

            pub fn with_background_color(mut self, color: Vec4) -> Self {
                self.state.config.background_color = Some(color);
                self
            }

            pub fn with_labels_inside(mut self, inside: bool) -> Self {
                self.state.config.labels_inside = inside;
                self
            }

            pub fn with_write_progress(mut self, progress: f32) -> Self {
                self.state.write_progress = progress.clamp(0.0, 1.0);
                self
            }

            pub fn num_rows(&self) -> usize {
                self.state.num_rows()
            }

            pub fn num_cols(&self) -> usize {
                self.state.num_cols()
            }

            pub fn get_cell(&self, row: usize, col: usize) -> Option<&TableCell> {
                self.state.get_cell(row, col)
            }

            pub fn write_progress(&self) -> f32 {
                self.state.write_progress
            }

            pub fn set_write_progress(&mut self, progress: f32) {
                self.state.write_progress = progress.clamp(0.0, 1.0);
            }
        }
    };
}

/// Previous table implementation retained for compatibility.
#[derive(Clone, Debug)]
pub struct TableV1 {
    state: TableState,
}

impl_table_api!(TableV1);

impl Project for TableV1 {
    fn project(&self, ctx: &mut ProjectionCtx) {
        if self.state.write_progress <= 0.0 {
            return;
        }
        let line_progress = (self.state.write_progress * 2.0).min(1.0);
        let text_progress = ((self.state.write_progress - 0.42) / 0.58).clamp(0.0, 1.0);
        draw_lines(&self.state, ctx, line_progress);
        if text_progress > 0.0 {
            project_v1_text(&self.state, ctx, text_progress);
        }
    }
}

impl Bounded for TableV1 {
    fn local_bounds(&self) -> Bounds {
        let (width, height) = self.state.calculate_dimensions();
        Bounds {
            min: Vec2::new(-width / 2.0, -height / 2.0),
            max: Vec2::new(width / 2.0, height / 2.0),
        }
    }
}

/// New table implementation with progressive line drawing and label-style text reveal.
#[derive(Clone, Debug)]
pub struct Table {
    state: TableState,
}

impl_table_api!(Table);

impl Project for Table {
    fn project(&self, ctx: &mut ProjectionCtx) {
        if self.state.write_progress <= 0.0 {
            return;
        }
        let line_progress = (self.state.write_progress / 0.52).clamp(0.0, 1.0);
        let text_progress = ((self.state.write_progress - 0.28) / 0.72).clamp(0.0, 1.0);
        draw_lines(&self.state, ctx, line_progress);
        if text_progress > 0.0 {
            project_written_text(&self.state, ctx, text_progress);
        }
    }
}

impl Bounded for Table {
    fn local_bounds(&self) -> Bounds {
        let (width, height) = self.state.calculate_dimensions();
        Bounds {
            min: Vec2::new(-width / 2.0, -height / 2.0),
            max: Vec2::new(width / 2.0, height / 2.0),
        }
    }
}
