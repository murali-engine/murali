use glam::{Mat2, Vec2, Vec4, vec2, vec3};

use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};

use super::EPSILON;
use super::vector::{LabeledVector2D, VectorLabelAnchor};

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

#[derive(Debug, Clone)]
/// Experimental API: this type is part of the evolving linear algebra visual toolkit.
pub struct BasisVectors2D {
    pub i: Vec2,
    pub j: Vec2,
    pub i_label: String,
    pub j_label: String,
    pub i_color: Vec4,
    pub j_color: Vec4,
    pub i_label_offset: Vec2,
    pub j_label_offset: Vec2,
    pub thickness: f32,
    pub show_coordinates: bool,
}

impl BasisVectors2D {
    pub fn standard() -> Self {
        Self::new(Vec2::X, Vec2::Y)
    }

    pub fn new(i: Vec2, j: Vec2) -> Self {
        Self {
            i,
            j,
            i_label: "i".to_string(),
            j_label: "j".to_string(),
            i_color: Vec4::new(0.34, 0.78, 0.95, 1.0),
            j_color: Vec4::new(0.98, 0.74, 0.28, 1.0),
            i_label_offset: vec2(0.16, 0.16),
            j_label_offset: vec2(0.16, 0.16),
            thickness: 0.045,
            show_coordinates: false,
        }
    }

    pub fn with_labels(mut self, i: impl Into<String>, j: impl Into<String>) -> Self {
        self.i_label = i.into();
        self.j_label = j.into();
        self
    }

    pub fn with_label_offsets(mut self, i_offset: Vec2, j_offset: Vec2) -> Self {
        self.i_label_offset = i_offset;
        self.j_label_offset = j_offset;
        self
    }

    pub fn with_thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness.max(0.0);
        self
    }

    pub fn with_coordinates(mut self, show: bool) -> Self {
        self.show_coordinates = show;
        self
    }

    pub fn determinant(&self) -> f32 {
        self.i.x * self.j.y - self.i.y * self.j.x
    }

    pub fn is_independent(&self) -> bool {
        self.determinant().abs() > EPSILON
    }

    /// Experimental API: basis-conversion helpers may change as the coordinate model settles.
    pub fn matrix(&self) -> Mat2 {
        Mat2::from_cols(self.i, self.j)
    }

    /// Experimental API: basis-conversion helpers may change as the coordinate model settles.
    pub fn coordinates_of(&self, vector: Vec2) -> Option<Vec2> {
        if !self.is_independent() {
            return None;
        }

        Some(self.matrix().inverse() * vector)
    }

    /// Experimental API: basis-conversion helpers may change as the coordinate model settles.
    pub fn vector_from_coordinates(&self, coordinates: Vec2) -> Vec2 {
        self.matrix() * coordinates
    }
}

impl Project for BasisVectors2D {
    fn project(&self, ctx: &mut ProjectionCtx) {
        LabeledVector2D::new(&self.i_label, self.i)
            .with_color(self.i_color)
            .with_thickness(self.thickness)
            .with_anchor(VectorLabelAnchor::Tip)
            .with_label_offset(self.i_label_offset)
            .with_coordinates(self.show_coordinates)
            .project(ctx);
        LabeledVector2D::new(&self.j_label, self.j)
            .with_color(self.j_color)
            .with_thickness(self.thickness)
            .with_anchor(VectorLabelAnchor::Tip)
            .with_label_offset(self.j_label_offset)
            .with_coordinates(self.show_coordinates)
            .project(ctx);
    }
}

impl Bounded for BasisVectors2D {
    fn local_bounds(&self) -> Bounds {
        bounds_from_points(&[Vec2::ZERO, self.i, self.j], 0.45)
    }
}

#[derive(Debug, Clone)]
/// Experimental API: this type is part of the evolving linear algebra visual toolkit.
pub struct BasisGrid2D {
    pub basis: BasisVectors2D,
    pub u_range: (f32, f32),
    pub v_range: (f32, f32),
    pub step: f32,
    pub color: Vec4,
    pub axis_color: Vec4,
    pub thickness: f32,
    pub axis_thickness: f32,
}

impl BasisGrid2D {
    pub fn new(basis: BasisVectors2D) -> Self {
        Self {
            basis,
            u_range: (-4.0, 4.0),
            v_range: (-3.0, 3.0),
            step: 1.0,
            color: Vec4::new(0.58, 0.72, 0.98, 0.32),
            axis_color: Vec4::new(0.88, 0.90, 0.94, 0.68),
            thickness: 0.018,
            axis_thickness: 0.035,
        }
    }

    pub fn from_vectors(i: Vec2, j: Vec2) -> Self {
        Self::new(BasisVectors2D::new(i, j))
    }

    pub fn with_range(mut self, u_range: (f32, f32), v_range: (f32, f32)) -> Self {
        self.u_range = u_range;
        self.v_range = v_range;
        self
    }

    pub fn with_step(mut self, step: f32) -> Self {
        self.step = step.abs().max(0.1);
        self
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }

    pub fn with_axis_color(mut self, color: Vec4) -> Self {
        self.axis_color = color;
        self
    }

    pub fn with_thickness(mut self, thickness: f32) -> Self {
        self.thickness = thickness.max(0.0);
        self
    }

    pub fn with_axis_thickness(mut self, thickness: f32) -> Self {
        self.axis_thickness = thickness.max(0.0);
        self
    }

    pub fn basis_point(&self, coordinates: Vec2) -> Vec2 {
        self.basis.vector_from_coordinates(coordinates)
    }

    fn emit_line(&self, ctx: &mut ProjectionCtx, start: Vec2, end: Vec2, is_axis: bool) {
        ctx.emit(RenderPrimitive::Line {
            start: vec3(start.x, start.y, 0.0),
            end: vec3(end.x, end.y, 0.0),
            thickness: if is_axis {
                self.axis_thickness
            } else {
                self.thickness
            },
            color: if is_axis { self.axis_color } else { self.color },
            dash_length: 0.0,
            gap_length: 0.0,
            dash_offset: 0.0,
        });
    }
}

impl Project for BasisGrid2D {
    fn project(&self, ctx: &mut ProjectionCtx) {
        if !self.basis.is_independent() {
            return;
        }

        let mut u = self.u_range.0;
        while u <= self.u_range.1 + EPSILON {
            self.emit_line(
                ctx,
                self.basis_point(vec2(u, self.v_range.0)),
                self.basis_point(vec2(u, self.v_range.1)),
                u.abs() <= EPSILON,
            );
            u += self.step;
        }

        let mut v = self.v_range.0;
        while v <= self.v_range.1 + EPSILON {
            self.emit_line(
                ctx,
                self.basis_point(vec2(self.u_range.0, v)),
                self.basis_point(vec2(self.u_range.1, v)),
                v.abs() <= EPSILON,
            );
            v += self.step;
        }
    }
}

impl Bounded for BasisGrid2D {
    fn local_bounds(&self) -> Bounds {
        let corners = [
            self.basis_point(vec2(self.u_range.0, self.v_range.0)),
            self.basis_point(vec2(self.u_range.1, self.v_range.0)),
            self.basis_point(vec2(self.u_range.1, self.v_range.1)),
            self.basis_point(vec2(self.u_range.0, self.v_range.1)),
        ];
        bounds_from_points(&corners, 0.35)
    }
}

#[derive(Debug, Clone)]
/// Experimental API: this type is part of the evolving linear algebra visual toolkit.
pub struct SpanRegion2D {
    pub u: Vec2,
    pub v: Option<Vec2>,
    pub extent: f32,
    pub step: f32,
    pub color: Vec4,
    pub thickness: f32,
}

impl SpanRegion2D {
    pub fn line(u: Vec2) -> Self {
        Self {
            u,
            v: None,
            extent: 4.0,
            step: 1.0,
            color: Vec4::new(0.34, 0.78, 0.95, 0.35),
            thickness: 0.025,
        }
    }

    pub fn plane(u: Vec2, v: Vec2) -> Self {
        Self {
            v: Some(v),
            ..Self::line(u)
        }
    }

    pub fn with_extent(mut self, extent: f32) -> Self {
        self.extent = extent.abs().max(0.1);
        self
    }

    pub fn with_step(mut self, step: f32) -> Self {
        self.step = step.abs().max(0.1);
        self
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }

    pub fn is_plane(&self) -> bool {
        self.v
            .map(|v| (self.u.x * v.y - self.u.y * v.x).abs() > EPSILON)
            .unwrap_or(false)
    }

    fn emit_span_line(&self, ctx: &mut ProjectionCtx, start: Vec2, end: Vec2) {
        ctx.emit(RenderPrimitive::Line {
            start: vec3(start.x, start.y, 0.0),
            end: vec3(end.x, end.y, 0.0),
            thickness: self.thickness,
            color: self.color,
            dash_length: 0.0,
            gap_length: 0.0,
            dash_offset: 0.0,
        });
    }
}

impl Project for SpanRegion2D {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let u_dir = self.u.normalize_or_zero();
        if u_dir.length_squared() <= EPSILON {
            return;
        }

        let Some(v) = self.v else {
            let scaled = u_dir * self.extent;
            self.emit_span_line(ctx, -scaled, scaled);
            return;
        };

        let v_dir = v.normalize_or_zero();
        if v_dir.length_squared() <= EPSILON || !self.is_plane() {
            let scaled = u_dir * self.extent;
            self.emit_span_line(ctx, -scaled, scaled);
            return;
        }

        let count = (self.extent / self.step).ceil() as i32;
        for index in -count..=count {
            let offset = v_dir * index as f32 * self.step;
            self.emit_span_line(
                ctx,
                offset - u_dir * self.extent,
                offset + u_dir * self.extent,
            );

            let offset = u_dir * index as f32 * self.step;
            self.emit_span_line(
                ctx,
                offset - v_dir * self.extent,
                offset + v_dir * self.extent,
            );
        }
    }
}

impl Bounded for SpanRegion2D {
    fn local_bounds(&self) -> Bounds {
        let extent = self.extent + self.step;
        Bounds::from_center_size(Vec2::ZERO, vec2(extent * 2.0, extent * 2.0))
    }
}
