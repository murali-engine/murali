use crate::frontend::layout::{Bounded, Bounds};
use crate::frontend::style::{StrokeParams, Style};
use crate::projection::Mesh;
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};
use glam::{Vec2, Vec4, vec2, vec3};

#[derive(Debug, Clone)]
pub struct Rectangle {
    pub width: f32,
    pub height: f32,
    pub style: Style,
}

impl Rectangle {
    pub fn new(width: f32, height: f32, color: Vec4) -> Self {
        Self {
            width,
            height,
            style: Style::new().with_fill(color),
        }
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_stroke(mut self, thickness: f32, color: Vec4) -> Self {
        self.style.stroke = Some(StrokeParams {
            thickness,
            color,
            ..Default::default()
        });
        self
    }
}

impl Project for Rectangle {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let hw = self.width * 0.5;
        let hh = self.height * 0.5;

        // Fill
        if let Some(fill) = &self.style.fill {
            let mesh = Mesh::rectangle(self.width, self.height, fill.clone());
            ctx.emit(RenderPrimitive::Mesh(mesh));
        }

        // Stroke
        if let Some(stroke) = &self.style.stroke {
            let pts = [vec2(-hw, -hh), vec2(hw, -hh), vec2(hw, hh), vec2(-hw, hh)];
            for i in 0..4 {
                let j = (i + 1) % 4;
                ctx.emit(RenderPrimitive::Line {
                    start: vec3(pts[i].x, pts[i].y, 0.0),
                    end: vec3(pts[j].x, pts[j].y, 0.0),
                    thickness: stroke.thickness,
                    color: stroke.color,
                    dash_length: stroke.dash_length,
                    gap_length: stroke.gap_length,
                    dash_offset: stroke.dash_offset,
                });
            }
        }
    }
}

impl Bounded for Rectangle {
    fn local_bounds(&self) -> Bounds {
        let hw = self.width * 0.5;
        let hh = self.height * 0.5;
        Bounds::new(vec2(-hw, -hh), vec2(hw, hh))
    }
}
