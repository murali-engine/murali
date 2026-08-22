use crate::frontend::layout::{Bounded, Bounds};
use crate::frontend::style::{StrokeParams, Style};
use crate::projection::Mesh;
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};
use glam::{Vec2, Vec4, vec3};

#[derive(Debug, Clone)]
pub struct Polygon {
    pub vertices: Vec<Vec2>,
    pub style: Style,
}

impl Polygon {
    pub fn new(vertices: Vec<Vec2>, color: Vec4) -> Self {
        Self {
            vertices,
            style: Style::new().with_fill(color),
        }
    }

    /// Creates a regular polygon with n sides.
    pub fn regular(n: usize, radius: f32, color: Vec4) -> Self {
        let mut vertices = Vec::with_capacity(n);
        for i in 0..n {
            let t = (i as f32 / n as f32) * std::f32::consts::TAU;
            vertices.push(glam::vec2(radius * t.cos(), radius * t.sin()));
        }
        Self::new(vertices, color)
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

impl Project for Polygon {
    fn project(&self, ctx: &mut ProjectionCtx) {
        // Fill
        if let Some(fill) = &self.style.fill {
            let mesh = Mesh::polygon(self.vertices.clone(), fill.clone());
            ctx.emit(RenderPrimitive::Mesh(mesh));
        }

        // Stroke
        if let Some(stroke) = &self.style.stroke {
            let n = self.vertices.len();
            if n >= 2 {
                for i in 0..n {
                    let j = (i + 1) % n;
                    ctx.emit(RenderPrimitive::Line {
                        start: vec3(self.vertices[i].x, self.vertices[i].y, 0.0),
                        end: vec3(self.vertices[j].x, self.vertices[j].y, 0.0),
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
}

impl Bounded for Polygon {
    fn local_bounds(&self) -> Bounds {
        if self.vertices.is_empty() {
            return Bounds::default();
        }
        let mut min = glam::vec2(f32::MAX, f32::MAX);
        let mut max = glam::vec2(f32::MIN, f32::MIN);
        for p in &self.vertices {
            min = min.min(*p);
            max = max.max(*p);
        }
        Bounds::new(min, max)
    }
}
