use glam::{Vec2, Vec3, Vec4};

use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Mesh, Project, ProjectionCtx, RenderPrimitive};
use crate::resource::text::layout::measure_label;

#[derive(Debug, Clone)]
pub struct MatrixCell {
    pub text: String,
    pub color: Vec4,
    pub key: Option<String>,
    pub opacity: f32,
    pub scale: f32,
    pub highlight: Option<Vec4>,
}

impl MatrixCell {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: Vec4::ONE,
            key: None,
            opacity: 1.0,
            scale: 1.0,
            highlight: None,
        }
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }

    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale.max(0.05);
        self
    }

    pub fn with_highlight(mut self, color: Vec4) -> Self {
        self.highlight = Some(color);
        self
    }

    pub fn continuity_key(&self, row: usize, col: usize) -> String {
        self.key
            .clone()
            .unwrap_or_else(|| format!("{}@{row}:{col}", self.text))
    }
}

#[derive(Debug, Clone)]
pub struct MatrixCellLayout {
    pub row: usize,
    pub col: usize,
    pub key: String,
    pub center: Vec3,
    pub width: f32,
    pub height: f32,
    pub color: Vec4,
    pub opacity: f32,
    pub scale: f32,
    pub highlight: Option<Vec4>,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct Matrix {
    pub entries: Vec<Vec<MatrixCell>>,
    pub cell_height: f32,
    pub h_gap: f32,
    pub v_gap: f32,
    pub color: Vec4,
    pub bracket_color: Vec4,
    pub bracket_thickness: f32,
}

impl Matrix {
    pub fn new(entries: Vec<Vec<impl Into<String>>>, cell_height: f32) -> Self {
        Self {
            entries: entries
                .into_iter()
                .map(|row| row.into_iter().map(MatrixCell::new).collect())
                .collect(),
            cell_height,
            h_gap: cell_height * 0.9,
            v_gap: cell_height * 0.45,
            color: Vec4::ONE,
            bracket_color: Vec4::new(0.88, 0.90, 0.94, 1.0),
            bracket_thickness: 0.03,
        }
    }

    fn dims(&self) -> (usize, usize) {
        let rows = self.entries.len();
        let cols = self.entries.iter().map(|r| r.len()).max().unwrap_or(0);
        (rows, cols)
    }

    fn cell_width(&self, cell: &MatrixCell) -> f32 {
        measure_label(&cell.text, self.cell_height * cell.scale.max(0.05), None)
            .width
            .max(self.cell_height * cell.scale.max(0.05) * 0.7)
    }

    fn max_col_widths(&self) -> Vec<f32> {
        let (_, cols) = self.dims();
        let mut widths = vec![self.cell_height * 0.7; cols];
        for row in &self.entries {
            for (idx, entry) in row.iter().enumerate() {
                widths[idx] = widths[idx].max(self.cell_width(entry));
            }
        }
        widths
    }

    pub fn cell_mut(&mut self, row: usize, col: usize) -> Option<&mut MatrixCell> {
        self.entries.get_mut(row)?.get_mut(col)
    }

    pub fn layout_snapshot(&self) -> Vec<MatrixCellLayout> {
        let (rows, cols) = self.dims();
        if rows == 0 || cols == 0 {
            return Vec::new();
        }

        let col_widths = self.max_col_widths();
        let total_width =
            col_widths.iter().sum::<f32>() + self.h_gap * (cols.saturating_sub(1) as f32);
        let total_height =
            rows as f32 * self.cell_height + self.v_gap * (rows.saturating_sub(1) as f32);

        let left_edge = -total_width * 0.5;
        let top_edge = total_height * 0.5;
        let mut x_positions = Vec::with_capacity(cols);
        let mut cursor_x = left_edge;
        for width in &col_widths {
            x_positions.push(cursor_x + width * 0.5);
            cursor_x += *width + self.h_gap;
        }

        let mut out = Vec::new();
        for (row_idx, row) in self.entries.iter().enumerate() {
            let y = top_edge
                - self.cell_height * 0.5
                - row_idx as f32 * (self.cell_height + self.v_gap);
            for (col_idx, cell) in row.iter().enumerate() {
                out.push(MatrixCellLayout {
                    row: row_idx,
                    col: col_idx,
                    key: cell.continuity_key(row_idx, col_idx),
                    center: Vec3::new(x_positions[col_idx], y, 0.0),
                    width: col_widths[col_idx],
                    height: self.cell_height * cell.scale.max(0.05),
                    color: if cell.color == Vec4::ONE {
                        self.color
                    } else {
                        cell.color
                    },
                    opacity: cell.opacity.clamp(0.0, 1.0),
                    scale: cell.scale.max(0.05),
                    highlight: cell.highlight,
                    text: cell.text.clone(),
                });
            }
        }

        out
    }
}

impl Project for Matrix {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let layout = self.layout_snapshot();
        if layout.is_empty() {
            return;
        }

        for cell in &layout {
            if let Some(mut bg) = cell.highlight {
                bg.w *= cell.opacity;
                let pad_x = self.cell_height * 0.42;
                let pad_y = self.cell_height * 0.28;
                ctx.emit(RenderPrimitive::Mesh(
                    Mesh::rectangle(cell.width + pad_x, cell.height + pad_y, bg)
                        .translated(cell.center),
                ));
            }

            let mut color = cell.color;
            color.w *= cell.opacity;
            ctx.emit(RenderPrimitive::Text {
                content: cell.text.clone(),
                height: self.cell_height * cell.scale,
                color,
                font_name: None,
                offset: cell.center,
                rotation: 0.0,
            });
        }

        let (_, cols) = self.dims();
        let col_widths = self.max_col_widths();
        let total_width =
            col_widths.iter().sum::<f32>() + self.h_gap * (cols.saturating_sub(1) as f32);
        let total_height = self.entries.len() as f32 * self.cell_height
            + self.v_gap * (self.entries.len().saturating_sub(1) as f32);

        let bracket_pad = self.cell_height * 0.45;
        let x_left = -total_width * 0.5 - bracket_pad;
        let x_right = total_width * 0.5 + bracket_pad;
        let y_top = total_height * 0.5 + self.cell_height * 0.35;
        let y_bottom = -total_height * 0.5 - self.cell_height * 0.35;
        let arm = self.cell_height * 0.28;

        for (x, sign) in [(x_left, 1.0_f32), (x_right, -1.0_f32)] {
            ctx.emit(RenderPrimitive::Line {
                start: Vec3::new(x, y_bottom, 0.0),
                end: Vec3::new(x, y_top, 0.0),
                thickness: self.bracket_thickness,
                color: self.bracket_color,
                dash_length: 0.0,
                gap_length: 0.0,
                dash_offset: 0.0,
            });
            ctx.emit(RenderPrimitive::Line {
                start: Vec3::new(x, y_top, 0.0),
                end: Vec3::new(x + arm * sign, y_top, 0.0),
                thickness: self.bracket_thickness,
                color: self.bracket_color,
                dash_length: 0.0,
                gap_length: 0.0,
                dash_offset: 0.0,
            });
            ctx.emit(RenderPrimitive::Line {
                start: Vec3::new(x, y_bottom, 0.0),
                end: Vec3::new(x + arm * sign, y_bottom, 0.0),
                thickness: self.bracket_thickness,
                color: self.bracket_color,
                dash_length: 0.0,
                gap_length: 0.0,
                dash_offset: 0.0,
            });
        }
    }
}

impl Bounded for Matrix {
    fn local_bounds(&self) -> Bounds {
        let (rows, cols) = self.dims();
        if rows == 0 || cols == 0 {
            return Bounds::default();
        }
        let col_widths = self.max_col_widths();
        let total_width =
            col_widths.iter().sum::<f32>() + self.h_gap * (cols.saturating_sub(1) as f32);
        let total_height =
            rows as f32 * self.cell_height + self.v_gap * (rows.saturating_sub(1) as f32);
        let bracket_pad = self.cell_height * 0.45 + self.cell_height * 0.28;
        Bounds::from_center_size(
            Vec2::ZERO,
            Vec2::new(
                total_width + bracket_pad * 2.0,
                total_height + self.cell_height * 0.7,
            ),
        )
    }
}
