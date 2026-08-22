use crate::frontend::layout::{Bounded, Bounds};
use crate::frontend::style::{StrokeParams, Style};
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};
use glam::{Vec3, Vec4};

#[derive(Debug, Clone)]
pub struct Line {
    pub start: Vec3,
    pub end: Vec3,
    pub style: Style,
}

impl Line {
    /// Pure semantic constructor.
    pub fn new(start: Vec3, end: Vec3, thickness: f32, color: Vec4) -> Self {
        Self {
            start,
            end,
            style: Style::new().with_stroke(StrokeParams {
                thickness,
                color,
                ..Default::default()
            }),
        }
    }

    /// Creates a line from the origin to a point
    pub fn to(end: Vec3) -> Self {
        Self::new(Vec3::ZERO, end, 0.02, Vec4::ONE)
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_dash(mut self, dash: f32, gap: f32) -> Self {
        if let Some(stroke) = &mut self.style.stroke {
            stroke.dash_length = dash;
            stroke.gap_length = gap;
        }
        self
    }
}

impl Project for Line {
    fn project(&self, ctx: &mut ProjectionCtx) {
        if let Some(stroke) = &self.style.stroke {
            ctx.emit(RenderPrimitive::Line {
                start: self.start,
                end: self.end,
                thickness: stroke.thickness,
                color: stroke.color,
                dash_length: stroke.dash_length,
                gap_length: stroke.gap_length,
                dash_offset: stroke.dash_offset,
            });
        }
    }
}

impl Bounded for Line {
    fn local_bounds(&self) -> Bounds {
        let thickness = self
            .style
            .stroke
            .as_ref()
            .map(|s| s.thickness)
            .unwrap_or(0.0);
        let half = thickness * 0.5;
        Bounds::new(
            glam::vec2(
                self.start.x.min(self.end.x) - half,
                self.start.y.min(self.end.y) - half,
            ),
            glam::vec2(
                self.start.x.max(self.end.x) + half,
                self.start.y.max(self.end.y) + half,
            ),
        )
    }
}
