use glam::{Vec2, Vec4, vec2};
use std::fmt::Debug;
use std::sync::Arc;

use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Project, ProjectionCtx};

use super::types::{FlowNodeContentVisibility, FlowNodePlacement, FlowNodeShape};

pub trait FlowNodeContent: Debug + Send + Sync {
    fn local_bounds(&self) -> Bounds;
    fn project(&self, ctx: &mut ProjectionCtx);
}

#[derive(Debug)]
pub struct ProjectedFlowNodeContent<T> {
    pub state: T,
}

impl<T> ProjectedFlowNodeContent<T> {
    pub fn new(state: T) -> Self {
        Self { state }
    }
}

impl<T> FlowNodeContent for ProjectedFlowNodeContent<T>
where
    T: Project + Bounded + std::fmt::Debug + Send + Sync + 'static,
{
    fn local_bounds(&self) -> Bounds {
        self.state.local_bounds()
    }

    fn project(&self, ctx: &mut ProjectionCtx) {
        self.state.project(ctx);
    }
}

#[derive(Debug, Clone)]
pub struct FlowNode {
    pub label: String,
    pub shape: FlowNodeShape,
    pub size: Option<Vec2>,
    pub placement: Option<FlowNodePlacement>,
    pub fill_color: Vec4,
    pub stroke_color: Vec4,
    pub text_color: Vec4,
    pub embedded_content: Option<Arc<dyn FlowNodeContent>>,
    pub content_visibility: FlowNodeContentVisibility,
    pub content_padding: Vec2,
    pub reveal_at: Option<f32>,
}

impl FlowNode {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            shape: FlowNodeShape::Rounded,
            size: None,
            placement: None,
            fill_color: Vec4::new(0.16, 0.20, 0.28, 1.0),
            stroke_color: Vec4::new(0.56, 0.63, 0.74, 1.0),
            text_color: Vec4::new(0.96, 0.97, 0.99, 1.0),
            embedded_content: None,
            content_visibility: FlowNodeContentVisibility::Always,
            content_padding: vec2(0.22, 0.18),
            reveal_at: None,
        }
    }

    pub fn with_shape(mut self, shape: FlowNodeShape) -> Self {
        self.shape = shape;
        self
    }

    pub fn with_size(mut self, size: Vec2) -> Self {
        self.size = Some(size);
        self
    }

    pub fn with_placement(mut self, placement: FlowNodePlacement) -> Self {
        self.placement = Some(placement);
        self
    }

    pub fn with_fill_color(mut self, color: Vec4) -> Self {
        self.fill_color = color;
        self
    }

    pub fn with_stroke_color(mut self, color: Vec4) -> Self {
        self.stroke_color = color;
        self
    }

    pub fn with_text_color(mut self, color: Vec4) -> Self {
        self.text_color = color;
        self
    }

    pub fn with_content<T>(mut self, content: T) -> Self
    where
        T: Project + Bounded + std::fmt::Debug + Send + Sync + 'static,
    {
        self.embedded_content = Some(Arc::new(ProjectedFlowNodeContent::new(content)));
        self
    }

    pub fn with_content_renderer(mut self, content: Arc<dyn FlowNodeContent>) -> Self {
        self.embedded_content = Some(content);
        self
    }

    pub fn with_content_visibility(mut self, visibility: FlowNodeContentVisibility) -> Self {
        self.content_visibility = visibility;
        self
    }

    pub fn with_active_only_content(mut self) -> Self {
        self.content_visibility = FlowNodeContentVisibility::ActiveOnly;
        self
    }

    pub fn with_content_padding(mut self, padding: Vec2) -> Self {
        self.content_padding = padding;
        self
    }

    pub fn with_reveal_at(mut self, reveal_at: f32) -> Self {
        self.reveal_at = Some(reveal_at);
        self
    }
}

impl From<&str> for FlowNode {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}
