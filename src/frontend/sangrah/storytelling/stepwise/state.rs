#[derive(Debug, Clone, PartialEq)]
pub enum StepState {
    Pending,
    Active { t: f32 }, // 0.0 → 1.0 within this step
    Completed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransitionState {
    Hidden,
    Drawing { t: f32 },
    Completed,
}

/// The output of `TimelineEngine::compute`. Contains only step and transition
/// states — signal flow is handled independently via `Stepwise::signal_progress`.
#[derive(Debug, Clone, PartialEq)]
pub struct StepwiseState {
    pub steps: Vec<StepState>,
    pub transitions: Vec<TransitionState>,
}
