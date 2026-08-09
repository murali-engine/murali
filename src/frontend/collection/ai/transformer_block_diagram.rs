use std::collections::HashSet;

use glam::{Vec2, Vec3, Vec4, vec2};
use serde::{Deserialize, Serialize};

use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};
use crate::validation::ValidationError;

const COMPONENT: &str = "TransformerBlockDiagram";

/// Reserved source ID used when a residual connection begins at the block input.
pub const TRANSFORMER_INPUT_ID: &str = "input";

/// Mathematical role of one stage in a transformer composition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransformerStageKind {
    Normalization,
    SelfAttention,
    CrossAttention,
    FeedForward,
    ResidualAdd,
    Projection,
    Custom,
}

/// One stable, data-aware stage in a transformer block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransformerStage {
    pub id: String,
    pub label: String,
    pub kind: TransformerStageKind,
    pub input_tensor_ids: Vec<String>,
    pub output_tensor_ids: Vec<String>,
    pub residual_from: Option<String>,
}

/// Transient render state for a deterministic focus transition between semantic stages.
#[derive(Debug, Clone, PartialEq)]
pub struct TransformerStageFocusFrame {
    pub source_stage_id: Option<String>,
    pub target_stage_id: Option<String>,
    pub progress: f32,
}

impl TransformerStage {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        kind: TransformerStageKind,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind,
            input_tensor_ids: Vec::new(),
            output_tensor_ids: Vec::new(),
            residual_from: None,
        }
    }

    pub fn with_tensors<I, O, IS, OS>(mut self, inputs: I, outputs: O) -> Self
    where
        I: IntoIterator<Item = IS>,
        O: IntoIterator<Item = OS>,
        IS: Into<String>,
        OS: Into<String>,
    {
        self.input_tensor_ids = inputs.into_iter().map(Into::into).collect();
        self.output_tensor_ids = outputs.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_residual_from(mut self, source_id: impl Into<String>) -> Self {
        self.residual_from = Some(source_id.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct TransformerBlockDiagram {
    pub stages: Vec<TransformerStage>,
    pub input_label: String,
    pub output_label: String,
    pub active_stage_id: Option<String>,
    #[doc(hidden)]
    pub focus_transition: Option<TransformerStageFocusFrame>,
    pub width: f32,
    pub block_height: f32,
    pub gap: f32,
    pub line_thickness: f32,
    pub frame_color: Vec4,
    pub accent_color: Vec4,
    pub text_color: Vec4,
    pub residual_color: Vec4,
    pub inactive_opacity: f32,
}

impl TransformerBlockDiagram {
    /// Creates a semantic pre-norm transformer encoder block.
    pub fn new() -> Self {
        Self::encoder()
    }

    /// Creates the standard pre-norm encoder composition with stable stage and tensor IDs.
    pub fn encoder() -> Self {
        Self::base(vec![
            TransformerStage::new(
                "attention_norm",
                "Layer Norm",
                TransformerStageKind::Normalization,
            )
            .with_tensors(["residual.input"], ["attention.normalized"]),
            TransformerStage::new(
                "self_attention",
                "Multi-Head Self-Attention",
                TransformerStageKind::SelfAttention,
            )
            .with_tensors(
                ["attention.normalized"],
                [
                    "attention.q",
                    "attention.k",
                    "attention.v",
                    "attention.output",
                ],
            ),
            TransformerStage::new(
                "attention_residual",
                "Residual Add",
                TransformerStageKind::ResidualAdd,
            )
            .with_tensors(
                ["residual.input", "attention.output"],
                ["residual.attention"],
            )
            .with_residual_from(TRANSFORMER_INPUT_ID),
            TransformerStage::new(
                "mlp_norm",
                "Layer Norm",
                TransformerStageKind::Normalization,
            )
            .with_tensors(["residual.attention"], ["mlp.normalized"]),
            TransformerStage::new("mlp", "MLP", TransformerStageKind::FeedForward)
                .with_tensors(["mlp.normalized"], ["mlp.output"]),
            TransformerStage::new(
                "mlp_residual",
                "Residual Add",
                TransformerStageKind::ResidualAdd,
            )
            .with_tensors(["residual.attention", "mlp.output"], ["residual.output"])
            .with_residual_from("attention_residual"),
        ])
    }

    /// Creates a decoder composition with masked self-attention and cross-attention stages.
    pub fn decoder() -> Self {
        Self::base(vec![
            TransformerStage::new(
                "self_attention_norm",
                "Layer Norm",
                TransformerStageKind::Normalization,
            ),
            TransformerStage::new(
                "masked_self_attention",
                "Masked Self-Attention",
                TransformerStageKind::SelfAttention,
            ),
            TransformerStage::new(
                "self_attention_residual",
                "Residual Add",
                TransformerStageKind::ResidualAdd,
            )
            .with_residual_from(TRANSFORMER_INPUT_ID),
            TransformerStage::new(
                "cross_attention_norm",
                "Layer Norm",
                TransformerStageKind::Normalization,
            ),
            TransformerStage::new(
                "cross_attention",
                "Cross-Attention",
                TransformerStageKind::CrossAttention,
            ),
            TransformerStage::new(
                "cross_attention_residual",
                "Residual Add",
                TransformerStageKind::ResidualAdd,
            )
            .with_residual_from("self_attention_residual"),
            TransformerStage::new(
                "mlp_norm",
                "Layer Norm",
                TransformerStageKind::Normalization,
            ),
            TransformerStage::new("mlp", "MLP", TransformerStageKind::FeedForward),
            TransformerStage::new(
                "mlp_residual",
                "Residual Add",
                TransformerStageKind::ResidualAdd,
            )
            .with_residual_from("cross_attention_residual"),
        ])
    }

    /// Creates a custom validated transformer composition.
    pub fn try_from_stages(stages: Vec<TransformerStage>) -> Result<Self, ValidationError> {
        let diagram = Self::base(stages);
        diagram.validate()?;
        Ok(diagram)
    }

    pub fn with_active_stage(
        mut self,
        stage_id: impl Into<String>,
    ) -> Result<Self, ValidationError> {
        self.active_stage_id = Some(stage_id.into());
        self.validate()?;
        Ok(self)
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.stages.is_empty() {
            return Err(ValidationError::Empty {
                component: COMPONENT,
                field: "stages",
            });
        }
        for (field, value) in [
            ("width", self.width),
            ("block height", self.block_height),
            ("gap", self.gap),
            ("line thickness", self.line_thickness),
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
        if !self.inactive_opacity.is_finite() {
            return Err(ValidationError::NonFinite {
                component: COMPONENT,
                field: "inactive opacity",
                value: self.inactive_opacity,
            });
        }
        if !(0.0..=1.0).contains(&self.inactive_opacity) {
            return Err(ValidationError::OutOfRange {
                component: COMPONENT,
                field: "inactive opacity",
                minimum: 0.0,
                maximum: 1.0,
                value: self.inactive_opacity,
            });
        }

        let mut preceding_ids = HashSet::with_capacity(self.stages.len());
        for stage in &self.stages {
            if stage.id.trim().is_empty() {
                return Err(ValidationError::Empty {
                    component: COMPONENT,
                    field: "stage id",
                });
            }
            if stage.id == TRANSFORMER_INPUT_ID {
                return Err(ValidationError::DuplicateIdentifier {
                    component: COMPONENT,
                    field: "stages",
                    value: stage.id.clone(),
                });
            }
            if !preceding_ids.insert(stage.id.as_str()) {
                return Err(ValidationError::DuplicateIdentifier {
                    component: COMPONENT,
                    field: "stages",
                    value: stage.id.clone(),
                });
            }
            if stage.label.trim().is_empty() {
                return Err(ValidationError::Empty {
                    component: COMPONENT,
                    field: "stage label",
                });
            }
            for tensor_id in stage
                .input_tensor_ids
                .iter()
                .chain(&stage.output_tensor_ids)
            {
                if tensor_id.trim().is_empty() {
                    return Err(ValidationError::Empty {
                        component: COMPONENT,
                        field: "stage tensor id",
                    });
                }
            }
            match (&stage.kind, &stage.residual_from) {
                (TransformerStageKind::ResidualAdd, Some(source)) => {
                    if source != TRANSFORMER_INPUT_ID && !preceding_ids.contains(source.as_str()) {
                        return Err(ValidationError::UnknownIdentifier {
                            component: COMPONENT,
                            field: "residual source",
                            value: source.clone(),
                        });
                    }
                    if source == &stage.id {
                        return Err(ValidationError::Incompatible {
                            component: COMPONENT,
                            field: "residual source",
                            reason: "a residual stage cannot reference itself".to_string(),
                        });
                    }
                }
                (TransformerStageKind::ResidualAdd, None) => {
                    return Err(ValidationError::Empty {
                        component: COMPONENT,
                        field: "residual source",
                    });
                }
                (_, Some(_)) => {
                    return Err(ValidationError::Incompatible {
                        component: COMPONENT,
                        field: "residual source",
                        reason: format!("stage '{}' is not a residual-add stage", stage.id),
                    });
                }
                (_, None) => {}
            }
        }
        if let Some(active_stage_id) = &self.active_stage_id {
            if !preceding_ids.contains(active_stage_id.as_str()) {
                return Err(ValidationError::UnknownIdentifier {
                    component: COMPONENT,
                    field: "active stage",
                    value: active_stage_id.clone(),
                });
            }
        }
        Ok(())
    }

    fn base(stages: Vec<TransformerStage>) -> Self {
        Self {
            stages,
            input_label: "Input Residual Stream".to_string(),
            output_label: "Output Residual Stream".to_string(),
            active_stage_id: None,
            focus_transition: None,
            width: 3.0,
            block_height: 0.5,
            gap: 0.16,
            line_thickness: 0.03,
            frame_color: Vec4::new(0.86, 0.90, 0.95, 1.0),
            accent_color: Vec4::new(0.45, 0.78, 0.98, 1.0),
            text_color: Vec4::new(0.95, 0.97, 0.99, 1.0),
            residual_color: Vec4::new(0.98, 0.72, 0.35, 1.0),
            inactive_opacity: 0.35,
        }
    }

    fn total_stage_height(&self) -> f32 {
        self.stages.len() as f32 * self.block_height
            + self.stages.len().saturating_sub(1) as f32 * self.gap
    }

    fn stage_y(&self, index: usize) -> f32 {
        self.total_stage_height() * 0.5
            - self.block_height * 0.5
            - index as f32 * (self.block_height + self.gap)
    }

    fn stage_color(&self, kind: TransformerStageKind) -> Vec4 {
        match kind {
            TransformerStageKind::SelfAttention
            | TransformerStageKind::CrossAttention
            | TransformerStageKind::FeedForward
            | TransformerStageKind::Projection => self.accent_color,
            TransformerStageKind::ResidualAdd => self.residual_color,
            TransformerStageKind::Normalization | TransformerStageKind::Custom => self.frame_color,
        }
    }

    fn stage_opacity(&self, stage_id: &str) -> f32 {
        let opacity_for = |active_stage_id: Option<&str>| match active_stage_id {
            Some(active_stage_id) if active_stage_id != stage_id => self.inactive_opacity,
            _ => 1.0,
        };
        let Some(transition) = &self.focus_transition else {
            return opacity_for(self.active_stage_id.as_deref());
        };
        let source = opacity_for(transition.source_stage_id.as_deref());
        let target = opacity_for(transition.target_stage_id.as_deref());
        source + (target - source) * transition.progress.clamp(0.0, 1.0)
    }

    fn draw_box(&self, ctx: &mut ProjectionCtx, center: Vec2, size: Vec2, color: Vec4) {
        let left = center.x - size.x * 0.5;
        let right = center.x + size.x * 0.5;
        let top = center.y + size.y * 0.5;
        let bottom = center.y - size.y * 0.5;
        for (a, b) in [
            (Vec3::new(left, bottom, 0.0), Vec3::new(right, bottom, 0.0)),
            (Vec3::new(right, bottom, 0.0), Vec3::new(right, top, 0.0)),
            (Vec3::new(right, top, 0.0), Vec3::new(left, top, 0.0)),
            (Vec3::new(left, top, 0.0), Vec3::new(left, bottom, 0.0)),
        ] {
            ctx.emit(RenderPrimitive::Line {
                start: a,
                end: b,
                thickness: self.line_thickness,
                color,
                dash_length: 0.0,
                gap_length: 0.0,
                dash_offset: 0.0,
            });
        }
    }

    fn draw_residual(&self, ctx: &mut ProjectionCtx, source_y: f32, target_y: f32) {
        let inner_width = self.width * 0.86;
        let box_right = inner_width * 0.5;
        let lane_x = self.width * 0.5 + 0.3;
        for (start, end) in [
            (vec2(box_right, source_y), vec2(lane_x, source_y)),
            (vec2(lane_x, source_y), vec2(lane_x, target_y)),
            (vec2(lane_x, target_y), vec2(box_right, target_y)),
        ] {
            ctx.emit(RenderPrimitive::Line {
                start: start.extend(0.0),
                end: end.extend(0.0),
                thickness: self.line_thickness,
                color: self.residual_color,
                dash_length: 0.1,
                gap_length: 0.06,
                dash_offset: 0.0,
            });
        }
    }
}

impl Default for TransformerBlockDiagram {
    fn default() -> Self {
        Self::new()
    }
}

impl Project for TransformerBlockDiagram {
    fn project(&self, ctx: &mut ProjectionCtx) {
        if let Err(error) = self.validate() {
            ctx.report(error);
            return;
        }

        let inner_width = self.width * 0.86;
        for (index, stage) in self.stages.iter().enumerate() {
            let y = self.stage_y(index);
            let opacity = self.stage_opacity(&stage.id);
            ctx.with_opacity(opacity, |ctx| {
                self.draw_box(
                    ctx,
                    vec2(0.0, y),
                    vec2(inner_width, self.block_height),
                    self.stage_color(stage.kind),
                );
                let label_height = self.block_height * 0.32;
                let layout =
                    crate::resource::text::layout::measure_label(&stage.label, label_height, None);
                let final_height = if layout.width > inner_width * 0.92 {
                    label_height * (inner_width * 0.92 / layout.width)
                } else {
                    label_height
                };
                ctx.emit(RenderPrimitive::Text {
                    content: stage.label.clone(),
                    height: final_height,
                    color: self.text_color,
                    font_name: None,
                    offset: Vec3::new(0.0, y, 0.0),
                    rotation: 0.0,
                });
            });

            if index + 1 < self.stages.len() {
                let next_y = self.stage_y(index + 1);
                ctx.emit(RenderPrimitive::Line {
                    start: Vec3::new(0.0, y - self.block_height * 0.5, 0.0),
                    end: Vec3::new(0.0, next_y + self.block_height * 0.5, 0.0),
                    thickness: self.line_thickness,
                    color: self.frame_color,
                    dash_length: 0.0,
                    gap_length: 0.0,
                    dash_offset: 0.0,
                });
            }
        }

        let input_y = self.stage_y(0) + self.block_height * 0.5 + 0.4;
        let output_y = self.stage_y(self.stages.len() - 1) - self.block_height * 0.5 - 0.4;
        for stage in &self.stages {
            let Some(source_id) = &stage.residual_from else {
                continue;
            };
            let source_y = if source_id == TRANSFORMER_INPUT_ID {
                input_y
            } else {
                let source_index = self
                    .stages
                    .iter()
                    .position(|candidate| candidate.id == *source_id)
                    .expect("validated residual source");
                self.stage_y(source_index)
            };
            let target_index = self
                .stages
                .iter()
                .position(|candidate| candidate.id == stage.id)
                .expect("current stage exists");
            self.draw_residual(ctx, source_y, self.stage_y(target_index));
        }

        for (content, y) in [(&self.input_label, input_y), (&self.output_label, output_y)] {
            ctx.emit(RenderPrimitive::Text {
                content: content.clone(),
                height: 0.2,
                color: self.text_color,
                font_name: None,
                offset: Vec3::new(0.0, y, 0.0),
                rotation: 0.0,
            });
        }
    }
}

impl Bounded for TransformerBlockDiagram {
    fn local_bounds(&self) -> Bounds {
        let total_height = self.total_stage_height() + self.block_height * 1.8;
        let residual_width = if self
            .stages
            .iter()
            .any(|stage| stage.residual_from.is_some())
        {
            0.75
        } else {
            0.0
        };
        Bounds::from_center_size(Vec2::ZERO, vec2(self.width + residual_width, total_height))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::scene::Scene;
    use crate::engine::scene::SharedProps;
    use crate::engine::timeline::Timeline;
    use crate::frontend::animation::{Animation, Ease, TransformerStageFocus};

    #[test]
    fn encoder_has_semantic_stages_tensor_bindings_and_residuals() {
        let encoder = TransformerBlockDiagram::encoder();
        encoder.validate().unwrap();
        assert_eq!(encoder.stages.len(), 6);
        assert_eq!(encoder.stages[1].id, "self_attention");
        assert_eq!(encoder.stages[1].kind, TransformerStageKind::SelfAttention);
        assert!(
            encoder.stages[1]
                .output_tensor_ids
                .contains(&"attention.q".to_string())
        );
        assert_eq!(
            encoder.stages[5].residual_from.as_deref(),
            Some("attention_residual")
        );
    }

    #[test]
    fn custom_composition_preserves_declared_tensor_identity() {
        let diagram = TransformerBlockDiagram::try_from_stages(vec![
            TransformerStage::new(
                "projection",
                "QKV Projection",
                TransformerStageKind::Projection,
            )
            .with_tensors(["residual.input"], ["q", "k", "v"]),
            TransformerStage::new(
                "attention",
                "Attention",
                TransformerStageKind::SelfAttention,
            )
            .with_tensors(["q", "k", "v"], ["context"]),
            TransformerStage::new("add", "Residual Add", TransformerStageKind::ResidualAdd)
                .with_tensors(["residual.input", "context"], ["residual.output"])
                .with_residual_from(TRANSFORMER_INPUT_ID),
        ])
        .unwrap();

        assert_eq!(diagram.stages[0].output_tensor_ids, vec!["q", "k", "v"]);
        assert!(diagram.local_bounds().width() > diagram.width);
    }

    #[test]
    fn validation_rejects_duplicate_stages_and_forward_residuals() {
        let duplicate = TransformerStage::new("same", "One", TransformerStageKind::Custom);
        assert!(matches!(
            TransformerBlockDiagram::try_from_stages(vec![
                duplicate.clone(),
                TransformerStage::new("same", "Two", TransformerStageKind::Custom),
            ]),
            Err(ValidationError::DuplicateIdentifier { .. })
        ));

        let forward_residual =
            TransformerStage::new("add", "Residual Add", TransformerStageKind::ResidualAdd)
                .with_residual_from("future");
        assert!(matches!(
            TransformerBlockDiagram::try_from_stages(vec![
                forward_residual,
                TransformerStage::new("future", "Future", TransformerStageKind::Custom),
            ]),
            Err(ValidationError::UnknownIdentifier {
                field: "residual source",
                ..
            })
        ));
    }

    #[test]
    fn invalid_direct_mutation_reports_projection_diagnostic() {
        let mut diagram = TransformerBlockDiagram::new();
        diagram.active_stage_id = Some("missing".to_string());
        let mut ctx = ProjectionCtx::new(SharedProps::default());
        diagram.project(&mut ctx);
        assert!(ctx.primitives.is_empty());
        assert!(matches!(
            ctx.diagnostics[0],
            ValidationError::UnknownIdentifier {
                field: "active stage",
                ..
            }
        ));
    }

    #[test]
    fn stage_focus_is_deterministic_and_restores_on_reset() {
        let mut scene = Scene::new();
        let diagram_id = scene.add_tattva(TransformerBlockDiagram::encoder(), Vec3::ZERO);
        let mut focus = TransformerStageFocus::new(
            diagram_id,
            Some("self_attention".to_string()),
            Ease::Linear,
        );

        focus.validate(&scene).unwrap();
        focus.on_start(&mut scene);
        focus.apply_at(&mut scene, 0.5);
        let diagram = scene
            .get_tattva_typed::<TransformerBlockDiagram>(diagram_id)
            .unwrap();
        assert_eq!(
            diagram.state.active_stage_id.as_deref(),
            Some("self_attention")
        );
        assert_eq!(
            diagram.state.focus_transition.as_ref().unwrap().progress,
            0.5
        );

        focus.on_finish(&mut scene);
        let diagram = scene
            .get_tattva_typed::<TransformerBlockDiagram>(diagram_id)
            .unwrap();
        assert!(diagram.state.focus_transition.is_none());

        focus.reset(&mut scene);
        let diagram = scene
            .get_tattva_typed::<TransformerBlockDiagram>(diagram_id)
            .unwrap();
        assert_eq!(diagram.state.active_stage_id, None);
        assert!(diagram.state.focus_transition.is_none());
    }

    #[test]
    fn stage_focus_reconstructs_across_repeated_timeline_seeks() {
        let mut scene = Scene::new();
        let diagram_id = scene.add_tattva(TransformerBlockDiagram::encoder(), Vec3::ZERO);
        let mut timeline = Timeline::new();
        timeline
            .animate(diagram_id)
            .at(0.0)
            .for_duration(1.0)
            .ease(Ease::Linear)
            .transformer_focus("self_attention")
            .spawn();
        scene.play(timeline).unwrap();

        for time in [0.5, 0.0, 0.5, 1.0] {
            scene.seek_to(time).unwrap();
            let diagram = scene
                .get_tattva_typed::<TransformerBlockDiagram>(diagram_id)
                .unwrap();
            assert_eq!(
                diagram.state.active_stage_id.as_deref(),
                Some("self_attention")
            );
            if time < 1.0 {
                assert_eq!(
                    diagram.state.focus_transition.as_ref().unwrap().progress,
                    time
                );
            } else {
                assert!(diagram.state.focus_transition.is_none());
            }
        }
    }
}
