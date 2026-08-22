//! Experimental linear algebra visual components.
//!
//! This module is under active development. Public names, builder methods, layout defaults, and
//! rendering details may change before this API is promoted to stable.
//!
//! The components are available for examples, feedback, and early video production, but should not
//! yet be treated as a stable compatibility contract.
//!
//! Every public item exported from this module is experimental unless its own documentation says
//! otherwise.

mod angle;
mod badge;
mod basis;
mod composition;
mod coordinate;
mod meter;
mod projection;
mod transform;
mod vector;

pub use angle::{AngleArc, AngleUnit, OrthogonalityMarker};
pub use badge::{DimensionBadge, QuantityBadge};
pub use basis::{BasisGrid2D, BasisVectors2D, SpanRegion2D};
pub use composition::{
    ColumnCombinationView, LinearCombinationView, ScalarMultiplicationView, VectorAdditionView,
};
pub use coordinate::{CoordinateReadout, CoordinateReadoutMode};
pub use meter::{DotProductMeter, MeterMode};
pub use projection::ProjectionShadow;
pub use transform::{
    DeterminantAreaView, MatrixTransformPanel, MatrixVectorFlow, TransformableGrid2D,
};
pub use vector::{LabeledVector2D, VectorArrow2D, VectorLabelAnchor};

pub(super) const EPSILON: f32 = 1.0e-5;

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Mat2, Vec2, vec2};

    #[test]
    fn projection_shadow_computes_projection_and_residual() {
        let shadow = ProjectionShadow::new(vec2(2.0, 2.0), vec2(1.0, 0.0));

        assert_eq!(shadow.projection(), vec2(2.0, 0.0));
        assert_eq!(shadow.residual(), vec2(0.0, 2.0));
    }

    #[test]
    fn dot_product_meter_handles_zero_vectors() {
        let meter = DotProductMeter::new(Vec2::ZERO, vec2(1.0, 0.0));

        assert_eq!(meter.dot(), 0.0);
        assert_eq!(meter.cosine(), 0.0);
    }

    #[test]
    fn angle_arc_reports_angle_between_vectors() {
        let arc = AngleArc::between(vec2(1.0, 0.0), vec2(0.0, 1.0));

        assert!((arc.angle() - std::f32::consts::FRAC_PI_2).abs() < 1.0e-5);
    }

    #[test]
    fn orthogonality_marker_reports_right_angle_geometry() {
        let marker = OrthogonalityMarker::new(vec2(2.0, 0.0), vec2(0.0, 3.0)).with_size(0.25);

        assert!(marker.is_orthogonal());
        assert_eq!(
            marker.corner_points(),
            Some([vec2(0.25, 0.0), vec2(0.25, 0.25), vec2(0.0, 0.25)])
        );
    }

    #[test]
    fn orthogonality_marker_ignores_degenerate_directions() {
        let marker = OrthogonalityMarker::new(Vec2::ZERO, vec2(0.0, 1.0));

        assert_eq!(marker.corner_points(), None);
    }

    #[test]
    fn quantity_badge_formats_label_and_value() {
        let badge = QuantityBadge::new("rank", "2");

        assert_eq!(badge.text(), "rank: 2");
        assert!(badge.resolved_size().x >= badge.min_width);
        assert!(badge.resolved_size().y >= badge.min_height);
    }

    #[test]
    fn dimension_badge_formats_matrix_and_vector_shapes() {
        let matrix = DimensionBadge::new("A", 3, 2);
        let vector = DimensionBadge::vector("x", 2);

        assert_eq!(matrix.value(), "3x2");
        assert_eq!(matrix.as_quantity_badge().text(), "A: 3x2");
        assert_eq!(vector.value(), "2x1");
        assert_eq!(vector.as_quantity_badge().text(), "x: 2x1");
    }

    #[test]
    fn vector_addition_view_reports_sum() {
        let view = VectorAdditionView::new(vec2(1.5, -2.0), vec2(0.5, 3.0));

        assert_eq!(view.sum(), vec2(2.0, 1.0));
    }

    #[test]
    fn linear_combination_view_reports_result() {
        let view = LinearCombinationView::new(vec2(2.0, 0.0), vec2(0.0, 3.0), 1.5, -0.5);

        assert_eq!(view.u_component(), vec2(3.0, 0.0));
        assert_eq!(view.v_component(), vec2(0.0, -1.5));
        assert_eq!(view.result(), vec2(3.0, -1.5));
    }

    #[test]
    fn column_combination_view_reports_matrix_product() {
        let matrix = Mat2::from_cols(vec2(2.0, 1.0), vec2(-1.0, 3.0));
        let view = ColumnCombinationView::new(matrix, vec2(1.5, -0.5));

        assert_eq!(view.columns(), (vec2(2.0, 1.0), vec2(-1.0, 3.0)));
        assert_eq!(view.first_component(), vec2(3.0, 1.5));
        assert_eq!(view.second_component(), vec2(0.5, -1.5));
        assert_eq!(view.result(), matrix * vec2(1.5, -0.5));
    }

    #[test]
    fn column_combination_view_reports_target_residual() {
        let matrix = Mat2::from_cols(vec2(2.0, 0.0), vec2(0.0, 3.0));
        let view =
            ColumnCombinationView::new(matrix, vec2(1.0, 1.0)).with_target(vec2(3.0, 2.0), "b");

        assert_eq!(view.result(), vec2(2.0, 3.0));
        assert_eq!(view.target_residual(), Some(vec2(1.0, -1.0)));
    }

    #[test]
    fn basis_vectors_detect_independence() {
        assert!(BasisVectors2D::standard().is_independent());
        assert!(!BasisVectors2D::new(vec2(1.0, 1.0), vec2(2.0, 2.0)).is_independent());
    }

    #[test]
    fn basis_vectors_convert_between_basis_and_standard_coordinates() {
        let basis = BasisVectors2D::new(vec2(2.0, 1.0), vec2(-1.0, 1.0));
        let coordinates = vec2(1.5, -0.5);
        let vector = basis.vector_from_coordinates(coordinates);
        let recovered = basis.coordinates_of(vector).unwrap();

        assert_eq!(vector, vec2(3.5, 1.0));
        assert!((recovered - coordinates).length() < 1.0e-5);
    }

    #[test]
    fn basis_vectors_reject_dependent_coordinate_conversion() {
        let basis = BasisVectors2D::new(vec2(1.0, 1.0), vec2(2.0, 2.0));

        assert_eq!(basis.coordinates_of(vec2(3.0, 3.0)), None);
    }

    #[test]
    fn basis_grid_maps_lattice_points_through_basis() {
        let grid = BasisGrid2D::from_vectors(vec2(2.0, 0.0), vec2(0.5, 1.0));

        assert_eq!(grid.basis_point(vec2(1.0, 2.0)), vec2(3.0, 2.0));
    }

    #[test]
    fn transformable_grid_transforms_vectors() {
        let grid = TransformableGrid2D::from_columns(vec2(2.0, 1.0), vec2(-1.0, 1.5));

        assert_eq!(grid.transform_vector(vec2(3.0, 2.0)), vec2(4.0, 6.0));
        assert!((grid.determinant() - 4.0).abs() < 1.0e-5);
    }

    #[test]
    fn matrix_transform_panel_uses_columns_as_basis_images() {
        let panel = MatrixTransformPanel::from_columns(vec2(2.0, 1.0), vec2(-1.0, 1.5));
        let matrix = panel.to_matrix();

        assert_eq!(matrix.entries[0][0].text, "2");
        assert_eq!(matrix.entries[1][0].text, "1");
        assert_eq!(matrix.entries[0][1].text, "-1");
        assert_eq!(matrix.entries[1][1].text, "1.50");
    }

    #[test]
    fn matrix_vector_flow_reports_result() {
        let matrix = Mat2::from_cols(vec2(1.4, 0.35), vec2(-0.45, 1.15));
        let flow = MatrixVectorFlow::new(matrix, vec2(1.6, 1.1));

        assert!((flow.result().x - 1.745).abs() < 1.0e-5);
        assert!((flow.result().y - 1.825).abs() < 1.0e-5);
        assert_eq!(flow.row_dot_products(), flow.result_values());
    }

    #[test]
    fn matrix_vector_flow_supports_rectangular_matrices() {
        let flow = MatrixVectorFlow::try_from_rows(
            vec![vec![1.0, 2.0, 0.5], vec![0.0, -1.0, 3.0]],
            vec![2.0, -1.0, 4.0],
        )
        .unwrap();

        assert_eq!(flow.row_count(), 2);
        assert_eq!(flow.column_count(), 3);
        assert_eq!(flow.result_values(), vec![2.0, 13.0]);
        assert_eq!(flow.result(), vec2(2.0, 13.0));
    }

    #[test]
    fn matrix_vector_flow_rejects_incompatible_input() {
        let err = MatrixVectorFlow::try_from_rows(vec![vec![1.0, 2.0]], vec![1.0])
            .expect_err("input length mismatch should fail");

        assert!(err.contains("input length 1"));
    }

    #[test]
    fn determinant_area_view_reports_area_and_orientation() {
        let positive = DeterminantAreaView::new(Mat2::from_cols(vec2(2.0, 0.0), vec2(0.0, 3.0)));
        let negative = DeterminantAreaView::new(Mat2::from_cols(vec2(0.0, 1.0), vec2(1.0, 0.0)));
        let collapsed = DeterminantAreaView::new(Mat2::from_cols(vec2(1.0, 1.0), vec2(2.0, 2.0)));

        assert_eq!(positive.determinant(), 6.0);
        assert_eq!(positive.area_scale(), 6.0);
        assert_eq!(positive.orientation_sign(), 1.0);
        assert_eq!(negative.determinant(), -1.0);
        assert_eq!(negative.orientation_sign(), -1.0);
        assert!(collapsed.is_collapsed());
    }

    #[test]
    fn matrix_composition_order_is_explicit() {
        let scale_x = Mat2::from_cols(vec2(2.0, 0.0), vec2(0.0, 1.0));
        let shear = Mat2::from_cols(vec2(1.0, 0.0), vec2(1.0, 1.0));
        let composed = shear * scale_x;
        let reversed = scale_x * shear;

        assert_ne!(composed, reversed);
        assert_eq!(composed * vec2(1.0, 1.0), vec2(3.0, 1.0));
        assert_eq!(reversed * vec2(1.0, 1.0), vec2(4.0, 1.0));
    }
}
