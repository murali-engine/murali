use super::types::EdgeStep;

#[derive(Debug, Clone, PartialEq)]
pub struct FlowEdge {
    pub from: usize,
    pub to: usize,
    pub route_steps: Vec<EdgeStep>,
    pub reveal_at: Option<f32>,
    pub reveal_window: Option<f32>,
}

impl FlowEdge {
    pub fn new(from: usize, to: usize) -> Self {
        Self {
            from,
            to,
            route_steps: Vec::new(),
            reveal_at: None,
            reveal_window: None,
        }
    }

    pub fn with_route_steps(mut self, steps: Vec<EdgeStep>) -> Self {
        self.route_steps = steps;
        self
    }

    pub fn with_reveal_at(mut self, reveal_at: f32) -> Self {
        self.reveal_at = Some(reveal_at);
        self
    }

    pub fn with_reveal_window(mut self, window: f32) -> Self {
        self.reveal_window = Some(window);
        self
    }

    pub fn with_route_hint(self, steps: Vec<EdgeStep>) -> Self {
        self.with_route_steps(steps)
    }
}
