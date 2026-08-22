use glam::{Vec2, Vec3, Vec4, vec2, vec3};

use crate::frontend::collection::common::TensorNormalization;
use crate::frontend::collection::common::tensor::TensorSnapshot;
use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Mesh, Project, ProjectionCtx, RenderPrimitive};
use crate::validation::ValidationError;

const COMPONENT: &str = "NormalizationView";
const MAX_GROUPS: usize = 12;
const MAX_FEATURES: usize = 12;

/// Per-group statistics used by one normalization operation.
#[derive(Debug, Clone, PartialEq)]
pub struct NormalizationStats {
    pub group_element_id: String,
    pub group_label: String,
    pub mean: f32,
    pub variance: f32,
    pub rms: f32,
    pub divisor: f32,
}

/// A computed before/after view of LayerNorm or RMSNorm along one named axis.
#[derive(Debug, Clone)]
pub struct NormalizationView {
    pub input: TensorSnapshot,
    pub normalized: TensorSnapshot,
    pub axis_id: String,
    pub normalization: TensorNormalization,
    pub epsilon: f32,
    pub stats: Vec<NormalizationStats>,
    pub cell_size: Vec2,
    pub panel_gap: f32,
    pub padding: f32,
    pub text_color: Vec4,
    pub zero_color: Vec4,
    pub negative_color: Vec4,
    pub input_color: Vec4,
    pub output_color: Vec4,
    pub grid_color: Vec4,
}

impl NormalizationView {
    pub fn try_new(
        input: TensorSnapshot,
        output_id: impl Into<String>,
        axis_id: impl Into<String>,
        normalization: TensorNormalization,
        epsilon: f32,
    ) -> Result<Self, ValidationError> {
        let axis_id = axis_id.into();
        input.validate()?;
        if input.rank() != 2 {
            return Err(ValidationError::RankMismatch {
                component: COMPONENT,
                field: "input",
                expected: 2,
                actual: input.rank(),
            });
        }
        let normalized = input.try_normalized(output_id, &axis_id, normalization, epsilon)?;
        let stats = compute_stats(&input, &axis_id, normalization, epsilon)?;
        let view = Self {
            input,
            normalized,
            axis_id,
            normalization,
            epsilon,
            stats,
            cell_size: vec2(0.54, 0.46),
            panel_gap: 2.35,
            padding: 0.34,
            text_color: Vec4::new(0.94, 0.97, 1.0, 1.0),
            zero_color: Vec4::new(0.14, 0.17, 0.21, 1.0),
            negative_color: Vec4::new(0.90, 0.36, 0.48, 1.0),
            input_color: Vec4::new(0.45, 0.61, 0.88, 1.0),
            output_color: Vec4::new(0.33, 0.79, 0.68, 1.0),
            grid_color: Vec4::new(0.44, 0.49, 0.55, 1.0),
        };
        view.validate()?;
        Ok(view)
    }

    pub fn group_count(&self) -> usize {
        self.input.shape[self.group_axis_index().unwrap_or(0)]
    }

    pub fn feature_count(&self) -> usize {
        self.input.shape[self.normalization_axis_index().unwrap_or(1)]
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        self.input.validate()?;
        self.normalized.validate()?;
        if self.input.rank() != 2 || self.normalized.rank() != 2 {
            return Err(ValidationError::RankMismatch {
                component: COMPONENT,
                field: "normalization tensors",
                expected: 2,
                actual: self.input.rank().max(self.normalized.rank()),
            });
        }
        self.normalization_axis_index()?;
        let expected = self.input.try_normalized(
            self.normalized.id.clone(),
            &self.axis_id,
            self.normalization,
            self.epsilon,
        )?;
        if expected.shape != self.normalized.shape || expected.axes != self.normalized.axes {
            return Err(ValidationError::Incompatible {
                component: COMPONENT,
                field: "normalized tensor semantics",
                reason: "output shape and axes must match the computed normalization".to_string(),
            });
        }
        if expected
            .values
            .iter()
            .zip(&self.normalized.values)
            .any(|(expected, actual)| (expected - actual).abs() > 1e-5)
        {
            return Err(ValidationError::Incompatible {
                component: COMPONENT,
                field: "normalized tensor values",
                reason: "output values do not match the declared normalization".to_string(),
            });
        }
        let expected_stats =
            compute_stats(&self.input, &self.axis_id, self.normalization, self.epsilon)?;
        if self.stats != expected_stats {
            return Err(ValidationError::Incompatible {
                component: COMPONENT,
                field: "normalization statistics",
                reason: "statistics do not match the input tensor".to_string(),
            });
        }
        if self.group_count() > MAX_GROUPS || self.feature_count() > MAX_FEATURES {
            return Err(ValidationError::Incompatible {
                component: COMPONENT,
                field: "focused normalization size",
                reason: format!(
                    "view supports at most {MAX_GROUPS} groups by {MAX_FEATURES} normalized features; slice larger tensors explicitly"
                ),
            });
        }
        for (field, value) in [
            ("cell width", self.cell_size.x),
            ("cell height", self.cell_size.y),
            ("panel gap", self.panel_gap),
            ("padding", self.padding),
        ] {
            if !value.is_finite() {
                return Err(ValidationError::NonFinite {
                    component: COMPONENT,
                    field,
                    value,
                });
            }
            if value <= 0.0 {
                return Err(ValidationError::NonPositive {
                    component: COMPONENT,
                    field,
                    value,
                });
            }
        }
        Ok(())
    }

    fn normalization_axis_index(&self) -> Result<usize, ValidationError> {
        self.input
            .axes
            .iter()
            .position(|axis| axis.id == self.axis_id)
            .ok_or_else(|| ValidationError::UnknownIdentifier {
                component: COMPONENT,
                field: "normalization axis",
                value: self.axis_id.clone(),
            })
    }

    fn group_axis_index(&self) -> Result<usize, ValidationError> {
        Ok(1 - self.normalization_axis_index()?)
    }

    fn value(&self, snapshot: &TensorSnapshot, group: usize, feature: usize) -> f32 {
        if self.normalization_axis_index().expect("validated axis") == 1 {
            snapshot.value(&[group, feature]).expect("validated index")
        } else {
            snapshot.value(&[feature, group]).expect("validated index")
        }
    }

    fn scale_limit(&self) -> f32 {
        self.input
            .values
            .iter()
            .chain(&self.normalized.values)
            .map(|value| value.abs())
            .fold(0.0, f32::max)
            .max(f32::EPSILON)
    }

    fn dimensions(&self) -> Vec2 {
        let matrix_width = self.feature_count() as f32 * self.cell_size.x;
        vec2(
            self.padding * 2.0 + 1.25 + matrix_width * 2.0 + self.panel_gap,
            self.padding * 2.0 + 0.86 + self.group_count() as f32 * self.cell_size.y,
        )
    }
}

impl Project for NormalizationView {
    fn project(&self, ctx: &mut ProjectionCtx) {
        if let Err(error) = self.validate() {
            ctx.report(error);
            return;
        }
        let size = self.dimensions();
        emit_rect(
            ctx,
            size.x,
            size.y,
            Vec4::new(0.055, 0.068, 0.085, 0.98),
            Vec3::ZERO,
        );
        let left = -size.x * 0.5 + self.padding;
        let top = size.y * 0.5 - self.padding;
        let operation_label = match self.normalization {
            TensorNormalization::LayerNorm => "LAYER NORM",
            TensorNormalization::RmsNorm => "RMS NORM",
        };
        emit_text(
            ctx,
            operation_label,
            0.22,
            self.text_color,
            vec3(left + 0.85, top - 0.13, 0.05),
        );
        emit_text(
            ctx,
            &format!("axis {}   epsilon {:.0e}", self.axis_id, self.epsilon),
            0.14,
            self.grid_color,
            vec3(size.x * 0.5 - self.padding - 1.05, top - 0.13, 0.05),
        );

        let label_width = 1.25;
        let matrix_width = self.feature_count() as f32 * self.cell_size.x;
        let input_left = left + label_width;
        let output_left = input_left + matrix_width + self.panel_gap;
        let stats_x = input_left + matrix_width + self.panel_gap * 0.5;
        emit_text(
            ctx,
            "INPUT",
            0.16,
            self.input_color,
            vec3(input_left + matrix_width * 0.5, top - 0.53, 0.05),
        );
        emit_text(
            ctx,
            "NORMALIZED",
            0.16,
            self.output_color,
            vec3(output_left + matrix_width * 0.5, top - 0.53, 0.05),
        );

        let group_axis = &self.input.axes[self.group_axis_index().unwrap()];
        let row_top = top - 0.86;
        let scale = self.scale_limit();
        for group in 0..self.group_count() {
            let row_y = row_top - group as f32 * self.cell_size.y - self.cell_size.y * 0.5;
            emit_text(
                ctx,
                &group_axis.element_labels[group],
                0.14,
                self.text_color,
                vec3(left + 0.52, row_y, 0.05),
            );
            let stat = &self.stats[group];
            let stat_label = match self.normalization {
                TensorNormalization::LayerNorm => {
                    format!("mu {:+.2}  sigma {:.2}", stat.mean, stat.divisor)
                }
                TensorNormalization::RmsNorm => format!("rms {:.2}", stat.rms),
            };
            emit_text(
                ctx,
                &stat_label,
                0.12,
                self.grid_color,
                vec3(stats_x, row_y, 0.05),
            );
            for feature in 0..self.feature_count() {
                for (snapshot, panel_left, positive) in [
                    (&self.input, input_left, self.input_color),
                    (&self.normalized, output_left, self.output_color),
                ] {
                    let value = self.value(snapshot, group, feature);
                    let x = panel_left + feature as f32 * self.cell_size.x + self.cell_size.x * 0.5;
                    emit_rect(
                        ctx,
                        self.cell_size.x * 0.94,
                        self.cell_size.y * 0.88,
                        value_color(value, scale, self.negative_color, self.zero_color, positive),
                        vec3(x, row_y, 0.03),
                    );
                    emit_text(
                        ctx,
                        &format!("{value:+.1}"),
                        0.11,
                        self.text_color,
                        vec3(x, row_y, 0.06),
                    );
                }
            }
        }
    }
}

impl Bounded for NormalizationView {
    fn local_bounds(&self) -> Bounds {
        if self.validate().is_err() {
            return Bounds::from_center_size(Vec2::ZERO, Vec2::splat(0.1));
        }
        Bounds::from_center_size(Vec2::ZERO, self.dimensions())
    }
}

fn compute_stats(
    input: &TensorSnapshot,
    axis_id: &str,
    normalization: TensorNormalization,
    epsilon: f32,
) -> Result<Vec<NormalizationStats>, ValidationError> {
    let Some(axis) = input.axes.iter().position(|axis| axis.id == axis_id) else {
        return Err(ValidationError::UnknownIdentifier {
            component: COMPONENT,
            field: "normalization axis",
            value: axis_id.to_string(),
        });
    };
    if input.rank() != 2 {
        return Err(ValidationError::RankMismatch {
            component: COMPONENT,
            field: "statistics input",
            expected: 2,
            actual: input.rank(),
        });
    }
    let group_axis = 1 - axis;
    let mut stats = Vec::with_capacity(input.shape[group_axis]);
    for group in 0..input.shape[group_axis] {
        let values: Vec<f32> = (0..input.shape[axis])
            .map(|feature| {
                if axis == 1 {
                    input.value(&[group, feature]).unwrap()
                } else {
                    input.value(&[feature, group]).unwrap()
                }
            })
            .collect();
        let mean = values.iter().sum::<f32>() / values.len() as f32;
        let variance = values
            .iter()
            .map(|value| (value - mean).powi(2))
            .sum::<f32>()
            / values.len() as f32;
        let rms =
            (values.iter().map(|value| value.powi(2)).sum::<f32>() / values.len() as f32).sqrt();
        let divisor = match normalization {
            TensorNormalization::LayerNorm => (variance + epsilon).sqrt(),
            TensorNormalization::RmsNorm => (rms.powi(2) + epsilon).sqrt(),
        };
        stats.push(NormalizationStats {
            group_element_id: input.axes[group_axis].element_ids[group].clone(),
            group_label: input.axes[group_axis].element_labels[group].clone(),
            mean,
            variance,
            rms,
            divisor,
        });
    }
    Ok(stats)
}

fn value_color(value: f32, limit: f32, negative: Vec4, zero: Vec4, positive: Vec4) -> Vec4 {
    let normalized = (value / limit).clamp(-1.0, 1.0);
    if normalized < 0.0 {
        zero.lerp(negative, -normalized)
    } else {
        zero.lerp(positive, normalized)
    }
}

fn emit_rect(ctx: &mut ProjectionCtx, width: f32, height: f32, color: Vec4, center: Vec3) {
    ctx.emit(RenderPrimitive::Mesh(
        Mesh::rectangle(width, height, color).translated(center),
    ));
}

fn emit_text(ctx: &mut ProjectionCtx, content: &str, height: f32, color: Vec4, offset: Vec3) {
    ctx.emit(RenderPrimitive::Text {
        content: content.to_string(),
        height,
        color,
        font_name: None,
        offset,
        rotation: 0.0,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::scene::SharedProps;
    use crate::frontend::collection::ai::TensorAxis;

    fn input() -> TensorSnapshot {
        TensorSnapshot::try_new(
            "residual.input",
            vec![3, 4],
            vec![1.0, 2.0, 4.0, 5.0, -2.0, 0.0, 2.0, 4.0, 3.0, 3.5, 4.5, 7.0],
            vec![
                TensorAxis::with_elements(
                    "token",
                    "Tokens",
                    [("t0", "The"), ("t1", "model"), ("t2", "learns")],
                ),
                TensorAxis::new("feature", "Features", vec!["f0", "f1", "f2", "f3"]),
            ],
        )
        .unwrap()
    }

    #[test]
    fn computes_output_and_per_token_statistics() {
        let view = NormalizationView::try_new(
            input(),
            "residual.normalized",
            "feature",
            TensorNormalization::LayerNorm,
            1e-5,
        )
        .unwrap();
        assert_eq!(view.group_count(), 3);
        assert_eq!(view.feature_count(), 4);
        assert_eq!(view.stats[1].group_element_id, "t1");
        for group in 0..3 {
            let sum = (0..4)
                .map(|feature| view.value(&view.normalized, group, feature))
                .sum::<f32>();
            assert!(sum.abs() < 1e-5);
        }
    }

    #[test]
    fn rms_norm_preserves_nonzero_mean_and_normalizes_square_mean() {
        let view = NormalizationView::try_new(
            input(),
            "residual.rms",
            "feature",
            TensorNormalization::RmsNorm,
            1e-5,
        )
        .unwrap();
        let values: Vec<f32> = (0..4)
            .map(|feature| view.value(&view.normalized, 0, feature))
            .collect();
        let square_mean = values.iter().map(|value| value.powi(2)).sum::<f32>() / 4.0;
        assert!((square_mean - 1.0).abs() < 1e-5);
        assert!(values.iter().sum::<f32>() > 0.0);
    }

    #[test]
    fn mutated_output_is_rejected_before_projection() {
        let mut view = NormalizationView::try_new(
            input(),
            "residual.normalized",
            "feature",
            TensorNormalization::LayerNorm,
            1e-5,
        )
        .unwrap();
        view.normalized.values[0] += 0.5;
        let mut ctx = ProjectionCtx::new(SharedProps::default());
        view.project(&mut ctx);
        assert!(ctx.primitives.is_empty());
        assert!(matches!(
            ctx.diagnostics[0],
            ValidationError::Incompatible {
                field: "normalized tensor values",
                ..
            }
        ));
    }
}
