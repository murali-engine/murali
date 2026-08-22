use crate::frontend::layout::{Bounded, Bounds};
use crate::frontend::style::{StrokeParams, Style};
use crate::projection::Mesh;
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};
use glam::{Vec4, vec2, vec3};

#[derive(Debug, Clone)]
pub struct Square {
    pub size: f32,
    pub style: Style,
}

impl Square {
    /// Pure semantic constructor.
    pub fn new(size: f32, color: Vec4) -> Self {
        Self {
            size,
            style: Style::new().with_fill(color),
        }
    }

    /// Convenience default
    pub fn default_unit() -> Self {
        Self::new(1.0, Vec4::new(1.0, 1.0, 1.0, 1.0))
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

impl Project for Square {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let half = self.size * 0.5;

        // Fill
        if let Some(fill) = &self.style.fill {
            let mesh = Mesh::square(self.size, fill.clone());
            ctx.emit(RenderPrimitive::Mesh(mesh));
        }

        // Stroke
        if let Some(stroke) = &self.style.stroke {
            let pts = [
                vec2(-half, -half),
                vec2(half, -half),
                vec2(half, half),
                vec2(-half, half),
            ];
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

impl Bounded for Square {
    fn local_bounds(&self) -> Bounds {
        let half = self.size * 0.5;
        Bounds::new(glam::vec2(-half, -half), glam::vec2(half, half))
    }
}
