use glam::{Vec2, Vec3, Vec4, vec2, vec3};

use crate::frontend::collection::common::tensor::{TensorAxis, TensorSnapshot};
use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Mesh, Project, ProjectionCtx, RenderPrimitive};
use crate::validation::ValidationError;

const COMPONENT: &str = "NextTokenDistribution";
const MAX_CANDIDATES: usize = 32;

/// Deterministic sampling controls applied to a rank-1 logit tensor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NextTokenSampling {
    pub temperature: f32,
    pub top_k: Option<usize>,
    pub top_p: Option<f32>,
    pub unit_sample: f32,
}

impl NextTokenSampling {
    pub fn new(unit_sample: f32) -> Self {
        Self {
            temperature: 1.0,
            top_k: None,
            top_p: None,
            unit_sample,
        }
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature;
        self
    }

    pub fn with_top_k(mut self, top_k: usize) -> Self {
        self.top_k = Some(top_k);
        self
    }

    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }
}

/// One vocabulary entry before and after sampling filters are applied.
#[derive(Debug, Clone, PartialEq)]
pub struct NextTokenCandidate {
    pub element_id: String,
    pub token: String,
    pub logit: f32,
    pub model_probability: f32,
    pub sampling_probability: f32,
    pub retained: bool,
    pub selected: bool,
}

/// A computed teaching view for one next-token sampling decision.
#[derive(Debug, Clone)]
pub struct NextTokenDistribution {
    pub tensor_id: String,
    pub axis_id: String,
    pub candidates: Vec<NextTokenCandidate>,
    pub sampling: NextTokenSampling,
    pub width: f32,
    pub row_height: f32,
    pub row_gap: f32,
    pub padding: f32,
    pub text_color: Vec4,
    pub panel_color: Vec4,
    pub track_color: Vec4,
    pub probability_color: Vec4,
    pub filtered_color: Vec4,
    pub selected_color: Vec4,
}

impl NextTokenDistribution {
    /// Computes a distribution from a focused, rank-1 vocabulary logit tensor.
    ///
    /// Large full-vocabulary tensors should first be sliced to authored candidates. Murali does
    /// not silently aggregate or discard vocabulary entries in this view.
    pub fn try_from_logits(
        logits: &TensorSnapshot,
        axis_id: &str,
        sampling: NextTokenSampling,
    ) -> Result<Self, ValidationError> {
        logits.validate()?;
        if logits.rank() != 1 {
            return Err(ValidationError::RankMismatch {
                component: COMPONENT,
                field: "logits",
                expected: 1,
                actual: logits.rank(),
            });
        }
        if logits.axes[0].id != axis_id {
            return Err(ValidationError::UnknownIdentifier {
                component: COMPONENT,
                field: "vocabulary axis",
                value: axis_id.to_string(),
            });
        }
        validate_sampling(sampling, logits.shape[0])?;
        if logits.shape[0] > MAX_CANDIDATES {
            return Err(ValidationError::Incompatible {
                component: COMPONENT,
                field: "candidate count",
                reason: format!(
                    "{} candidates exceed the focused-view limit of {MAX_CANDIDATES}; slice the vocabulary explicitly",
                    logits.shape[0]
                ),
            });
        }

        let model_probabilities = logits
            .try_scaled(sampling.temperature)?
            .try_softmax(axis_id)?;
        let mut filtered_values = model_probabilities.values.clone();
        apply_top_k(&mut filtered_values, sampling.top_k);
        renormalize(&mut filtered_values);
        apply_top_p(&mut filtered_values, sampling.top_p);
        renormalize(&mut filtered_values);

        let filtered = TensorSnapshot::try_new(
            format!("{}.sampling", logits.id),
            logits.shape.clone(),
            filtered_values.clone(),
            vec![TensorAxis::with_elements(
                logits.axes[0].id.clone(),
                logits.axes[0].label.clone(),
                logits.axes[0]
                    .element_ids
                    .iter()
                    .cloned()
                    .zip(logits.axes[0].element_labels.iter().cloned()),
            )],
        )?;
        let sample = filtered
            .try_sample_categorical(axis_id, &[sampling.unit_sample])?
            .into_iter()
            .next()
            .expect("a rank-1 tensor produces one categorical sample");
        let selected_id = &sample.element_id.coordinates[0].element_id;

        let candidates = logits.axes[0]
            .element_ids
            .iter()
            .zip(&logits.axes[0].element_labels)
            .zip(&logits.values)
            .zip(&model_probabilities.values)
            .zip(&filtered_values)
            .map(
                |((((element_id, token), &logit), &model_probability), &sampling_probability)| {
                    NextTokenCandidate {
                        element_id: element_id.clone(),
                        token: token.clone(),
                        logit,
                        model_probability,
                        sampling_probability,
                        retained: sampling_probability > 0.0,
                        selected: element_id == selected_id,
                    }
                },
            )
            .collect();

        let distribution = Self {
            tensor_id: logits.id.clone(),
            axis_id: axis_id.to_string(),
            candidates,
            sampling,
            width: 8.8,
            row_height: 0.48,
            row_gap: 0.08,
            padding: 0.34,
            text_color: Vec4::new(0.94, 0.97, 1.0, 1.0),
            panel_color: Vec4::new(0.055, 0.068, 0.085, 0.98),
            track_color: Vec4::new(0.14, 0.17, 0.21, 1.0),
            probability_color: Vec4::new(0.35, 0.77, 0.87, 1.0),
            filtered_color: Vec4::new(0.31, 0.33, 0.37, 1.0),
            selected_color: Vec4::new(0.96, 0.72, 0.35, 1.0),
        };
        distribution.validate()?;
        Ok(distribution)
    }

    pub fn selected(&self) -> &NextTokenCandidate {
        self.candidates
            .iter()
            .find(|candidate| candidate.selected)
            .expect("validated distributions have one selected candidate")
    }

    pub fn retained_count(&self) -> usize {
        self.candidates
            .iter()
            .filter(|candidate| candidate.retained)
            .count()
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.tensor_id.trim().is_empty() {
            return Err(ValidationError::Empty {
                component: COMPONENT,
                field: "tensor id",
            });
        }
        if self.axis_id.trim().is_empty() {
            return Err(ValidationError::Empty {
                component: COMPONENT,
                field: "axis id",
            });
        }
        if self.candidates.is_empty() {
            return Err(ValidationError::Empty {
                component: COMPONENT,
                field: "candidates",
            });
        }
        if self.candidates.len() > MAX_CANDIDATES {
            return Err(ValidationError::Incompatible {
                component: COMPONENT,
                field: "candidate count",
                reason: format!("focused views support at most {MAX_CANDIDATES} candidates"),
            });
        }
        validate_sampling(self.sampling, self.candidates.len())?;
        for (field, value) in [
            ("width", self.width),
            ("row height", self.row_height),
            ("row gap", self.row_gap),
            ("padding", self.padding),
        ] {
            if !value.is_finite() {
                return Err(ValidationError::NonFinite {
                    component: COMPONENT,
                    field,
                    value,
                });
            }
        }
        for (field, value) in [("width", self.width), ("row height", self.row_height)] {
            if value <= 0.0 {
                return Err(ValidationError::NonPositive {
                    component: COMPONENT,
                    field,
                    value,
                });
            }
        }
        if self.row_gap < 0.0 || self.padding < 0.0 {
            return Err(ValidationError::Incompatible {
                component: COMPONENT,
                field: "spacing",
                reason: "row gap and padding must be non-negative".to_string(),
            });
        }
        if self.width <= self.padding * 2.0 + 4.0 {
            return Err(ValidationError::Incompatible {
                component: COMPONENT,
                field: "layout width",
                reason: "width must leave room for labels and probability bars".to_string(),
            });
        }

        let selected = self
            .candidates
            .iter()
            .filter(|candidate| candidate.selected)
            .count();
        if selected != 1 {
            return Err(ValidationError::LengthMismatch {
                component: COMPONENT,
                field: "selected candidates",
                expected: 1,
                actual: selected,
            });
        }
        for candidate in &self.candidates {
            if candidate.element_id.trim().is_empty() || candidate.token.is_empty() {
                return Err(ValidationError::Empty {
                    component: COMPONENT,
                    field: "candidate identity",
                });
            }
            for (field, value) in [
                ("candidate logit", candidate.logit),
                ("model probability", candidate.model_probability),
                ("sampling probability", candidate.sampling_probability),
            ] {
                if !value.is_finite() {
                    return Err(ValidationError::NonFinite {
                        component: COMPONENT,
                        field,
                        value,
                    });
                }
            }
            if candidate.selected && !candidate.retained {
                return Err(ValidationError::Incompatible {
                    component: COMPONENT,
                    field: "selected candidate",
                    reason: "the selected candidate must survive sampling filters".to_string(),
                });
            }
        }
        Ok(())
    }

    fn height(&self) -> f32 {
        let rows = self.candidates.len() as f32 * self.row_height
            + self.candidates.len().saturating_sub(1) as f32 * self.row_gap;
        self.padding * 2.0 + 0.72 + rows + 0.32
    }
}

impl Project for NextTokenDistribution {
    fn project(&self, ctx: &mut ProjectionCtx) {
        if let Err(error) = self.validate() {
            ctx.report(error);
            return;
        }

        let height = self.height();
        emit_rect(ctx, self.width, height, self.panel_color, Vec3::ZERO);
        let top = height * 0.5 - self.padding;
        emit_text(
            ctx,
            "NEXT TOKEN",
            0.22,
            self.text_color,
            vec3(-self.width * 0.5 + self.padding + 0.8, top - 0.13, 0.04),
        );
        emit_text(
            ctx,
            &settings_label(self.sampling),
            0.15,
            Vec4::new(0.70, 0.75, 0.80, 1.0),
            vec3(self.width * 0.5 - self.padding - 1.55, top - 0.13, 0.04),
        );

        let left = -self.width * 0.5 + self.padding;
        let label_x = left + 0.7;
        let logit_x = left + 1.75;
        let track_x = left + 2.25;
        let track_width = self.width - self.padding * 2.0 - 3.0;
        let mut row_y = top - 0.72 - self.row_height * 0.5;

        for candidate in &self.candidates {
            if candidate.selected {
                emit_rect(
                    ctx,
                    self.width - self.padding * 2.0,
                    self.row_height,
                    Vec4::new(0.18, 0.145, 0.09, 1.0),
                    vec3(0.0, row_y, 0.01),
                );
            }
            let label_color = if candidate.retained {
                self.text_color
            } else {
                self.filtered_color
            };
            let selected_prefix = if candidate.selected { "> " } else { "" };
            emit_text(
                ctx,
                &format!("{selected_prefix}{}", candidate.token),
                0.17,
                if candidate.selected {
                    self.selected_color
                } else {
                    label_color
                },
                vec3(label_x, row_y, 0.05),
            );
            emit_text(
                ctx,
                &format!("{:+.2}", candidate.logit),
                0.13,
                label_color,
                vec3(logit_x, row_y, 0.05),
            );
            emit_rect(
                ctx,
                track_width,
                0.18,
                self.track_color,
                vec3(track_x + track_width * 0.5, row_y, 0.02),
            );

            let bar_width = track_width * candidate.sampling_probability;
            if bar_width > 0.0 {
                emit_rect(
                    ctx,
                    bar_width,
                    0.18,
                    if candidate.selected {
                        self.selected_color
                    } else {
                        self.probability_color
                    },
                    vec3(track_x + bar_width * 0.5, row_y, 0.04),
                );
            }
            let probability_label = if candidate.retained {
                format!(
                    "{:>5.1}%  model {:>5.1}%",
                    candidate.sampling_probability * 100.0,
                    candidate.model_probability * 100.0
                )
            } else {
                format!(
                    "FILTERED  model {:>5.1}%",
                    candidate.model_probability * 100.0
                )
            };
            emit_text(
                ctx,
                &probability_label,
                0.12,
                label_color,
                vec3(track_x + track_width - 0.72, row_y, 0.06),
            );
            row_y -= self.row_height + self.row_gap;
        }
    }
}

impl Bounded for NextTokenDistribution {
    fn local_bounds(&self) -> Bounds {
        if self.validate().is_err() {
            return Bounds::from_center_size(Vec2::ZERO, Vec2::splat(0.1));
        }
        Bounds::from_center_size(Vec2::ZERO, vec2(self.width, self.height()))
    }
}

fn validate_sampling(
    sampling: NextTokenSampling,
    candidate_count: usize,
) -> Result<(), ValidationError> {
    if !sampling.temperature.is_finite() {
        return Err(ValidationError::NonFinite {
            component: COMPONENT,
            field: "temperature",
            value: sampling.temperature,
        });
    }
    if sampling.temperature <= 0.0 {
        return Err(ValidationError::NonPositive {
            component: COMPONENT,
            field: "temperature",
            value: sampling.temperature,
        });
    }
    if let Some(top_k) = sampling.top_k {
        if top_k == 0 {
            return Err(ValidationError::CountTooSmall {
                component: COMPONENT,
                field: "top-k",
                minimum: 1,
                actual: top_k,
            });
        }
        if top_k > candidate_count {
            return Err(ValidationError::Incompatible {
                component: COMPONENT,
                field: "top-k",
                reason: format!("top-k {top_k} exceeds the {candidate_count} available candidates"),
            });
        }
    }
    if let Some(top_p) = sampling.top_p {
        if !top_p.is_finite() {
            return Err(ValidationError::NonFinite {
                component: COMPONENT,
                field: "top-p",
                value: top_p,
            });
        }
        if !(0.0..=1.0).contains(&top_p) || top_p == 0.0 {
            return Err(ValidationError::OutOfRange {
                component: COMPONENT,
                field: "top-p",
                minimum: f32::EPSILON,
                maximum: 1.0,
                value: top_p,
            });
        }
    }
    if !sampling.unit_sample.is_finite() {
        return Err(ValidationError::NonFinite {
            component: COMPONENT,
            field: "unit sample",
            value: sampling.unit_sample,
        });
    }
    if !(0.0..1.0).contains(&sampling.unit_sample) {
        return Err(ValidationError::OutOfRange {
            component: COMPONENT,
            field: "unit sample",
            minimum: 0.0,
            maximum: 1.0,
            value: sampling.unit_sample,
        });
    }
    Ok(())
}

fn apply_top_k(values: &mut [f32], top_k: Option<usize>) {
    let Some(top_k) = top_k else {
        return;
    };
    let mut indices: Vec<usize> = (0..values.len()).collect();
    indices.sort_by(|&a, &b| values[b].total_cmp(&values[a]).then(a.cmp(&b)));
    for &index in indices.iter().skip(top_k) {
        values[index] = 0.0;
    }
}

fn apply_top_p(values: &mut [f32], top_p: Option<f32>) {
    let Some(top_p) = top_p else {
        return;
    };
    let mut indices: Vec<usize> = (0..values.len())
        .filter(|&index| values[index] > 0.0)
        .collect();
    indices.sort_by(|&a, &b| values[b].total_cmp(&values[a]).then(a.cmp(&b)));
    let mut cumulative = 0.0;
    let mut keep = 0;
    for &index in &indices {
        cumulative += values[index];
        keep += 1;
        if cumulative >= top_p {
            break;
        }
    }
    for &index in indices.iter().skip(keep) {
        values[index] = 0.0;
    }
}

fn renormalize(values: &mut [f32]) {
    let sum = values.iter().sum::<f32>();
    if sum > 0.0 {
        for value in values {
            *value /= sum;
        }
    }
}

fn settings_label(sampling: NextTokenSampling) -> String {
    let top_k = sampling
        .top_k
        .map_or_else(|| "off".to_string(), |value| value.to_string());
    let top_p = sampling
        .top_p
        .map_or_else(|| "off".to_string(), |value| format!("{value:.2}"));
    format!(
        "T {:.2}   TOP-K {top_k}   TOP-P {top_p}   u {:.2}",
        sampling.temperature, sampling.unit_sample
    )
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

    fn logits() -> TensorSnapshot {
        TensorSnapshot::try_new(
            "decoder.logits",
            vec![5],
            vec![2.4, 1.8, 0.9, 0.2, -0.4],
            vec![TensorAxis::with_elements(
                "vocabulary",
                "Vocabulary",
                [
                    ("token.blue", "blue"),
                    ("token.clear", "clear"),
                    ("token.bright", "bright"),
                    ("token.dark", "dark"),
                    ("token.warm", "warm"),
                ],
            )],
        )
        .unwrap()
    }

    #[test]
    fn temperature_filters_and_sample_are_computed_deterministically() {
        let distribution = NextTokenDistribution::try_from_logits(
            &logits(),
            "vocabulary",
            NextTokenSampling::new(0.72)
                .with_temperature(0.8)
                .with_top_k(4)
                .with_top_p(0.88),
        )
        .unwrap();

        assert_eq!(distribution.retained_count(), 3);
        assert_eq!(distribution.selected().element_id, "token.clear");
        assert!(
            (distribution
                .candidates
                .iter()
                .map(|c| c.sampling_probability)
                .sum::<f32>()
                - 1.0)
                .abs()
                < 1e-6
        );
        assert!(distribution.candidates[4].model_probability > 0.0);
        assert_eq!(distribution.candidates[4].sampling_probability, 0.0);
    }

    #[test]
    fn rejects_invalid_controls_and_large_unsliced_vocabularies() {
        assert!(matches!(
            NextTokenDistribution::try_from_logits(
                &logits(),
                "vocabulary",
                NextTokenSampling::new(0.5).with_temperature(0.0),
            ),
            Err(ValidationError::NonPositive {
                field: "temperature",
                ..
            })
        ));

        let labels: Vec<_> = (0..33).map(|index| format!("token.{index}")).collect();
        let large = TensorSnapshot::try_new(
            "large",
            vec![33],
            vec![0.0; 33],
            vec![TensorAxis::new("vocabulary", "Vocabulary", labels)],
        )
        .unwrap();
        assert!(matches!(
            NextTokenDistribution::try_from_logits(
                &large,
                "vocabulary",
                NextTokenSampling::new(0.5),
            ),
            Err(ValidationError::Incompatible {
                field: "candidate count",
                ..
            })
        ));
    }

    #[test]
    fn invalid_direct_mutation_reports_a_projection_diagnostic() {
        let mut distribution = NextTokenDistribution::try_from_logits(
            &logits(),
            "vocabulary",
            NextTokenSampling::new(0.5),
        )
        .unwrap();
        for candidate in &mut distribution.candidates {
            candidate.selected = false;
        }
        let mut ctx = ProjectionCtx::new(SharedProps::default());
        distribution.project(&mut ctx);
        assert!(ctx.primitives.is_empty());
        assert!(matches!(
            ctx.diagnostics[0],
            ValidationError::LengthMismatch {
                field: "selected candidates",
                ..
            }
        ));
    }
}
