use crate::frontend::layout::{Bounded, Bounds};
use crate::frontend::style::{StrokeParams, Style};
use crate::projection::Mesh;
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};
use glam::{Vec2, Vec4, vec2, vec3};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatBubbleTipSide {
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub struct ChatBubble {
    pub width: f32,
    pub height: f32,
    pub radius: f32,
    pub corner_segments: usize,
    pub tip_side: ChatBubbleTipSide,
    pub tip_width: f32,
    pub tip_height: f32,
    pub tip_inset: f32,
    pub style: Style,
}

impl ChatBubble {
    pub fn new(width: f32, height: f32, radius: f32, color: Vec4) -> Self {
        Self {
            width,
            height,
            radius,
            corner_segments: 8,
            tip_side: ChatBubbleTipSide::Left,
            tip_width: 0.42,
            tip_height: 0.28,
            tip_inset: 0.52,
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

    pub fn with_tip(mut self, side: ChatBubbleTipSide, width: f32, height: f32) -> Self {
        self.tip_side = side;
        self.tip_width = width;
        self.tip_height = height;
        self
    }

    pub fn with_tip_inset(mut self, inset: f32) -> Self {
        self.tip_inset = inset;
        self
    }

    fn outline(&self) -> Vec<Vec2> {
        chat_bubble_outline(
            self.width,
            self.height,
            self.radius,
            self.corner_segments,
            self.tip_side,
            self.tip_width,
            self.tip_height,
            self.tip_inset,
        )
    }
}

impl Project for ChatBubble {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let outline = self.outline();

        if let Some(fill) = &self.style.fill {
            ctx.emit(RenderPrimitive::Mesh(Mesh::polygon(
                outline.clone(),
                fill.clone(),
            )));
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

impl Bounded for ChatBubble {
    fn local_bounds(&self) -> Bounds {
        let half_width = self.width.abs() * 0.5;
        let half_height = self.height.abs() * 0.5;
        Bounds::new(
            vec2(-half_width, -half_height - self.tip_height.abs()),
            vec2(half_width, half_height),
        )
    }
}

fn chat_bubble_outline(
    width: f32,
    height: f32,
    radius: f32,
    corner_segments: usize,
    tip_side: ChatBubbleTipSide,
    tip_width: f32,
    tip_height: f32,
    tip_inset: f32,
) -> Vec<Vec2> {
    let half = vec2(width.abs() * 0.5, height.abs() * 0.5);
    let r = radius.abs().max(0.01).min(half.x.min(half.y));
    let bottom = -half.y;
    let top = half.y;
    let left = -half.x;
    let right = half.x;
    let corner_segments = corner_segments.max(1);

    let base_half = (tip_width.abs() * 0.5).min((half.x - r).max(0.0) * 0.5);
    let min_center = left + r + base_half;
    let max_center = right - r - base_half;
    let raw_center = match tip_side {
        ChatBubbleTipSide::Left => left + tip_inset.abs(),
        ChatBubbleTipSide::Right => right - tip_inset.abs(),
    };
    let tip_center = raw_center.clamp(min_center, max_center);
    let left_base = vec2(tip_center - base_half, bottom);
    let right_base = vec2(tip_center + base_half, bottom);
    let tip_point = vec2(tip_center, bottom - tip_height.abs());

    let mut points = Vec::with_capacity(corner_segments * 4 + 8);
    points.push(right_base);
    points.push(vec2(right - r, bottom));
    add_arc(
        &mut points,
        vec2(right - r, bottom + r),
        r,
        -std::f32::consts::FRAC_PI_2,
        0.0,
        corner_segments,
    );
    points.push(vec2(right, top - r));
    add_arc(
        &mut points,
        vec2(right - r, top - r),
        r,
        0.0,
        std::f32::consts::FRAC_PI_2,
        corner_segments,
    );
    points.push(vec2(left + r, top));
    add_arc(
        &mut points,
        vec2(left + r, top - r),
        r,
        std::f32::consts::FRAC_PI_2,
        std::f32::consts::PI,
        corner_segments,
    );
    points.push(vec2(left, bottom + r));
    add_arc(
        &mut points,
        vec2(left + r, bottom + r),
        r,
        std::f32::consts::PI,
        std::f32::consts::PI * 1.5,
        corner_segments,
    );
    points.push(left_base);
    points.push(tip_point);
    dedup_consecutive(points)
}

fn add_arc(
    points: &mut Vec<Vec2>,
    center: Vec2,
    radius: f32,
    start: f32,
    end: f32,
    segments: usize,
) {
    for step in 0..=segments {
        let t = step as f32 / segments as f32;
        let angle = start + (end - start) * t;
        points.push(center + vec2(angle.cos() * radius, angle.sin() * radius));
    }
}

fn dedup_consecutive(points: Vec<Vec2>) -> Vec<Vec2> {
    let mut deduped = Vec::with_capacity(points.len());
    for point in points {
        if deduped
            .last()
            .is_none_or(|previous: &Vec2| previous.distance_squared(point) > 1e-8)
        {
            deduped.push(point);
        }
    }
    deduped
}
