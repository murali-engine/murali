use glam::{Vec2, Vec3, Vec4, vec2};

use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Mesh, Project, ProjectionCtx, RenderPrimitive};

#[derive(Debug, Clone)]
pub struct AttentionMatrix {
    pub values: Vec<Vec<f32>>,
    pub tokens: Option<Vec<String>>,
    pub cell_size: f32,
    pub low_color: Vec4,
    pub high_color: Vec4,
    pub grid_color: Vec4,
    pub grid_thickness: f32,
    pub label_height: f32,
}

impl AttentionMatrix {
    pub fn new(values: Vec<Vec<f32>>, tokens: Option<Vec<String>>) -> Self {
        Self {
            values,
            tokens,
            cell_size: 0.38,
            low_color: Vec4::new(0.14, 0.18, 0.27, 1.0),
            high_color: Vec4::new(0.20, 0.82, 0.88, 1.0),
            grid_color: Vec4::new(0.88, 0.92, 0.96, 1.0),
            grid_thickness: 0.015,
            label_height: 0.20,
        }
    }

    fn mix(&self, t: f32) -> Vec4 {
        let t = t.clamp(0.0, 1.0);
        self.low_color.lerp(self.high_color, t)
    }
}

impl Project for AttentionMatrix {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let rows = self.values.len();
        let cols = self.values.iter().map(|r| r.len()).max().unwrap_or(0);
        let width = cols as f32 * self.cell_size;
        let height = rows as f32 * self.cell_size;
        let left = -width * 0.5;
        let top = height * 0.5;

        for (r, row) in self.values.iter().enumerate() {
            for (c, value) in row.iter().enumerate() {
                let cx = left + c as f32 * self.cell_size + self.cell_size * 0.5;
                let cy = top - r as f32 * self.cell_size - self.cell_size * 0.5;
                let mesh = Mesh::square(self.cell_size * 0.96, self.mix(*value))
                    .as_ref()
                    .translated(Vec3::new(cx, cy, 0.0));
                ctx.emit(RenderPrimitive::Mesh(mesh));
            }
        }

        for c in 0..=cols {
            let x = left + c as f32 * self.cell_size;
            ctx.emit(RenderPrimitive::Line {
                start: Vec3::new(x, -height * 0.5, 0.0),
                end: Vec3::new(x, height * 0.5, 0.0),
                thickness: self.grid_thickness,
                color: self.grid_color,
                dash_length: 0.0,
                gap_length: 0.0,
                dash_offset: 0.0,
            });
        }
        for r in 0..=rows {
            let y = top - r as f32 * self.cell_size;
            ctx.emit(RenderPrimitive::Line {
                start: Vec3::new(left, y, 0.0),
                end: Vec3::new(left + width, y, 0.0),
                thickness: self.grid_thickness,
                color: self.grid_color,
                dash_length: 0.0,
                gap_length: 0.0,
                dash_offset: 0.0,
            });
        }

        if let Some(tokens) = &self.tokens {
            let max_label_width = tokens
                .iter()
                .map(|t| {
                    crate::resource::text::layout::measure_label(t, self.label_height, None).width
                })
                .fold(0.0, f32::max);

            for (idx, token) in tokens.iter().enumerate().take(cols) {
                let layout =
                    crate::resource::text::layout::measure_label(token, self.label_height, None);
                let x = left + idx as f32 * self.cell_size + self.cell_size * 0.5;
                ctx.emit(RenderPrimitive::Text {
                    content: token.clone(),
                    height: self.label_height,
                    color: self.grid_color,
                    font_name: None,
                    offset: Vec3::new(x, top + layout.width * 0.5 + self.label_height * 0.4, 0.0),
                    rotation: std::f32::consts::PI / 2.0,
                });
            }
            for (idx, token) in tokens.iter().enumerate().take(rows) {
                let y = top - idx as f32 * self.cell_size - self.cell_size * 0.5;
                ctx.emit(RenderPrimitive::Text {
                    content: token.clone(),
                    height: self.label_height,
                    color: self.grid_color,
                    font_name: None,
                    offset: Vec3::new(left - max_label_width * 0.5 - self.cell_size * 0.25, y, 0.0),
                    rotation: 0.0,
                });
            }
        }
    }
}

impl Bounded for AttentionMatrix {
    fn local_bounds(&self) -> Bounds {
        let rows = self.values.len();
        let cols = self.values.iter().map(|r| r.len()).max().unwrap_or(0);
        let label_pad = if self.tokens.is_some() {
            self.cell_size * 1.4
        } else {
            0.0
        };
        Bounds::from_center_size(
            Vec2::ZERO,
            vec2(
                cols as f32 * self.cell_size + label_pad * 1.5,
                rows as f32 * self.cell_size + label_pad * 1.2,
            ),
        )
    }
}
