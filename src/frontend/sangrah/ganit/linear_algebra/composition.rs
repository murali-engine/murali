use glam::{Mat2, Vec2, Vec4, vec2};

use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Project, ProjectionCtx};

use super::vector::{LabeledVector2D, VectorArrow2D, VectorLabelAnchor};

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
    if value.abs() <= super::EPSILON {
        "0".to_string()
    } else if (value - value.round()).abs() <= 1.0e-4 {
        format!("{:.0}", value)
    } else {
        format!("{:.2}", value)
    }
}

#[derive(Debug, Clone)]
/// Experimental API: this type is part of the evolving linear algebra visual toolkit.
pub struct VectorAdditionView {
    pub a: Vec2,
    pub b: Vec2,
    pub a_label: String,
    pub b_label: String,
    pub sum_label: String,
    pub a_color: Vec4,
    pub b_color: Vec4,
    pub sum_color: Vec4,
    pub guide_color: Vec4,
    pub thickness: f32,
    pub show_parallelogram: bool,
    pub show_result: bool,
}

impl VectorAdditionView {
    pub fn new(a: Vec2, b: Vec2) -> Self {
        Self {
            a,
            b,
            a_label: "a".to_string(),
            b_label: "b".to_string(),
            sum_label: "a + b".to_string(),
            a_color: Vec4::new(0.34, 0.78, 0.95, 1.0),
            b_color: Vec4::new(0.98, 0.74, 0.28, 1.0),
            sum_color: Vec4::new(0.44, 0.86, 0.52, 1.0),
            guide_color: Vec4::new(0.78, 0.82, 0.88, 0.45),
            thickness: 0.04,
            show_parallelogram: true,
            show_result: true,
        }
    }

    pub fn sum(&self) -> Vec2 {
        self.a + self.b
    }

    pub fn with_labels(
        mut self,
        a: impl Into<String>,
        b: impl Into<String>,
        sum: impl Into<String>,
    ) -> Self {
        self.a_label = a.into();
        self.b_label = b.into();
        self.sum_label = sum.into();
        self
    }

    pub fn with_parallelogram(mut self, show: bool) -> Self {
        self.show_parallelogram = show;
        self
    }
}

impl Project for VectorAdditionView {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let sum = self.sum();
        let label_offset = vec2(0.14, 0.14);

        LabeledVector2D::from_arrow(
            &self.a_label,
            VectorArrow2D::from_origin(self.a)
                .with_color(self.a_color)
                .with_thickness(self.thickness),
        )
        .with_anchor(VectorLabelAnchor::ShaftSide)
        .with_label_offset(label_offset)
        .project(ctx);

        LabeledVector2D::from_arrow(
            &self.b_label,
            VectorArrow2D::new(self.a, sum)
                .with_color(self.b_color)
                .with_thickness(self.thickness),
        )
        .with_anchor(VectorLabelAnchor::ShaftSide)
        .with_label_offset(label_offset)
        .project(ctx);

        if self.show_parallelogram {
            VectorArrow2D::emit_line(ctx, self.b, sum, self.thickness * 0.55, self.guide_color);
            VectorArrow2D::emit_line(
                ctx,
                Vec2::ZERO,
                self.b,
                self.thickness * 0.45,
                self.guide_color,
            );
        }

        if self.show_result {
            LabeledVector2D::from_arrow(
                &self.sum_label,
                VectorArrow2D::from_origin(sum)
                    .with_color(self.sum_color)
                    .with_thickness(self.thickness * 1.15),
            )
            .with_anchor(VectorLabelAnchor::Tip)
            .with_label_offset(vec2(0.16, 0.16))
            .project(ctx);
        }
    }
}

impl Bounded for VectorAdditionView {
    fn local_bounds(&self) -> Bounds {
        bounds_from_points(&[Vec2::ZERO, self.a, self.b, self.sum()], 0.45)
    }
}

#[derive(Debug, Clone)]
/// Experimental API: this type is part of the evolving linear algebra visual toolkit.
pub struct ScalarMultiplicationView {
    pub vector: Vec2,
    pub scalar: f32,
    pub base_label: String,
    pub scaled_label: String,
    pub base_color: Vec4,
    pub scaled_color: Vec4,
    pub thickness: f32,
    pub show_base: bool,
}

impl ScalarMultiplicationView {
    pub fn new(vector: Vec2, scalar: f32) -> Self {
        Self {
            vector,
            scalar,
            base_label: "v".to_string(),
            scaled_label: "cv".to_string(),
            base_color: Vec4::new(0.78, 0.82, 0.88, 0.5),
            scaled_color: Vec4::new(0.34, 0.78, 0.95, 1.0),
            thickness: 0.04,
            show_base: true,
        }
    }

    pub fn scaled(&self) -> Vec2 {
        self.vector * self.scalar
    }

    pub fn with_labels(mut self, base: impl Into<String>, scaled: impl Into<String>) -> Self {
        self.base_label = base.into();
        self.scaled_label = scaled.into();
        self
    }
}

impl Project for ScalarMultiplicationView {
    fn project(&self, ctx: &mut ProjectionCtx) {
        if self.show_base {
            LabeledVector2D::new(&self.base_label, self.vector)
                .with_color(self.base_color)
                .with_anchor(VectorLabelAnchor::ShaftSide)
                .project(ctx);
        }

        LabeledVector2D::new(&self.scaled_label, self.scaled())
            .with_color(self.scaled_color)
            .with_anchor(VectorLabelAnchor::Tip)
            .with_coordinates(true)
            .project(ctx);
    }
}

impl Bounded for ScalarMultiplicationView {
    fn local_bounds(&self) -> Bounds {
        bounds_from_points(&[Vec2::ZERO, self.vector, self.scaled()], 0.45)
    }
}

#[derive(Debug, Clone)]
/// Experimental API: this type is part of the evolving linear algebra visual toolkit.
pub struct LinearCombinationView {
    pub u: Vec2,
    pub v: Vec2,
    pub u_coefficient: f32,
    pub v_coefficient: f32,
    pub u_label: String,
    pub v_label: String,
    pub result_label: String,
    pub u_color: Vec4,
    pub v_color: Vec4,
    pub result_color: Vec4,
    pub thickness: f32,
    pub show_components: bool,
}

impl LinearCombinationView {
    pub fn new(u: Vec2, v: Vec2, u_coefficient: f32, v_coefficient: f32) -> Self {
        Self {
            u,
            v,
            u_coefficient,
            v_coefficient,
            u_label: "cu".to_string(),
            v_label: "dv".to_string(),
            result_label: "cu + dv".to_string(),
            u_color: Vec4::new(0.34, 0.78, 0.95, 1.0),
            v_color: Vec4::new(0.98, 0.74, 0.28, 1.0),
            result_color: Vec4::new(0.44, 0.86, 0.52, 1.0),
            thickness: 0.04,
            show_components: true,
        }
    }

    pub fn u_component(&self) -> Vec2 {
        self.u * self.u_coefficient
    }

    pub fn v_component(&self) -> Vec2 {
        self.v * self.v_coefficient
    }

    pub fn result(&self) -> Vec2 {
        self.u_component() + self.v_component()
    }

    pub fn with_labels(
        mut self,
        u: impl Into<String>,
        v: impl Into<String>,
        result: impl Into<String>,
    ) -> Self {
        self.u_label = u.into();
        self.v_label = v.into();
        self.result_label = result.into();
        self
    }
}

impl Project for LinearCombinationView {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let u_component = self.u_component();
        let result = self.result();

        if self.show_components {
            LabeledVector2D::from_arrow(
                &self.u_label,
                VectorArrow2D::from_origin(u_component)
                    .with_color(self.u_color)
                    .with_thickness(self.thickness),
            )
            .with_anchor(VectorLabelAnchor::ShaftSide)
            .project(ctx);

            LabeledVector2D::from_arrow(
                &self.v_label,
                VectorArrow2D::new(u_component, result)
                    .with_color(self.v_color)
                    .with_thickness(self.thickness),
            )
            .with_anchor(VectorLabelAnchor::ShaftSide)
            .project(ctx);
        }

        LabeledVector2D::from_arrow(
            &self.result_label,
            VectorArrow2D::from_origin(result)
                .with_color(self.result_color)
                .with_thickness(self.thickness * 1.15),
        )
        .with_anchor(VectorLabelAnchor::Tip)
        .with_coordinates(true)
        .project(ctx);
    }
}

impl Bounded for LinearCombinationView {
    fn local_bounds(&self) -> Bounds {
        bounds_from_points(&[Vec2::ZERO, self.u_component(), self.result()], 0.55)
    }
}

#[derive(Debug, Clone)]
/// Experimental API: this type is part of the evolving linear algebra visual toolkit.
pub struct ColumnCombinationView {
    pub matrix: Mat2,
    pub coefficients: Vec2,
    pub first_column_label: String,
    pub second_column_label: String,
    pub result_label: String,
    pub target_label: String,
    pub first_color: Vec4,
    pub second_color: Vec4,
    pub result_color: Vec4,
    pub target_color: Vec4,
    pub guide_color: Vec4,
    pub thickness: f32,
    pub show_basis_columns: bool,
    pub show_components: bool,
    pub target: Option<Vec2>,
}

impl ColumnCombinationView {
    pub fn new(matrix: Mat2, coefficients: Vec2) -> Self {
        Self {
            matrix,
            coefficients,
            first_column_label: "a1".to_string(),
            second_column_label: "a2".to_string(),
            result_label: "Ax".to_string(),
            target_label: "b".to_string(),
            first_color: Vec4::new(0.34, 0.78, 0.95, 1.0),
            second_color: Vec4::new(0.98, 0.74, 0.28, 1.0),
            result_color: Vec4::new(0.44, 0.86, 0.52, 1.0),
            target_color: Vec4::new(0.95, 0.36, 0.34, 1.0),
            guide_color: Vec4::new(0.78, 0.82, 0.88, 0.38),
            thickness: 0.04,
            show_basis_columns: true,
            show_components: true,
            target: None,
        }
    }

    pub fn from_columns(first: Vec2, second: Vec2, coefficients: Vec2) -> Self {
        Self::new(Mat2::from_cols(first, second), coefficients)
    }

    pub fn columns(&self) -> (Vec2, Vec2) {
        (self.matrix * Vec2::X, self.matrix * Vec2::Y)
    }

    pub fn first_component(&self) -> Vec2 {
        self.columns().0 * self.coefficients.x
    }

    pub fn second_component(&self) -> Vec2 {
        self.columns().1 * self.coefficients.y
    }

    pub fn result(&self) -> Vec2 {
        self.first_component() + self.second_component()
    }

    pub fn target_residual(&self) -> Option<Vec2> {
        self.target.map(|target| target - self.result())
    }

    pub fn with_labels(
        mut self,
        first_column: impl Into<String>,
        second_column: impl Into<String>,
        result: impl Into<String>,
    ) -> Self {
        self.first_column_label = first_column.into();
        self.second_column_label = second_column.into();
        self.result_label = result.into();
        self
    }

    pub fn with_target(mut self, target: Vec2, label: impl Into<String>) -> Self {
        self.target = Some(target);
        self.target_label = label.into();
        self
    }

    pub fn with_basis_columns(mut self, show: bool) -> Self {
        self.show_basis_columns = show;
        self
    }

    pub fn with_components(mut self, show: bool) -> Self {
        self.show_components = show;
        self
    }

    fn component_label(&self, coefficient: f32, label: &str) -> String {
        format!("{}{}", fmt_entry(coefficient), label)
    }
}

impl Project for ColumnCombinationView {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let (first_column, second_column) = self.columns();
        let first_component = self.first_component();
        let second_component = self.second_component();
        let result = self.result();

        if self.show_basis_columns {
            LabeledVector2D::new(&self.first_column_label, first_column)
                .with_color(Vec4::new(
                    self.first_color.x,
                    self.first_color.y,
                    self.first_color.z,
                    self.first_color.w * 0.42,
                ))
                .with_anchor(VectorLabelAnchor::ShaftSide)
                .project(ctx);
            LabeledVector2D::new(&self.second_column_label, second_column)
                .with_color(Vec4::new(
                    self.second_color.x,
                    self.second_color.y,
                    self.second_color.z,
                    self.second_color.w * 0.42,
                ))
                .with_anchor(VectorLabelAnchor::ShaftSide)
                .project(ctx);
        }

        if self.show_components {
            LabeledVector2D::from_arrow(
                self.component_label(self.coefficients.x, &self.first_column_label),
                VectorArrow2D::from_origin(first_component)
                    .with_color(self.first_color)
                    .with_thickness(self.thickness),
            )
            .with_anchor(VectorLabelAnchor::ShaftSide)
            .project(ctx);

            LabeledVector2D::from_arrow(
                self.component_label(self.coefficients.y, &self.second_column_label),
                VectorArrow2D::new(first_component, result)
                    .with_color(self.second_color)
                    .with_thickness(self.thickness),
            )
            .with_anchor(VectorLabelAnchor::ShaftSide)
            .project(ctx);

            VectorArrow2D::emit_line(
                ctx,
                Vec2::ZERO,
                second_component,
                self.thickness * 0.45,
                self.guide_color,
            );
            VectorArrow2D::emit_line(
                ctx,
                second_component,
                result,
                self.thickness * 0.55,
                self.guide_color,
            );
        }

        LabeledVector2D::new(&self.result_label, result)
            .with_color(self.result_color)
            .with_anchor(VectorLabelAnchor::Tip)
            .with_coordinates(true)
            .project(ctx);

        if let Some(target) = self.target {
            LabeledVector2D::new(&self.target_label, target)
                .with_color(self.target_color)
                .with_anchor(VectorLabelAnchor::Tip)
                .with_coordinates(true)
                .project(ctx);
            VectorArrow2D::emit_line(
                ctx,
                result,
                target,
                self.thickness * 0.55,
                self.target_color,
            );
        }
    }
}

impl Bounded for ColumnCombinationView {
    fn local_bounds(&self) -> Bounds {
        let (first_column, second_column) = self.columns();
        let mut points = vec![
            Vec2::ZERO,
            first_column,
            second_column,
            self.first_component(),
            self.second_component(),
            self.result(),
        ];
        if let Some(target) = self.target {
            points.push(target);
        }

        bounds_from_points(&points, 0.6)
    }
}
