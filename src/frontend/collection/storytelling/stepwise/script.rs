use crate::frontend::collection::storytelling::stepwise::model::{
    Direction, Step, StepContent, StepwiseModel, Transition,
};
use std::sync::Arc;

pub fn stepwise<F: FnOnce(&mut ScriptBuilder)>(f: F) -> StepwiseModel {
    let mut builder = ScriptBuilder::new();
    f(&mut builder);
    builder.build()
}

/// A fluent builder for defining `Stepwise` storytelling models.
///
/// The builder allows you to define nodes (steps), the connections between them,
/// and the high-level sequence of the "journey".
///
/// ### Example: Loop Transition
/// ```rust
/// use murali::frontend::collection::storytelling::stepwise::model::Direction;
/// use murali::frontend::collection::storytelling::stepwise::script::stepwise;
///
/// let model = stepwise(|s| {
///     let a = s.step("Start");
///     let b = s.step("Process");
///     s.connect(a, b);
///     s.connect(b, a).route(vec![Direction::Up, Direction::Left]);
///     s.with_sequence(vec![a, b, a, b]);
/// });
/// ```
pub struct ScriptBuilder {
    steps: Vec<Step>,
    explicit_connections: Vec<(usize, usize, Option<Vec<Direction>>)>,
    has_explicit_connections: bool,
    explicit_sequence: Option<Vec<usize>>,
}

impl ScriptBuilder {
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            explicit_connections: Vec::new(),
            has_explicit_connections: false,
            explicit_sequence: None,
        }
    }

    /// Adds a basic labeled node to the storytelling model.
    /// Returns the stable index of the step for use in `connect` or `with_sequence`.
    pub fn step(&mut self, label: &str) -> usize {
        let idx = self.steps.len();
        self.steps.push(Step {
            label: label.to_string(),
            content: None,
        });
        idx
    }

    /// Adds a node with custom projected content.
    pub fn step_with_content(
        &mut self,
        label: &str,
        content: impl Into<Arc<dyn StepContent>>,
    ) -> usize {
        let idx = self.steps.len();
        self.steps.push(Step {
            label: label.to_string(),
            content: Some(content.into()),
        });
        idx
    }

    /// Defines a directional transition from one step to another.
    ///
    /// If no explicit connections are provided, `Stepwise` will default to a
    /// linear chain (0 -> 1 -> 2...). If any explicit connections are defined,
    /// the default chain is suppressed.
    pub fn connect(&mut self, from: usize, to: usize) -> ConnectionBuilder<'_> {
        let n = self.steps.len();
        for &idx in &[from, to] {
            if idx >= n {
                panic!(
                    "stepwise: connect() called with invalid step index {}; only {} steps have been added",
                    idx, n
                );
            }
        }
        self.has_explicit_connections = true;
        self.explicit_connections.push((from, to, None));
        let transition_index = self.explicit_connections.len() - 1;
        ConnectionBuilder {
            builder: self,
            transition_index,
        }
    }

    /// Overrides the automatically computed story sequence.
    ///
    /// By default, `Stepwise` performs a topological sort of your connections.
    /// Use `with_sequence` to define manual paths, repeats, or complex loops
    /// (e.g. `[0, 1, 2, 1, 2, 3]`).
    pub fn with_sequence(&mut self, sequence: Vec<usize>) -> &mut Self {
        self.explicit_sequence = Some(sequence);
        self
    }

    pub fn build(self) -> StepwiseModel {
        let n = self.steps.len();

        let sequence = if let Some(seq) = self.explicit_sequence {
            seq
        } else if !self.has_explicit_connections {
            // Auto-generate linear chain
            (0..n).collect()
        } else {
            // Topological sort via Kahn's algorithm
            let mut in_degree = vec![0usize; n];
            let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
            for &(from, to, _) in &self.explicit_connections {
                if from < n && to < n {
                    adj[from].push(to);
                    in_degree[to] += 1;
                }
            }

            let mut queue: std::collections::VecDeque<usize> =
                (0..n).filter(|&i| in_degree[i] == 0).collect();

            let mut seq = Vec::with_capacity(n);
            while let Some(node) = queue.pop_front() {
                seq.push(node);
                for &neighbor in &adj[node] {
                    in_degree[neighbor] -= 1;
                    if in_degree[neighbor] == 0 {
                        queue.push_back(neighbor);
                    }
                }
            }
            seq
        };

        // Build final transitions
        let transitions: Vec<Transition> = if !self.has_explicit_connections {
            // Linear defaults
            (0..n.saturating_sub(1))
                .map(|i| Transition {
                    from: i,
                    to: i + 1,
                    route: None,
                })
                .collect()
        } else {
            self.explicit_connections
                .into_iter()
                .map(|(from, to, route)| Transition { from, to, route })
                .collect()
        };

        StepwiseModel {
            steps: self.steps,
            transitions,
            build_sequence: deduplicate(&sequence),
            sequence,
        }
    }
}

/// Returns unique elements in first-appearance order.
fn deduplicate(seq: &[usize]) -> Vec<usize> {
    let mut seen = std::collections::HashSet::new();
    seq.iter().filter(|&&x| seen.insert(x)).copied().collect()
}

/// Helper for configuring an individual transition (routing, styles, etc.)
pub struct ConnectionBuilder<'a> {
    builder: &'a mut ScriptBuilder,
    transition_index: usize,
}

impl<'a> ConnectionBuilder<'a> {
    /// Configures a deterministic grid-based route for this transition.
    ///
    /// Routes use simple directions (`Up`, `Down`, `Left`, `Right`).
    /// - `Left`/`Right`: Move precisely to the X-center of the previous/next node on the grid.
    /// - `Up`/`Down`: Move to a "lane" above or below the node centers.
    /// - **Spatial Anchoring**: The engine automatically picks the best entry face (top, bottom, side)
    ///   based on where your final route segment terminates.
    pub fn route(self, directions: Vec<Direction>) -> Self {
        self.builder.explicit_connections[self.transition_index].2 = Some(directions);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::collection::storytelling::stepwise::{
        model::Direction, timeline::TimelineEngine,
    };

    // Feature: stepwise-component, Property 1: Step index assignment is sequential
    // Validates: Requirements 1.2
    proptest::proptest! {
        #[test]
        fn prop_step_index_sequential(labels in proptest::collection::vec(".*", 1usize..=20)) {
            let n = labels.len();
            let model = stepwise(|s| {
                for label in &labels {
                    s.step(label);
                }
            });
            proptest::prop_assert_eq!(model.steps.len(), n);
            for (i, label) in labels.iter().enumerate() {
                proptest::prop_assert_eq!(&model.steps[i].label, label);
            }
        }
    }

    // Feature: stepwise-component, Property 2: Auto-generated model is a linear chain
    // Validates: Requirements 1.3, 8.2, 8.3
    proptest::proptest! {
        #[test]
        fn prop_auto_linear_chain(n in 1usize..=20) {
            let model = stepwise(|s| {
                for i in 0..n {
                    s.step(&format!("step_{}", i));
                }
            });
            let expected_seq: Vec<usize> = (0..n).collect();
            proptest::prop_assert_eq!(&model.sequence, &expected_seq);
            proptest::prop_assert_eq!(model.transitions.len(), n.saturating_sub(1));
            for i in 0..n.saturating_sub(1) {
                proptest::prop_assert_eq!(model.transitions[i].from, i);
                proptest::prop_assert_eq!(model.transitions[i].to, i + 1);
            }
        }
    }

    // Feature: stepwise-component, Property 3: Sequence is a permutation of step indices
    // Validates: Requirements 1.4
    proptest::proptest! {
        #[test]
        fn prop_sequence_is_permutation(n in 1usize..=10) {
            let model = stepwise(|s| {
                for i in 0..n {
                    s.step(&format!("step_{}", i));
                }
                // Connect i -> i+1 to form a linear DAG
                for i in 0..n.saturating_sub(1) {
                    s.connect(i, i + 1);
                }
            });
            let mut sorted = model.sequence.clone();
            sorted.sort();
            let expected: Vec<usize> = (0..n).collect();
            proptest::prop_assert_eq!(sorted, expected);
        }
    }

    // Feature: stepwise-component, Property 4: Explicit connections suppress auto-generation
    // Validates: Requirements 2.2
    proptest::proptest! {
        #[test]
        fn prop_explicit_connections_suppress_auto(n in 2usize..=10) {
            let model = stepwise(|s| {
                for i in 0..n {
                    s.step(&format!("step_{}", i));
                }
                s.connect(0, 1);
            });
            // Only the one explicit connection should exist
            proptest::prop_assert_eq!(model.transitions.len(), 1);
            proptest::prop_assert_eq!(model.transitions[0].from, 0);
            proptest::prop_assert_eq!(model.transitions[0].to, 1);
        }
    }

    // Feature: stepwise-component, Property 5: Route is stored on the transition
    // Validates: Requirements 3.2
    proptest::proptest! {
        #[test]
        fn prop_route_stored_on_transition(dirs in proptest::collection::vec(0u8..4, 0usize..=10)) {
            let directions: Vec<Direction> = dirs.iter().map(|&d| match d {
                0 => Direction::Up,
                1 => Direction::Down,
                2 => Direction::Left,
                _ => Direction::Right,
            }).collect();
            let dirs_clone = directions.clone();
            let model = stepwise(|s| {
                s.step("a");
                s.step("b");
                s.connect(0, 1).route(dirs_clone.clone());
            });
            proptest::prop_assert_eq!(&model.transitions[0].route, &Some(directions));
        }
    }

    // Feature: stepwise-component, Property 8: Script_API output is functionally equivalent to manual construction
    // Validates: Requirements 8.1
    proptest::proptest! {
        #[test]
        fn prop_script_api_equivalent_to_manual(
            labels in proptest::collection::vec("[a-z]{1,8}", 1usize..=8),
            p in 0.0f32..=1.0f32
        ) {
            use crate::frontend::collection::storytelling::stepwise::model::{Step, StepwiseModel, Transition};

            let n = labels.len();

            // Build via Script API
            let script_model = stepwise(|s| {
                for label in &labels {
                    s.step(label);
                }
            });

            // Build manually
            let manual_model = StepwiseModel {
                steps: labels.iter().map(|l| Step { label: l.clone(), content: None }).collect(),
                transitions: (0..n.saturating_sub(1))
                    .map(|i| Transition { from: i, to: i + 1, route: None })
                    .collect(),
                build_sequence: (0..n).collect(),
                sequence: (0..n).collect(),
            };

            let script_state = TimelineEngine::compute(&script_model, p);
            let manual_state = TimelineEngine::compute(&manual_model, p);

            proptest::prop_assert_eq!(script_state, manual_state);
        }
    }
}
