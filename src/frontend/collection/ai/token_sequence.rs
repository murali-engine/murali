use std::collections::HashSet;

use glam::{Vec2, Vec3, Vec4, vec2};

use super::tensor::TensorAxis;
use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};
use crate::resource::text::layout::measure_label;
use crate::validation::ValidationError;

const COMPONENT: &str = "TokenSequence";

#[derive(Debug, Clone)]
pub struct TokenSequence {
    pub token_ids: Vec<String>,
    pub tokens: Vec<String>,
    pub token_height: f32,
    pub gap: f32,
    pub box_padding: Vec2,
    pub text_color: Vec4,
    pub box_color: Vec4,
    pub line_thickness: f32,
}

impl TokenSequence {
    pub fn new(tokens: Vec<impl Into<String>>, token_height: f32) -> Self {
        let tokens: Vec<String> = tokens.into_iter().map(Into::into).collect();
        Self {
            token_ids: (0..tokens.len())
                .map(|index| format!("token.{index}"))
                .collect(),
            tokens,
            token_height,
            gap: token_height * 0.45,
            box_padding: vec2(token_height * 0.35, token_height * 0.28),
            text_color: Vec4::new(0.97, 0.98, 0.99, 1.0),
            box_color: Vec4::new(0.42, 0.55, 0.86, 1.0),
            line_thickness: 0.02,
        }
    }

    /// Creates a sequence with stable token IDs distinct from display text.
    pub fn try_with_tokens<I, ID, T>(tokens: I, token_height: f32) -> Result<Self, ValidationError>
    where
        I: IntoIterator<Item = (ID, T)>,
        ID: Into<String>,
        T: Into<String>,
    {
        let (token_ids, tokens) = tokens
            .into_iter()
            .map(|(id, text)| (id.into(), text.into()))
            .unzip();
        let sequence = Self {
            token_ids,
            tokens,
            ..Self::new(Vec::<String>::new(), token_height)
        };
        sequence.validate()?;
        Ok(sequence)
    }

    /// Creates a token row from a tensor axis, preserving shared element identity.
    pub fn try_from_axis(axis: &TensorAxis, token_height: f32) -> Result<Self, ValidationError> {
        Self::try_with_tokens(
            axis.element_ids
                .iter()
                .cloned()
                .zip(axis.element_labels.iter().cloned()),
            token_height,
        )
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.tokens.is_empty() {
            return Err(ValidationError::Empty {
                component: COMPONENT,
                field: "tokens",
            });
        }
        if self.token_ids.len() != self.tokens.len() {
            return Err(ValidationError::LengthMismatch {
                component: COMPONENT,
                field: "token ids",
                expected: self.tokens.len(),
                actual: self.token_ids.len(),
            });
        }
        let mut ids = HashSet::with_capacity(self.token_ids.len());
        for id in &self.token_ids {
            if id.trim().is_empty() {
                return Err(ValidationError::Empty {
                    component: COMPONENT,
                    field: "token id",
                });
            }
            if !ids.insert(id.as_str()) {
                return Err(ValidationError::DuplicateIdentifier {
                    component: COMPONENT,
                    field: "token ids",
                    value: id.clone(),
                });
            }
        }
        for token in &self.tokens {
            if token.is_empty() {
                return Err(ValidationError::Empty {
                    component: COMPONENT,
                    field: "token text",
                });
            }
        }
        for (field, value) in [
            ("token height", self.token_height),
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
        if !self.gap.is_finite() {
            return Err(ValidationError::NonFinite {
                component: COMPONENT,
                field: "gap",
                value: self.gap,
            });
        }
        if self.gap < 0.0 {
            return Err(ValidationError::OutOfRange {
                component: COMPONENT,
                field: "gap",
                minimum: 0.0,
                maximum: f32::MAX,
                value: self.gap,
            });
        }
        for (field, value) in [
            ("horizontal box padding", self.box_padding.x),
            ("vertical box padding", self.box_padding.y),
        ] {
            if !value.is_finite() {
                return Err(ValidationError::NonFinite {
                    component: COMPONENT,
                    field,
                    value,
                });
            }
            if value < 0.0 {
                return Err(ValidationError::OutOfRange {
                    component: COMPONENT,
                    field,
                    minimum: 0.0,
                    maximum: f32::MAX,
                    value,
                });
            }
        }
        Ok(())
    }

    fn token_size(&self, token: &str) -> Vec2 {
        let layout = measure_label(token, self.token_height, None);
        vec2(
            layout.width + self.box_padding.x * 2.0,
            layout.height + self.box_padding.y * 2.0,
        )
    }
}

impl Project for TokenSequence {
    fn project(&self, ctx: &mut ProjectionCtx) {
        if let Err(error) = self.validate() {
            ctx.report(error);
            return;
        }
        let sizes: Vec<Vec2> = self.tokens.iter().map(|t| self.token_size(t)).collect();
        let total_width = sizes.iter().map(|s| s.x).sum::<f32>()
            + self.gap * self.tokens.len().saturating_sub(1) as f32;
        let mut cursor = -total_width * 0.5;

        for (token, size) in self.tokens.iter().zip(sizes) {
            let center_x = cursor + size.x * 0.5;
            let left = center_x - size.x * 0.5;
            let right = center_x + size.x * 0.5;
            let top = size.y * 0.5;
            let bottom = -size.y * 0.5;

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
                    color: self.box_color,
                    dash_length: 0.0,
                    gap_length: 0.0,
                    dash_offset: 0.0,
                });
            }

            ctx.emit(RenderPrimitive::Text {
                content: token.clone(),
                height: self.token_height,
                color: self.text_color,
                font_name: None,
                offset: Vec3::new(center_x, 0.0, 0.0),
                rotation: 0.0,
            });

            cursor += size.x + self.gap;
        }
    }
}

impl Bounded for TokenSequence {
    fn local_bounds(&self) -> Bounds {
        if self.validate().is_err() {
            return Bounds::from_center_size(Vec2::ZERO, Vec2::splat(0.1));
        }
        let sizes: Vec<Vec2> = self.tokens.iter().map(|t| self.token_size(t)).collect();
        let total_width = sizes.iter().map(|s| s.x).sum::<f32>()
            + self.gap * self.tokens.len().saturating_sub(1) as f32;
        let max_height = sizes.iter().map(|s| s.y).fold(0.0, f32::max);
        Bounds::from_center_size(Vec2::ZERO, vec2(total_width.max(0.1), max_height.max(0.1)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::scene::SharedProps;

    #[test]
    fn tensor_axis_identity_is_preserved_in_token_sequence() {
        let axis = TensorAxis::with_elements(
            "token",
            "Tokens",
            [("token.7", "AI"), ("token.8", "learns")],
        );
        let sequence = TokenSequence::try_from_axis(&axis, 0.2).unwrap();
        assert_eq!(sequence.token_ids, axis.element_ids);
        assert_eq!(sequence.tokens, axis.element_labels);
    }

    #[test]
    fn duplicate_ids_and_direct_length_mutation_are_rejected() {
        assert!(matches!(
            TokenSequence::try_with_tokens([("same", "A"), ("same", "B")], 0.2),
            Err(ValidationError::DuplicateIdentifier { .. })
        ));

        let mut sequence = TokenSequence::new(vec!["A", "B"], 0.2);
        sequence.token_ids.pop();
        let mut ctx = ProjectionCtx::new(SharedProps::default());
        sequence.project(&mut ctx);
        assert!(ctx.primitives.is_empty());
        assert!(matches!(
            ctx.diagnostics[0],
            ValidationError::LengthMismatch {
                field: "token ids",
                ..
            }
        ));
    }
}
