// src/frontend/collection/storytelling/stepwise/model.rs
use super::state::StepState;
use crate::projection::{Project, ProjectionCtx};
use std::sync::Arc;

#[derive(Debug, PartialEq, Clone)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Clone)]
pub struct Step {
    pub label: String,
    pub content: Option<Arc<dyn StepContent>>,
}

impl std::fmt::Debug for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Step")
            .field("label", &self.label)
            .field("content", &self.content.as_ref().map(|_| "<StepContent>"))
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct Transition {
    pub from: usize,
    pub to: usize,
    pub route: Option<Vec<Direction>>,
}

#[derive(Debug, Clone)]
pub struct StepwiseModel {
    pub steps: Vec<Step>,
    pub transitions: Vec<Transition>,
    /// The signal/journey sequence — may contain repeated indices for loops.
    /// Used by `signal_progress` animation.
    pub sequence: Vec<usize>,
    /// Deduplicated build sequence — unique nodes in first-appearance order.
    /// Used by `TimelineEngine::compute` (the `progress` build animation).
    /// Automatically derived from `sequence` by the script builder.
    pub build_sequence: Vec<usize>,
}

// ── StepContent trait ────────────────────────────────────────────────────────

pub trait StepContent: Send + Sync {
    fn project(&self, ctx: &mut ProjectionCtx, state: &StepState);

    /// Projects the content when it is being "indicated" (pulsed by the storyteller).
    /// Default implementation performs a local 1.15x scale pulse.
    fn project_indicated(&self, ctx: &mut ProjectionCtx, state: &StepState, intensity: f32) {
        ctx.with_scale(1.0 + 0.15 * intensity, |ctx| {
            self.project(ctx, state);
        });
    }

    /// Return `true` if this content draws its own background container.
    fn draws_own_background(&self) -> bool {
        false
    }
}

// ── TattvaContent adapter ────────────────────────────────────────────────────

pub struct TattvaContent<T: Project + Send + Sync> {
    pub inner: T,
}

impl<T: Project + Send + Sync> TattvaContent<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T: Project + Send + Sync + 'static> From<TattvaContent<T>> for Arc<dyn StepContent> {
    fn from(value: TattvaContent<T>) -> Self {
        Arc::new(value)
    }
}

impl<T: Project + Send + Sync> StepContent for TattvaContent<T> {
    fn project(&self, ctx: &mut ProjectionCtx, state: &StepState) {
        let opacity = match state {
            StepState::Pending => 0.0,
            StepState::Active { t } => *t,
            StepState::Completed => 0.75,
        };

        // Use the new tiered opacity stack in the engine:
        ctx.with_opacity(opacity, |ctx| {
            self.inner.project(ctx);
        });
    }
}
