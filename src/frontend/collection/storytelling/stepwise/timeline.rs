use crate::frontend::collection::storytelling::stepwise::model::*;
use crate::frontend::collection::storytelling::stepwise::state::*;

pub struct TimelineEngine;

impl TimelineEngine {
    pub fn compute(model: &StepwiseModel, progress: f32) -> StepwiseState {
        // Build phase uses the deduplicated sequence (unique nodes only).
        // Signal phase uses model.sequence (may have repeats) — handled separately.
        let sequence = &model.build_sequence;
        let num_steps = sequence.len();

        if num_steps == 0 {
            return StepwiseState {
                steps: vec![],
                transitions: vec![],
            };
        }

        let p = progress.clamp(0.0, 1.0);

        if p >= 1.0 {
            return StepwiseState {
                steps: vec![StepState::Completed; model.steps.len()],
                transitions: vec![TransitionState::Completed; model.transitions.len()],
            };
        }

        // Precompute position of each node in the build sequence.
        let mut seq_index = vec![None::<usize>; model.steps.len()];
        for (i, &step_idx) in sequence.iter().enumerate() {
            seq_index[step_idx] = Some(i);
        }

        let segment_size = 1.0 / num_steps as f32;
        let active_idx = ((p / segment_size).floor() as usize).min(num_steps - 1);
        let local_t = ((p - active_idx as f32 * segment_size) / segment_size).clamp(0.0, 1.0);

        // Step states
        let mut steps = vec![StepState::Pending; model.steps.len()];
        for (i, &step_idx) in sequence.iter().enumerate() {
            steps[step_idx] = if i < active_idx {
                StepState::Completed
            } else if i == active_idx {
                StepState::Active { t: local_t }
            } else {
                StepState::Pending
            };
        }

        // Transition states — use seq_index to determine timing
        let mut transitions = vec![TransitionState::Hidden; model.transitions.len()];
        for (i, transition) in model.transitions.iter().enumerate() {
            if let (Some(from_i), Some(to_i)) =
                (seq_index[transition.from], seq_index[transition.to])
            {
                if from_i + 1 == to_i {
                    // Forward adjacent in build sequence: animate growth
                    transitions[i] = if from_i < active_idx {
                        TransitionState::Completed
                    } else if from_i == active_idx {
                        TransitionState::Drawing { t: local_t }
                    } else {
                        TransitionState::Hidden
                    };
                } else {
                    // Non-adjacent (back-edges, skip-edges): appear when from-node is active
                    // TODO: animated routing for non-adjacent edges (v2)
                    transitions[i] = if from_i <= active_idx {
                        TransitionState::Completed
                    } else {
                        TransitionState::Hidden
                    };
                }
            }
        }

        StepwiseState { steps, transitions }
    }
}
