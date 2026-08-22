use glam::{Mat2, Vec2, Vec4, vec2, vec3};

use crate::frontend::collection::maths::notation::matrix::Matrix;
use crate::frontend::collection::primitives::polygon::Polygon;
use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};

use super::EPSILON;
use super::basis::BasisVectors2D;
use super::vector::{LabeledVector2D, VectorLabelAnchor};

fn emit_line(ctx: &mut ProjectionCtx, start: Vec2, end: Vec2, thickness: f32, color: Vec4) {
    ctx.emit(RenderPrimitive::Line {
        start: vec3(start.x, start.y, 0.0),
        end: vec3(end.x, end.y, 0.0),
        thickness,
        color,
        dash_length: 0.0,
        gap_length: 0.0,
        dash_offset: 0.0,
    });
}

fn transform_point(matrix: Mat2, point: Vec2) -> Vec2 {
    matrix * point
}

fn matrix_columns(matrix: Mat2) -> (Vec2, Vec2) {
    (matrix * Vec2::X, matrix * Vec2::Y)
}

fn bounds_from_points(points: &[Vec2], padding: f32) -> Bounds {
    let first = points.first().copied().unwrap_or(Vec2::ZERO);
    let bounds = points
        .iter()
        .copied()
        .fold(Bounds::new(first, first), |bounds, point| {
            Bounds::new(bounds.min.min(point), bounds.max.max(point))
        });
    Bounds::new(
        bounds.min - Vec2::splat(padding),
        bounds.max + Vec2::splat(padding),
    )
}

fn fmt_entry(value: f32) -> String {
    if value.abs() <= EPSILON {
        "0".to_string()
    } else if (value - value.round()).abs() <= 1.0e-4 {
        format!("{:.0}", value)
    } else {
        format!("{:.2}", value)
    }
}

#[derive(Debug, Clone)]
/// Experimental API: this type is part of the evolving linear algebra visual toolkit.
pub struct TransformableGrid2D {
    pub matrix: Mat2,
    pub x_range: (f32, f32),
    pub y_range: (f32, f32),
    pub step: f32,
    pub grid_color: Vec4,
    pub axis_color: Vec4,
    pub source_grid_color: Vec4,
    pub grid_thickness: f32,
    pub axis_thickness: f32,
    pub show_source_grid: bool,
    pub show_basis_vectors: bool,
    pub i_color: Vec4,
    pub j_color: Vec4,
}

impl TransformableGrid2D {
    pub fn new(matrix: Mat2) -> Self {
        Self {
            matrix,
            x_range: (-4.0, 4.0),
            y_range: (-3.0, 3.0),
            step: 1.0,
            grid_color: Vec4::new(0.34, 0.78, 0.95, 0.42),
            axis_color: Vec4::new(0.90, 0.92, 0.96, 0.82),
            source_grid_color: Vec4::new(0.55, 0.58, 0.64, 0.22),
            grid_thickness: 0.012,
            axis_thickness: 0.035,
            show_source_grid: true,
            show_basis_vectors: true,
            i_color: Vec4::new(0.34, 0.78, 0.95, 1.0),
            j_color: Vec4::new(0.98, 0.74, 0.28, 1.0),
        }
    }

    pub fn identity() -> Self {
        Self::new(Mat2::IDENTITY)
    }

    pub fn from_columns(i: Vec2, j: Vec2) -> Self {
        Self::new(Mat2::from_cols(i, j))
    }

    pub fn with_range(mut self, x_range: (f32, f32), y_range: (f32, f32)) -> Self {
        self.x_range = x_range;
        self.y_range = y_range;
        self
    }

    pub fn with_step(mut self, step: f32) -> Self {
        self.step = step.abs().max(0.1);
        self
    }

    pub fn with_source_grid(mut self, show: bool) -> Self {
        self.show_source_grid = show;
        self
    }

    pub fn with_basis_vectors(mut self, show: bool) -> Self {
        self.show_basis_vectors = show;
        self
    }

    pub fn determinant(&self) -> f32 {
        self.matrix.determinant()
    }

    pub fn transformed_basis(&self) -> BasisVectors2D {
        let (i, j) = matrix_columns(self.matrix);
        BasisVectors2D::new(i, j)
            .with_labels("Ae1", "Ae2")
            .with_coordinates(false)
    }

    pub fn transform_vector(&self, vector: Vec2) -> Vec2 {
        transform_point(self.matrix, vector)
    }

    fn project_grid_lines(
        &self,
        ctx: &mut ProjectionCtx,
        matrix: Mat2,
        color: Vec4,
        axis_color: Vec4,
    ) {
        let mut x = self.x_range.0;
        while x <= self.x_range.1 + EPSILON {
            let is_axis = x.abs() <= EPSILON;
            let start = transform_point(matrix, vec2(x, self.y_range.0));
            let end = transform_point(matrix, vec2(x, self.y_range.1));
            emit_line(
                ctx,
                start,
                end,
                if is_axis {
                    self.axis_thickness
                } else {
                    self.grid_thickness
                },
                if is_axis { axis_color } else { color },
            );
            x += self.step;
        }

        let mut y = self.y_range.0;
        while y <= self.y_range.1 + EPSILON {
            let is_axis = y.abs() <= EPSILON;
            let start = transform_point(matrix, vec2(self.x_range.0, y));
            let end = transform_point(matrix, vec2(self.x_range.1, y));
            emit_line(
                ctx,
                start,
                end,
                if is_axis {
                    self.axis_thickness
                } else {
                    self.grid_thickness
                },
                if is_axis { axis_color } else { color },
            );
            y += self.step;
        }
    }
}

impl Project for TransformableGrid2D {
    fn project(&self, ctx: &mut ProjectionCtx) {
        if self.show_source_grid {
            self.project_grid_lines(
                ctx,
                Mat2::IDENTITY,
                self.source_grid_color,
                self.source_grid_color,
            );
        }

        self.project_grid_lines(ctx, self.matrix, self.grid_color, self.axis_color);

        if self.show_basis_vectors {
            let (i, j) = matrix_columns(self.matrix);
            LabeledVector2D::new("Ae1", i)
                .with_color(self.i_color)
                .with_anchor(VectorLabelAnchor::Tip)
                .project(ctx);
            LabeledVector2D::new("Ae2", j)
                .with_color(self.j_color)
                .with_anchor(VectorLabelAnchor::Tip)
                .project(ctx);
        }
    }
}

impl Bounded for TransformableGrid2D {
    fn local_bounds(&self) -> Bounds {
        let corners = [
            vec2(self.x_range.0, self.y_range.0),
            vec2(self.x_range.1, self.y_range.0),
            vec2(self.x_range.1, self.y_range.1),
            vec2(self.x_range.0, self.y_range.1),
        ];
        let transformed = corners.map(|corner| transform_point(self.matrix, corner));
        bounds_from_points(&transformed, 0.5)
    }
}

#[derive(Debug, Clone)]
/// Experimental API: this type is part of the evolving linear algebra visual toolkit.
pub struct MatrixTransformPanel {
    pub matrix: Mat2,
    pub cell_height: f32,
    pub i_color: Vec4,
    pub j_color: Vec4,
    pub highlight_columns: bool,
}

impl MatrixTransformPanel {
    pub fn new(matrix: Mat2) -> Self {
        Self {
            matrix,
            cell_height: 0.32,
            i_color: Vec4::new(0.34, 0.78, 0.95, 1.0),
            j_color: Vec4::new(0.98, 0.74, 0.28, 1.0),
            highlight_columns: true,
        }
    }

    pub fn from_columns(i: Vec2, j: Vec2) -> Self {
        Self::new(Mat2::from_cols(i, j))
    }

    pub fn with_cell_height(mut self, cell_height: f32) -> Self {
        self.cell_height = cell_height.max(0.05);
        self
    }

    pub fn with_column_highlights(mut self, show: bool) -> Self {
        self.highlight_columns = show;
        self
    }

    pub fn to_matrix(&self) -> Matrix {
        let x_image = self.matrix * Vec2::X;
        let y_image = self.matrix * Vec2::Y;
        let mut matrix = Matrix::new(
            vec![
                vec![fmt_entry(x_image.x), fmt_entry(y_image.x)],
                vec![fmt_entry(x_image.y), fmt_entry(y_image.y)],
            ],
            self.cell_height,
        );

        if self.highlight_columns {
            let mut i_highlight = self.i_color;
            i_highlight.w = 0.18;
            let mut j_highlight = self.j_color;
            j_highlight.w = 0.18;
            for row in 0..2 {
                if let Some(cell) = matrix.cell_mut(row, 0) {
                    *cell = cell
                        .clone()
                        .with_color(self.i_color)
                        .with_highlight(i_highlight);
                }
                if let Some(cell) = matrix.cell_mut(row, 1) {
                    *cell = cell
                        .clone()
                        .with_color(self.j_color)
                        .with_highlight(j_highlight);
                }
            }
        }

        matrix
    }
}

impl Project for MatrixTransformPanel {
    fn project(&self, ctx: &mut ProjectionCtx) {
        self.to_matrix().project(ctx);
    }
}

impl Bounded for MatrixTransformPanel {
    fn local_bounds(&self) -> Bounds {
        self.to_matrix().local_bounds()
    }
}

#[derive(Debug, Clone)]
/// Experimental API: this type is part of the evolving linear algebra visual toolkit.
pub struct MatrixVectorFlow {
    pub matrix_rows: Vec<Vec<f32>>,
    pub input_values: Vec<f32>,
    pub matrix_label: String,
    pub input_label: String,
    pub output_label: String,
    pub text_height: f32,
    pub color: Vec4,
    pub matrix_position: Vec2,
    pub input_position: Vec2,
    pub equals_position: Vec2,
    pub output_position: Vec2,
    pub expansion_position: Vec2,
    pub matrix_cell_height: f32,
    pub show_row_expansion: bool,
}

impl MatrixVectorFlow {
    pub fn new(matrix: Mat2, input: Vec2) -> Self {
        let x_image = matrix * Vec2::X;
        let y_image = matrix * Vec2::Y;
        Self {
            matrix_rows: vec![vec![x_image.x, y_image.x], vec![x_image.y, y_image.y]],
            input_values: vec![input.x, input.y],
            matrix_label: "A".to_string(),
            input_label: "x".to_string(),
            output_label: "Ax".to_string(),
            text_height: 0.22,
            color: Vec4::ONE,
            matrix_position: vec2(-1.95, 0.2),
            input_position: vec2(-0.65, 0.2),
            equals_position: vec2(0.35, 0.2),
            output_position: vec2(1.3, 0.2),
            expansion_position: vec2(-0.2, -0.78),
            matrix_cell_height: 0.26,
            show_row_expansion: true,
        }
    }

    /// Experimental API: rectangular matrix-flow construction may change as layout support matures.
    pub fn try_from_rows(
        matrix_rows: Vec<Vec<f32>>,
        input_values: Vec<f32>,
    ) -> Result<Self, String> {
        let column_count = matrix_rows.first().map_or(0, Vec::len);
        if matrix_rows.is_empty() {
            return Err("matrix must have at least one row".to_string());
        }
        if column_count == 0 {
            return Err("matrix must have at least one column".to_string());
        }
        if matrix_rows.iter().any(|row| row.len() != column_count) {
            return Err("matrix rows must all have the same length".to_string());
        }
        if input_values.len() != column_count {
            return Err(format!(
                "input length {} does not match matrix column count {}",
                input_values.len(),
                column_count
            ));
        }

        let mut flow = Self::new(Mat2::IDENTITY, Vec2::ZERO);
        flow.matrix_rows = matrix_rows;
        flow.input_values = input_values;
        Ok(flow)
    }

    /// Experimental API: rectangular matrix-flow helpers may change as layout support matures.
    pub fn row_count(&self) -> usize {
        self.matrix_rows.len()
    }

    /// Experimental API: rectangular matrix-flow helpers may change as layout support matures.
    pub fn column_count(&self) -> usize {
        self.matrix_rows.first().map_or(0, Vec::len)
    }

    /// Experimental API: rectangular matrix-flow helpers may change as layout support matures.
    pub fn is_compatible(&self) -> bool {
        let columns = self.column_count();
        columns > 0
            && self.input_values.len() == columns
            && self.matrix_rows.iter().all(|row| row.len() == columns)
    }

    /// Experimental API: use this for rectangular outputs; naming may change before stabilization.
    pub fn result_values(&self) -> Vec<f32> {
        if !self.is_compatible() {
            return Vec::new();
        }

        self.matrix_rows
            .iter()
            .map(|row| {
                row.iter()
                    .zip(&self.input_values)
                    .map(|(entry, input)| entry * input)
                    .sum()
            })
            .collect()
    }

    /// Experimental API: 2D convenience result for legacy examples; rectangular callers should use
    /// `result_values`.
    pub fn result(&self) -> Vec2 {
        let result = self.result_values();
        vec2(
            result.first().copied().unwrap_or(0.0),
            result.get(1).copied().unwrap_or(0.0),
        )
    }

    /// Experimental API: alias for `result_values`; naming may change before stabilization.
    pub fn row_dot_products(&self) -> Vec<f32> {
        self.result_values()
    }

    pub fn with_labels(
        mut self,
        matrix: impl Into<String>,
        input: impl Into<String>,
        output: impl Into<String>,
    ) -> Self {
        self.matrix_label = matrix.into();
        self.input_label = input.into();
        self.output_label = output.into();
        self
    }

    pub fn with_row_expansion(mut self, show: bool) -> Self {
        self.show_row_expansion = show;
        self
    }

    /// Experimental API: layout controls may change as matrix-flow composition matures.
    pub fn with_positions(mut self, matrix: Vec2, input: Vec2, equals: Vec2, output: Vec2) -> Self {
        self.matrix_position = matrix;
        self.input_position = input;
        self.equals_position = equals;
        self.output_position = output;
        self
    }

    /// Experimental API: layout controls may change as matrix-flow composition matures.
    pub fn with_expansion_position(mut self, position: Vec2) -> Self {
        self.expansion_position = position;
        self
    }

    /// Experimental API: layout controls may change as matrix-flow composition matures.
    pub fn with_text_height(mut self, text_height: f32) -> Self {
        self.text_height = text_height.max(0.01);
        self
    }

    /// Experimental API: layout controls may change as matrix-flow composition matures.
    pub fn with_matrix_cell_height(mut self, cell_height: f32) -> Self {
        self.matrix_cell_height = cell_height.max(0.05);
        self
    }

    fn emit_text(
        ctx: &mut ProjectionCtx,
        content: impl Into<String>,
        pos: Vec2,
        height: f32,
        color: Vec4,
    ) {
        ctx.emit(RenderPrimitive::Text {
            content: content.into(),
            height,
            color,
            font_name: None,
            offset: vec3(pos.x, pos.y, 0.0),
            rotation: 0.0,
        });
    }

    fn emit_readout(ctx: &mut ProjectionCtx, values: &[f32], pos: Vec2, cell_height: f32) {
        let entries = values
            .iter()
            .map(|value| vec![fmt_entry(*value)])
            .collect::<Vec<_>>();
        let readout = Matrix::new(entries, cell_height);
        ctx.with_offset(vec3(pos.x, pos.y, 0.0), |ctx| readout.project(ctx));
    }

    fn matrix_panel(&self) -> Matrix {
        Matrix::new(
            self.matrix_rows
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|value| fmt_entry(*value))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
            self.matrix_cell_height,
        )
    }

    fn row_expansion_lines(&self) -> Vec<String> {
        let results = self.result_values();
        self.matrix_rows
            .iter()
            .zip(results)
            .map(|(row, result)| {
                let terms = row
                    .iter()
                    .zip(&self.input_values)
                    .map(|(entry, input)| format!("{}*{}", fmt_entry(*entry), fmt_entry(*input)))
                    .collect::<Vec<_>>()
                    .join(" + ");
                format!("{terms} = {}", fmt_entry(result))
            })
            .collect()
    }
}

impl Project for MatrixVectorFlow {
    fn project(&self, ctx: &mut ProjectionCtx) {
        if !self.is_compatible() {
            Self::emit_text(
                ctx,
                "incompatible matrix/vector sizes",
                Vec2::ZERO,
                self.text_height,
                Vec4::new(0.95, 0.36, 0.34, 1.0),
            );
            return;
        }

        let matrix_panel = self.matrix_panel();
        ctx.with_offset(
            vec3(self.matrix_position.x, self.matrix_position.y, 0.0),
            |ctx| {
                matrix_panel.project(ctx);
            },
        );

        Self::emit_readout(
            ctx,
            &self.input_values,
            self.input_position,
            self.matrix_cell_height,
        );
        Self::emit_text(
            ctx,
            "=",
            self.equals_position,
            self.text_height * 1.25,
            self.color,
        );
        Self::emit_readout(
            ctx,
            &self.result_values(),
            self.output_position,
            self.matrix_cell_height,
        );

        Self::emit_text(
            ctx,
            &self.matrix_label,
            self.matrix_position + vec2(0.0, 0.82),
            self.text_height,
            self.color,
        );
        Self::emit_text(
            ctx,
            &self.input_label,
            self.input_position + vec2(0.0, 0.82),
            self.text_height,
            self.color,
        );
        Self::emit_text(
            ctx,
            &self.output_label,
            self.output_position + vec2(0.0, 0.82),
            self.text_height,
            self.color,
        );

        if self.show_row_expansion {
            for (idx, line) in self.row_expansion_lines().iter().enumerate() {
                Self::emit_text(
                    ctx,
                    line,
                    self.expansion_position - vec2(0.0, idx as f32 * self.text_height * 1.45),
                    self.text_height * 0.82,
                    Vec4::new(
                        self.color.x,
                        self.color.y,
                        self.color.z,
                        self.color.w * 0.72,
                    ),
                );
            }
        }
    }
}

impl Bounded for MatrixVectorFlow {
    fn local_bounds(&self) -> Bounds {
        let mut bounds = self
            .matrix_panel()
            .local_bounds()
            .translate(self.matrix_position);

        let input_height = self.input_values.len().max(1) as f32 * self.matrix_cell_height * 1.45;
        bounds = bounds.union(&Bounds::from_center_size(
            self.input_position,
            vec2(self.matrix_cell_height * 1.8, input_height),
        ));

        let result_height =
            self.result_values().len().max(1) as f32 * self.matrix_cell_height * 1.45;
        bounds = bounds.union(&Bounds::from_center_size(
            self.output_position,
            vec2(self.matrix_cell_height * 1.8, result_height),
        ));

        bounds = bounds.union(&Bounds::from_center_size(
            self.equals_position,
            vec2(self.text_height, self.text_height),
        ));

        if self.show_row_expansion {
            let lines = self.row_expansion_lines();
            let expansion_height = lines.len().max(1) as f32 * self.text_height * 1.45;
            bounds = bounds.union(&Bounds::from_center_size(
                self.expansion_position - vec2(0.0, expansion_height * 0.5),
                vec2(self.text_height * 18.0, expansion_height),
            ));
        }

        bounds
    }
}

#[derive(Debug, Clone)]
/// Experimental API: this type is part of the evolving linear algebra visual toolkit.
pub struct DeterminantAreaView {
    pub matrix: Mat2,
    pub fill_color: Vec4,
    pub stroke_color: Vec4,
    pub collapsed_color: Vec4,
    pub text_color: Vec4,
    pub i_color: Vec4,
    pub j_color: Vec4,
    pub stroke_thickness: f32,
    pub show_basis_vectors: bool,
    pub show_label: bool,
}

impl DeterminantAreaView {
    pub fn new(matrix: Mat2) -> Self {
        Self {
            matrix,
            fill_color: Vec4::new(0.44, 0.86, 0.52, 0.28),
            stroke_color: Vec4::new(0.44, 0.86, 0.52, 0.92),
            collapsed_color: Vec4::new(0.95, 0.36, 0.34, 0.92),
            text_color: Vec4::ONE,
            i_color: Vec4::new(0.34, 0.78, 0.95, 1.0),
            j_color: Vec4::new(0.98, 0.74, 0.28, 1.0),
            stroke_thickness: 0.035,
            show_basis_vectors: true,
            show_label: true,
        }
    }

    pub fn determinant(&self) -> f32 {
        self.matrix.determinant()
    }

    pub fn area_scale(&self) -> f32 {
        self.determinant().abs()
    }

    pub fn orientation_sign(&self) -> f32 {
        self.determinant().signum()
    }

    pub fn is_collapsed(&self) -> bool {
        self.area_scale() <= EPSILON
    }

    pub fn transformed_unit_square(&self) -> [Vec2; 4] {
        [
            Vec2::ZERO,
            transform_point(self.matrix, Vec2::X),
            transform_point(self.matrix, Vec2::X + Vec2::Y),
            transform_point(self.matrix, Vec2::Y),
        ]
    }

    pub fn with_label(mut self, show: bool) -> Self {
        self.show_label = show;
        self
    }

    pub fn with_basis_vectors(mut self, show: bool) -> Self {
        self.show_basis_vectors = show;
        self
    }

    fn label_text(&self) -> String {
        let det = self.determinant();
        if self.is_collapsed() {
            "det(A) = 0, area collapses".to_string()
        } else if det < 0.0 {
            format!("det(A) = {}, area flips", fmt_entry(det))
        } else {
            format!(
                "det(A) = {}, area scales by {}",
                fmt_entry(det),
                fmt_entry(det.abs())
            )
        }
    }
}

impl Project for DeterminantAreaView {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let square = self.transformed_unit_square();

        if self.is_collapsed() {
            emit_line(
                ctx,
                square[0],
                square[2],
                self.stroke_thickness * 1.6,
                self.collapsed_color,
            );
        } else {
            Polygon::new(square.to_vec(), self.fill_color)
                .with_stroke(self.stroke_thickness, self.stroke_color)
                .project(ctx);
        }

        if self.show_basis_vectors {
            let (i, j) = matrix_columns(self.matrix);
            LabeledVector2D::new("Ae1", i)
                .with_color(self.i_color)
                .with_anchor(VectorLabelAnchor::Tip)
                .project(ctx);
            LabeledVector2D::new("Ae2", j)
                .with_color(self.j_color)
                .with_anchor(VectorLabelAnchor::Tip)
                .project(ctx);
        }

        if self.show_label {
            let bounds = self.local_bounds();
            MatrixVectorFlow::emit_text(
                ctx,
                self.label_text(),
                vec2(bounds.center().x, bounds.max.y + 0.35),
                0.2,
                self.text_color,
            );
        }
    }
}

impl Bounded for DeterminantAreaView {
    fn local_bounds(&self) -> Bounds {
        bounds_from_points(&self.transformed_unit_square(), 0.45)
    }
}
