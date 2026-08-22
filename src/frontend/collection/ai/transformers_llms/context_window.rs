use std::collections::HashSet;

use glam::{Vec2, Vec3, Vec4, vec2, vec3};

use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Mesh, Project, ProjectionCtx, RenderPrimitive};
use crate::validation::ValidationError;

const COMPONENT: &str = "ContextWindow";

/// Semantic source of one block in a model context window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextBlockRole {
    System,
    User,
    Assistant,
    Tool,
    Retrieved,
}

impl ContextBlockRole {
    pub fn label(self) -> &'static str {
        match self {
            Self::System => "SYSTEM",
            Self::User => "USER",
            Self::Assistant => "ASSISTANT",
            Self::Tool => "TOOL",
            Self::Retrieved => "RETRIEVED",
        }
    }

    fn color(self) -> Vec4 {
        match self {
            Self::System => Vec4::new(0.61, 0.48, 0.88, 1.0),
            Self::User => Vec4::new(0.35, 0.77, 0.87, 1.0),
            Self::Assistant => Vec4::new(0.36, 0.82, 0.70, 1.0),
            Self::Tool => Vec4::new(0.95, 0.67, 0.37, 1.0),
            Self::Retrieved => Vec4::new(0.52, 0.76, 0.40, 1.0),
        }
    }
}

/// Edge from which omitted tokens were removed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextTruncation {
    FromStart,
    FromEnd,
}

/// One ordered, role-tagged contribution to a model context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextBlock {
    pub id: String,
    pub label: String,
    pub preview: String,
    pub role: ContextBlockRole,
    pub token_count: usize,
    pub retained_tokens: usize,
    pub truncation: Option<ContextTruncation>,
}

impl ContextBlock {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        role: ContextBlockRole,
        token_count: usize,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            preview: String::new(),
            role,
            token_count,
            retained_tokens: token_count,
            truncation: None,
        }
    }

    pub fn with_preview(mut self, preview: impl Into<String>) -> Self {
        self.preview = preview.into();
        self
    }

    /// Marks the block as explicitly truncated while retaining `retained_tokens`.
    pub fn truncated_to(mut self, retained_tokens: usize, truncation: ContextTruncation) -> Self {
        self.retained_tokens = retained_tokens;
        self.truncation = Some(truncation);
        self
    }

    pub fn omitted_tokens(&self) -> usize {
        self.token_count.saturating_sub(self.retained_tokens)
    }
}

/// A semantic visualization of the ordered data assembled for one model invocation.
#[derive(Debug, Clone)]
pub struct ContextWindow {
    pub blocks: Vec<ContextBlock>,
    pub token_budget: usize,
    pub title: String,
    pub width: f32,
    pub row_height: f32,
    pub row_gap: f32,
    pub padding: f32,
    pub text_color: Vec4,
    pub panel_color: Vec4,
    pub track_color: Vec4,
    pub truncated_color: Vec4,
}

impl ContextWindow {
    pub fn try_new(
        blocks: Vec<ContextBlock>,
        token_budget: usize,
    ) -> Result<Self, ValidationError> {
        let window = Self {
            blocks,
            token_budget,
            title: "MODEL CONTEXT".to_string(),
            width: 9.2,
            row_height: 0.68,
            row_gap: 0.13,
            padding: 0.34,
            text_color: Vec4::new(0.94, 0.97, 1.0, 1.0),
            panel_color: Vec4::new(0.055, 0.068, 0.085, 0.98),
            track_color: Vec4::new(0.14, 0.17, 0.21, 1.0),
            truncated_color: Vec4::new(0.34, 0.22, 0.25, 1.0),
        };
        window.validate()?;
        Ok(window)
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn used_tokens(&self) -> usize {
        self.blocks.iter().map(|block| block.retained_tokens).sum()
    }

    pub fn available_tokens(&self) -> usize {
        self.token_budget.saturating_sub(self.used_tokens())
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.blocks.is_empty() {
            return Err(ValidationError::Empty {
                component: COMPONENT,
                field: "blocks",
            });
        }
        if self.token_budget == 0 {
            return Err(ValidationError::CountTooSmall {
                component: COMPONENT,
                field: "token budget",
                minimum: 1,
                actual: 0,
            });
        }
        if self.title.trim().is_empty() {
            return Err(ValidationError::Empty {
                component: COMPONENT,
                field: "title",
            });
        }

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
        for (field, value) in [("row gap", self.row_gap), ("padding", self.padding)] {
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
        if self.width <= self.padding * 2.0 + 3.0 {
            return Err(ValidationError::Incompatible {
                component: COMPONENT,
                field: "layout width",
                reason: "width must leave room for labels and the token track after padding"
                    .to_string(),
            });
        }

        let mut ids = HashSet::with_capacity(self.blocks.len());
        let mut used_tokens = 0usize;
        for block in &self.blocks {
            if block.id.trim().is_empty() {
                return Err(ValidationError::Empty {
                    component: COMPONENT,
                    field: "block id",
                });
            }
            if !ids.insert(block.id.as_str()) {
                return Err(ValidationError::DuplicateIdentifier {
                    component: COMPONENT,
                    field: "blocks",
                    value: block.id.clone(),
                });
            }
            if block.label.trim().is_empty() {
                return Err(ValidationError::Empty {
                    component: COMPONENT,
                    field: "block label",
                });
            }
            if block.token_count == 0 {
                return Err(ValidationError::CountTooSmall {
                    component: COMPONENT,
                    field: "block token count",
                    minimum: 1,
                    actual: 0,
                });
            }
            if block.retained_tokens > block.token_count {
                return Err(ValidationError::Incompatible {
                    component: COMPONENT,
                    field: "block retention",
                    reason: format!(
                        "block '{}' retains {} of {} tokens",
                        block.id, block.retained_tokens, block.token_count
                    ),
                });
            }
            let is_truncated = block.retained_tokens < block.token_count;
            if is_truncated != block.truncation.is_some() {
                return Err(ValidationError::Incompatible {
                    component: COMPONENT,
                    field: "block truncation",
                    reason: format!(
                        "block '{}' must name a truncation edge exactly when tokens are omitted",
                        block.id
                    ),
                });
            }
            used_tokens = used_tokens
                .checked_add(block.retained_tokens)
                .ok_or_else(|| ValidationError::ShapeOverflow {
                    component: COMPONENT,
                    field: "retained token total",
                })?;
        }
        if used_tokens > self.token_budget {
            return Err(ValidationError::Incompatible {
                component: COMPONENT,
                field: "token budget",
                reason: format!(
                    "{used_tokens} retained tokens exceed budget {}",
                    self.token_budget
                ),
            });
        }
        Ok(())
    }

    fn height(&self) -> f32 {
        let rows = self.blocks.len() as f32 * self.row_height
            + self.blocks.len().saturating_sub(1) as f32 * self.row_gap;
        self.padding * 2.0 + 0.55 + rows + 0.52
    }
}

impl Project for ContextWindow {
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
            &self.title,
            0.22,
            self.text_color,
            vec3(-self.width * 0.5 + self.padding + 1.25, top - 0.13, 0.04),
        );
        emit_text(
            ctx,
            &format!("{} / {} TOKENS", self.used_tokens(), self.token_budget),
            0.17,
            Vec4::new(0.70, 0.75, 0.80, 1.0),
            vec3(self.width * 0.5 - self.padding - 0.85, top - 0.13, 0.04),
        );

        let label_x = -self.width * 0.5 + self.padding + 1.05;
        let track_x = -self.width * 0.5 + self.padding + 2.25;
        let track_width = self.width - self.padding * 2.0 - 2.45;
        let mut row_y = top - 0.55 - self.row_height * 0.5;

        for block in &self.blocks {
            let role_color = block.role.color();
            emit_rect(
                ctx,
                self.width - self.padding * 2.0,
                self.row_height,
                Vec4::new(0.09, 0.11, 0.14, 1.0),
                vec3(0.0, row_y, 0.01),
            );
            emit_rect(
                ctx,
                0.07,
                self.row_height,
                role_color,
                vec3(-self.width * 0.5 + self.padding + 0.035, row_y, 0.03),
            );

            emit_text(
                ctx,
                block.role.label(),
                0.12,
                role_color,
                vec3(label_x, row_y + 0.14, 0.05),
            );
            emit_text(
                ctx,
                &block.label,
                0.16,
                self.text_color,
                vec3(label_x, row_y - 0.10, 0.05),
            );

            emit_rect(
                ctx,
                track_width,
                0.28,
                self.track_color,
                vec3(track_x + track_width * 0.5, row_y, 0.03),
            );

            let scale = track_width / self.token_budget as f32;
            let original_width = (block.token_count as f32 * scale).min(track_width);
            let retained_width = (block.retained_tokens as f32 * scale).min(original_width);
            let omitted_width = (original_width - retained_width).max(0.0);
            let (retained_x, omitted_x) = match block.truncation {
                Some(ContextTruncation::FromStart) => (
                    track_x + omitted_width + retained_width * 0.5,
                    track_x + omitted_width * 0.5,
                ),
                _ => (
                    track_x + retained_width * 0.5,
                    track_x + retained_width + omitted_width * 0.5,
                ),
            };
            if retained_width > 0.0 {
                emit_rect(
                    ctx,
                    retained_width,
                    0.28,
                    role_color,
                    vec3(retained_x, row_y, 0.05),
                );
            }
            if omitted_width > 0.0 {
                emit_rect(
                    ctx,
                    omitted_width,
                    0.28,
                    self.truncated_color,
                    vec3(omitted_x, row_y, 0.05),
                );
                emit_truncation_marks(ctx, omitted_x, row_y, omitted_width);
            }

            let preview = compact_preview(&block.preview, 34);
            let annotation = if preview.is_empty() {
                format!("{} tokens", block.retained_tokens)
            } else {
                format!("{preview}  |  {} tokens", block.retained_tokens)
            };
            emit_text(
                ctx,
                &annotation,
                0.12,
                Vec4::new(0.89, 0.92, 0.95, 1.0),
                vec3(track_x + track_width * 0.5, row_y, 0.08),
            );

            row_y -= self.row_height + self.row_gap;
        }

        let meter_y = -height * 0.5 + self.padding + 0.13;
        let meter_width = self.width - self.padding * 2.0;
        emit_rect(
            ctx,
            meter_width,
            0.12,
            self.track_color,
            vec3(0.0, meter_y, 0.03),
        );
        let used_width = meter_width * self.used_tokens() as f32 / self.token_budget as f32;
        if used_width > 0.0 {
            emit_rect(
                ctx,
                used_width,
                0.12,
                Vec4::new(0.35, 0.77, 0.87, 1.0),
                vec3(-meter_width * 0.5 + used_width * 0.5, meter_y, 0.05),
            );
        }
    }
}

impl Bounded for ContextWindow {
    fn local_bounds(&self) -> Bounds {
        if self.validate().is_err() {
            return Bounds::from_center_size(Vec2::ZERO, Vec2::splat(0.1));
        }
        Bounds::from_center_size(Vec2::ZERO, vec2(self.width, self.height()))
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

fn emit_truncation_marks(ctx: &mut ProjectionCtx, center_x: f32, center_y: f32, width: f32) {
    let half = width * 0.5;
    let mut x = center_x - half + 0.08;
    while x < center_x + half {
        ctx.emit(RenderPrimitive::Line {
            start: vec3(x - 0.05, center_y - 0.10, 0.07),
            end: vec3(x + 0.05, center_y + 0.10, 0.07),
            thickness: 0.012,
            color: Vec4::new(0.85, 0.50, 0.53, 0.9),
            dash_length: 0.0,
            gap_length: 0.0,
            dash_offset: 0.0,
        });
        x += 0.16;
    }
}

fn compact_preview(preview: &str, max_chars: usize) -> String {
    let mut chars = preview.chars();
    let compact: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{compact}...")
    } else {
        compact
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::scene::SharedProps;

    fn valid_blocks() -> Vec<ContextBlock> {
        vec![
            ContextBlock::new("system", "Instructions", ContextBlockRole::System, 120),
            ContextBlock::new("history", "Earlier turns", ContextBlockRole::User, 700)
                .truncated_to(420, ContextTruncation::FromStart),
        ]
    }

    #[test]
    fn reports_usage_and_explicit_truncation() {
        let window = ContextWindow::try_new(valid_blocks(), 1_000).unwrap();
        assert_eq!(window.used_tokens(), 540);
        assert_eq!(window.available_tokens(), 460);
        assert_eq!(window.blocks[1].omitted_tokens(), 280);
    }

    #[test]
    fn rejects_implicit_truncation_and_budget_overflow() {
        let mut implicit = valid_blocks();
        implicit[1].truncation = None;
        assert!(matches!(
            ContextWindow::try_new(implicit, 1_000),
            Err(ValidationError::Incompatible {
                field: "block truncation",
                ..
            })
        ));

        assert!(matches!(
            ContextWindow::try_new(valid_blocks(), 500),
            Err(ValidationError::Incompatible {
                field: "token budget",
                ..
            })
        ));
    }

    #[test]
    fn invalid_window_emits_a_diagnostic_instead_of_geometry() {
        let mut window = ContextWindow::try_new(valid_blocks(), 1_000).unwrap();
        window.blocks[0].id = window.blocks[1].id.clone();
        let mut ctx = ProjectionCtx::new(SharedProps::default());
        window.project(&mut ctx);
        assert!(ctx.primitives.is_empty());
        assert!(matches!(
            ctx.diagnostics[0],
            ValidationError::DuplicateIdentifier { .. }
        ));
    }
}
