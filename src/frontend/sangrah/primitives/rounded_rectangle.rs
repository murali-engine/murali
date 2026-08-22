use crate::frontend::layout::{Bounded, Bounds};
use crate::frontend::style::{StrokeParams, Style};
use crate::projection::Mesh;
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};
use glam::{Vec2, Vec4, vec2, vec3};

#[derive(Debug, Clone)]
pub struct RoundedRectangle {
    pub width: f32,
    pub height: f32,
    pub radius: f32,
    pub corner_segments: usize,
    pub style: Style,
}

impl RoundedRectangle {
    pub fn new(width: f32, height: f32, radius: f32, color: Vec4) -> Self {
        Self {
            width,
            height,
            radius,
            corner_segments: 8,
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

    pub fn with_radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn with_corner_segments(mut self, corner_segments: usize) -> Self {
        self.corner_segments = corner_segments.max(1);
        self
    }

    fn outline(&self) -> Vec<Vec2> {
        rounded_rect_outline(self.width, self.height, self.radius, self.corner_segments)
    }
}

impl Project for RoundedRectangle {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let outline = self.outline();

        if let Some(fill) = &self.style.fill {
            let mesh = Mesh::polygon(outline.clone(), fill.clone());
            ctx.emit(RenderPrimitive::Mesh(mesh));
        }

        if let Some(stroke) = &self.style.stroke {
            let n = outline.len();
            if n >= 2 {
                for i in 0..n {
                    let j = (i + 1) % n;
                    ctx.emit(RenderPrimitive::Line {
                        start: vec3(outline[i].x, outline[i].y, 0.0),
                        end: vec3(outline[j].x, outline[j].y, 0.0),
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

impl Bounded for RoundedRectangle {
    fn local_bounds(&self) -> Bounds {
        let hw = self.width * 0.5;
        let hh = self.height * 0.5;
        Bounds::new(vec2(-hw, -hh), vec2(hw, hh))
    }
}

fn rounded_rect_outline(width: f32, height: f32, radius: f32, corner_segments: usize) -> Vec<Vec2> {
    let half = vec2(width.abs() * 0.5, height.abs() * 0.5);
    let r = radius.abs().max(0.01).min(half.x.min(half.y));
    let centers = [
        vec2(half.x - r, half.y - r),
        vec2(-(half.x - r), half.y - r),
        vec2(-(half.x - r), -(half.y - r)),
        vec2(half.x - r, -(half.y - r)),
    ];
    let ranges = [
        (0.0, std::f32::consts::FRAC_PI_2),
        (std::f32::consts::FRAC_PI_2, std::f32::consts::PI),
        (std::f32::consts::PI, std::f32::consts::PI * 1.5),
        (std::f32::consts::PI * 1.5, std::f32::consts::TAU),
    ];

    let mut points = Vec::new();
    for (center, (start, end)) in centers.into_iter().zip(ranges) {
        for step in 0..=corner_segments.max(1) {
            let t = step as f32 / corner_segments.max(1) as f32;
            let angle = start + (end - start) * t;
            points.push(center + vec2(angle.cos() * r, angle.sin() * r));
        }
    }
    points
}
