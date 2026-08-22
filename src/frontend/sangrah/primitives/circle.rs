use crate::frontend::layout::{Bounded, Bounds};
use crate::frontend::style::{StrokeParams, Style};
use crate::projection::Mesh;
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};
use glam::{Vec4, vec3};

#[derive(Debug, Clone)]
pub struct Circle {
    pub radius: f32,
    pub segments: u32,
    pub style: Style,
}

impl Circle {
    /// Pure semantic constructor. No Mesh is created here.
    pub fn new(radius: f32, segments: u32, color: Vec4) -> Self {
        Self {
            radius,
            segments,
            style: Style::new().with_fill(color),
        }
    }

    /// Convenience default
    pub fn default_unit() -> Self {
        Self::new(1.0, 32, Vec4::new(1.0, 1.0, 1.0, 1.0))
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

impl Project for Circle {
    fn project(&self, ctx: &mut ProjectionCtx) {
        // Fill
        if let Some(fill) = &self.style.fill {
            let mesh = Mesh::circle(self.radius, self.segments, fill.clone());
            ctx.emit(RenderPrimitive::Mesh(mesh));
        }

        // Stroke
        if let Some(stroke) = &self.style.stroke {
            let seg = self.segments.max(3);
            for i in 0..seg {
                let t0 = (i as f32 / seg as f32) * std::f32::consts::TAU;
                let t1 = ((i + 1) as f32 / seg as f32) * std::f32::consts::TAU;
                let p0 = glam::vec2(self.radius * t0.cos(), self.radius * t0.sin());
                let p1 = glam::vec2(self.radius * t1.cos(), self.radius * t1.sin());

                ctx.emit(RenderPrimitive::Line {
                    start: vec3(p0.x, p0.y, 0.0),
                    end: vec3(p1.x, p1.y, 0.0),
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

impl Bounded for Circle {
    fn local_bounds(&self) -> Bounds {
        Bounds::new(
            glam::vec2(-self.radius, -self.radius),
            glam::vec2(self.radius, self.radius),
        )
    }
}
