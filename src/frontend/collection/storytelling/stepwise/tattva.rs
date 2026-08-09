// src/frontend/collection/storytelling/stepwise/tattva.rs
//! `Stepwise` — a first-class Murali tattva.

use glam::{Vec2, Vec3, Vec4};

use crate::frontend::animation::Ease;
use crate::frontend::collection::primitives::path::Path;
use crate::frontend::collection::text::label::Label;
use crate::frontend::layout::{Bounded, Bounds};
use crate::frontend::style::{ColorSource, StrokeParams, Style};
use crate::projection::{Project, ProjectionCtx};

use super::layout::{StepwiseDirection, StepwiseLayout};
use super::model::{Direction, StepwiseModel};
use super::script::ScriptBuilder;
use super::state::{StepState, TransitionState};
use super::timeline::TimelineEngine;

// ── Stepwise tattva ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct StepwiseStyle {
    pub node_height: f32,
    pub stroke_thickness: f32,
    pub label_height: f32,
    pub signal_radius: f32,
    pub edge_thickness: f32,
    pub pulse_scale: f32,
    pub corner_radius: f32,
    pub fill_active: Vec4,
    pub fill_completed: Vec4,
    pub stroke_active: Vec4,
    pub stroke_completed: Vec4,
    pub label_color: Vec4,
    pub signal_color: Vec4,
    pub edge_color: Vec4,
    pub debug_color: Vec4,
}

impl Default for StepwiseStyle {
    fn default() -> Self {
        Self {
            node_height: 1.2,
            stroke_thickness: 0.04,
            label_height: 0.3,
            signal_radius: 0.1,
            edge_thickness: 0.04,
            pulse_scale: 0.15,
            corner_radius: 0.15,
            fill_active: Vec4::new(0.12, 0.18, 0.30, 1.0),
            fill_completed: Vec4::new(0.18, 0.22, 0.28, 1.0),
            stroke_active: Vec4::new(0.35, 0.70, 1.00, 1.0),
            stroke_completed: Vec4::new(0.40, 0.50, 0.60, 1.0),
            label_color: Vec4::new(0.95, 0.97, 1.00, 1.0),
            signal_color: Vec4::new(1.0, 0.82, 0.25, 1.0),
            edge_color: Vec4::new(0.35, 0.45, 0.60, 1.0),
            debug_color: Vec4::new(1.0, 1.0, 0.5, 1.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Stepwise {
    pub model: StepwiseModel,
    pub progress: f32,
    pub signal_progress: f32,
    pub layout: StepwiseLayout,
    pub style: StepwiseStyle,
    pub debug: bool,
}

impl Stepwise {
    pub fn new(model: StepwiseModel) -> Self {
        Self {
            model,
            progress: 0.0,
            signal_progress: 0.0,
            layout: StepwiseLayout::default(),
            style: StepwiseStyle::default(),
            debug: false,
        }
    }

    pub fn from_script<F: FnOnce(&mut ScriptBuilder)>(f: F) -> Self {
        let mut builder = ScriptBuilder::new();
        f(&mut builder);
        Self::new(builder.build())
    }

    pub fn with_progress(mut self, p: f32) -> Self {
        self.progress = p.clamp(0.0, 1.0);
        self
    }

    pub fn with_reveal_progress(self, p: f32) -> Self {
        self.with_progress(p)
    }

    pub fn with_signal_progress(mut self, p: f32) -> Self {
        self.signal_progress = p.clamp(0.0, 1.0);
        self
    }

    pub fn with_journey_progress(self, p: f32) -> Self {
        self.with_signal_progress(p)
    }

    pub fn with_layout(mut self, layout: StepwiseLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn with_style(mut self, style: StepwiseStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Computes signal intensity (0.0 to 1.0) and visited status for a node
    fn node_signal_state(&self, node_idx: usize) -> (f32, bool) {
        if self.signal_progress < 0.001 {
            return (0.0, false);
        }

        let n = self.model.sequence.len();
        if n == 0 {
            return (0.0, false);
        }

        let total_segments = (2 * n - 1) as f32;
        let raw_pos = self.signal_progress * total_segments;
        let segment = (raw_pos.floor() as usize).min(2 * n - 2);
        let segment_t = (raw_pos - segment as f32).clamp(0.0, 1.0);

        let mut intensity = 0.0;
        let mut visited = false;

        for h in 0..n {
            let s_idx = self.model.sequence[h];
            if s_idx == node_idx {
                let pulse_segment = h * 2;

                // Determine visitation
                if segment >= pulse_segment {
                    visited = true;
                }

                // Determine pulse intensity (during its specific even segment)
                if segment == pulse_segment {
                    // Triangle pulse peaked at 0.5
                    intensity = if segment_t < 0.5 {
                        segment_t * 2.0
                    } else {
                        2.0 - segment_t * 2.0
                    };
                    intensity = Ease::InOutSmooth.eval(intensity);
                }
            }
        }

        (intensity, visited)
    }

    fn is_transition_signaled(&self, transition_idx: usize) -> bool {
        if self.signal_progress < 0.001 {
            return false;
        }

        let transition = &self.model.transitions[transition_idx];
        let n = self.model.sequence.len();
        if n < 2 {
            return false;
        }

        let total_segments = (2 * n - 1) as f32;
        let raw_pos = self.signal_progress * total_segments;
        let current_segment = raw_pos.floor() as usize;

        for h in 0..(n - 1) {
            let from = self.model.sequence[h];
            let to = self.model.sequence[h + 1];
            if from == transition.from && to == transition.to {
                let transit_segment = h * 2 + 1;
                if current_segment >= transit_segment {
                    return true;
                }
            }
        }
        false
    }
}

// ── internal rendering helpers using primitives ─────────────────────────────────

fn node_size_for(label: &str, style: &StepwiseStyle) -> glam::Vec2 {
    let layout = crate::resource::text::layout::measure_label(label, style.label_height, None);
    let min_width = style.node_height;
    let width = (layout.width + 0.6).max(min_width);
    glam::vec2(width, style.node_height)
}

fn rounded_rect_path(size: glam::Vec2, radius: f32, color: glam::Vec4) -> Path {
    let hw = size.x * 0.5;
    let hh = size.y * 0.5;
    let r = radius.min(hw).min(hh);

    let mut path = Path::new();
    path.style.fill = Some(ColorSource::Solid(color));
    path.closed = true;

    // Start at top-left after corner
    path = path
        .move_to(glam::vec2(-hw + r, -hh))
        .line_to(glam::vec2(hw - r, -hh))
        .quad_to(glam::vec2(hw, -hh), glam::vec2(hw, -hh + r))
        .line_to(glam::vec2(hw, hh - r))
        .quad_to(glam::vec2(hw, hh), glam::vec2(hw - r, hh))
        .line_to(glam::vec2(-hw + r, hh))
        .quad_to(glam::vec2(-hw, hh), glam::vec2(-hw, hh - r))
        .line_to(glam::vec2(-hw, -hh + r))
        .quad_to(glam::vec2(-hw, -hh), glam::vec2(-hw + r, -hh));

    path
}

fn render_node_path(
    ctx: &mut ProjectionCtx,
    stepwise_style: &StepwiseStyle,
    size: glam::Vec2,
    trim: f32,
    fill_alpha: f32,
    stroke_color: Vec4,
    fill_color: Vec4,
) {
    let mut style = Style::new();
    style.stroke = Some(StrokeParams {
        thickness: stepwise_style.stroke_thickness,
        color: stroke_color,
        ..Default::default()
    });
    style.fill = Some(ColorSource::Solid(fill_color));

    let mut path = rounded_rect_path(size, stepwise_style.corner_radius, fill_color);
    path.style = style;

    path.trim_end = trim;
    path.fill_opacity = fill_alpha;

    path.project(ctx);
}

fn render_default_node(
    ctx: &mut ProjectionCtx,
    style: &StepwiseStyle,
    pos: Vec3,
    state: &StepState,
    label: &str,
    scale: f32,
    is_visited: bool,
) {
    let size = node_size_for(label, style) * scale;
    ctx.with_offset(pos, |ctx| {
        let (stroke, fill) = (style.stroke_active, style.fill_active);

        match state {
            StepState::Pending => {}
            StepState::Active { t } => {
                let t = Ease::InOutSmooth.eval(*t);
                let outline_t = (t * 2.0).clamp(0.0, 1.0);
                let fill_t = ((t - 0.5) * 2.0).clamp(0.0, 1.0);

                render_node_path(ctx, style, size, outline_t, fill_t, stroke, fill);

                if fill_t > 0.001 {
                    ctx.with_offset(Vec3::new(0.0, 0.0, 0.1), |ctx| {
                        Label::new(label, style.label_height * scale)
                            .with_color(Vec4::new(
                                style.label_color.x,
                                style.label_color.y,
                                style.label_color.z,
                                style.label_color.w * fill_t,
                            ))
                            .with_char_reveal(fill_t)
                            .project(ctx);
                    });
                }
            }
            StepState::Completed => {
                let s_color = if is_visited {
                    style.stroke_active
                } else {
                    style.stroke_completed
                };
                let f_color = if is_visited {
                    style.fill_active
                } else {
                    style.fill_completed
                };

                render_node_path(ctx, style, size, 1.0, 1.0, s_color, f_color);

                ctx.with_offset(Vec3::new(0.0, 0.0, 0.1), |ctx| {
                    Label::new(label, style.label_height * scale)
                        .with_color(style.label_color)
                        .project(ctx);
                });
            }
        }
    });
}

fn render_background(
    ctx: &mut ProjectionCtx,
    style: &StepwiseStyle,
    pos: Vec3,
    state: &StepState,
    label: &str,
    scale: f32,
    is_visited: bool,
) {
    let size = node_size_for(label, style) * scale;
    ctx.with_offset(pos, |ctx| {
        let (stroke, fill) = (style.stroke_active, style.fill_active);

        match state {
            StepState::Pending => {}
            StepState::Active { t } => {
                let t = Ease::InOutSmooth.eval(*t);
                let outline_t = (t * 2.0).clamp(0.0, 1.0);
                let fill_t = ((t - 0.5) * 2.0).clamp(0.0, 1.0);
                render_node_path(ctx, style, size, outline_t, fill_t, stroke, fill);
            }
            StepState::Completed => {
                let s_color = if is_visited {
                    style.stroke_active
                } else {
                    style.stroke_completed
                };
                let f_color = if is_visited {
                    style.fill_active
                } else {
                    style.fill_completed
                };
                render_node_path(ctx, style, size, 1.0, 1.0, s_color, f_color);
            }
        }
    });
}

// ── Project ───────────────────────────────────────────────────────────────────

impl Project for Stepwise {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let state = TimelineEngine::compute(&self.model, self.progress);
        let style = &self.style;

        // 1. Precalculate node sizes and positions based on actual labels
        let mut node_sizes = Vec::with_capacity(self.model.steps.len());
        let mut node_positions = Vec::with_capacity(self.model.steps.len());

        let mut cursor = 0.0;
        let gap = self.layout.spacing; // Repurpose spacing as "Gap"

        for (i, step) in self.model.steps.iter().enumerate() {
            let size = node_size_for(&step.label, style);
            node_sizes.push(size);

            if i == 0 {
                cursor = 0.0;
            } else {
                let prev_size = node_sizes[i - 1];
                let dist = match self.layout.direction {
                    StepwiseDirection::Horizontal => prev_size.x * 0.5 + gap + size.x * 0.5,
                    StepwiseDirection::Vertical => prev_size.y * 0.5 + gap + size.y * 0.5,
                };
                cursor += dist;
            }

            let pos = match self.layout.direction {
                StepwiseDirection::Horizontal => Vec3::new(cursor, 0.0, 0.0),
                StepwiseDirection::Vertical => Vec3::new(0.0, -cursor, 0.0),
            };
            node_positions.push(pos);
        }

        // ── transitions ───────────────────────────────────────────────────
        for (i, transition) in self.model.transitions.iter().enumerate() {
            let from_idx = transition.from;
            let to_idx = transition.to;

            let from_raw = node_positions[from_idx];
            let to_raw = node_positions[to_idx];
            let from_size = node_sizes[from_idx];
            let to_size = node_sizes[to_idx];

            // Clip edges to node boundaries (Rectangular clipping)
            let (from, to) = clip_edge_rect(from_raw, to_raw, from_size, to_size, 0.05);
            let transition = &self.model.transitions[i];

            match &state.transitions[i] {
                TransitionState::Hidden => {}
                TransitionState::Drawing { t } => {
                    let mut path = Path::new()
                        .with_thickness(style.edge_thickness)
                        .with_color(style.edge_color);

                    if let Some(route) = &transition.route {
                        let points = self.calculate_route_points(
                            transition.from,
                            transition.to,
                            route,
                            &node_positions,
                            &node_sizes,
                        );
                        path = path.move_to(points[0].truncate());
                        for p in &points[1..] {
                            path = path.line_to(p.truncate());
                        }
                    } else {
                        path = path.move_to(from.truncate()).line_to(to.truncate());
                    }

                    path.trim_end = Ease::InOutSmooth.eval(*t);
                    path.project(ctx);
                }
                TransitionState::Completed => {
                    let is_signaled = self.is_transition_signaled(i);
                    let color = if is_signaled {
                        style.stroke_active
                    } else {
                        Vec4::new(
                            style.edge_color.x,
                            style.edge_color.y,
                            style.edge_color.z,
                            style.edge_color.w * 0.6,
                        )
                    };

                    let mut path = Path::new()
                        .with_thickness(style.edge_thickness)
                        .with_color(color);

                    if let Some(route) = &transition.route {
                        let points = self.calculate_route_points(
                            transition.from,
                            transition.to,
                            route,
                            &node_positions,
                            &node_sizes,
                        );
                        path = path.move_to(points[0].truncate());
                        for p in &points[1..] {
                            path = path.line_to(p.truncate());
                        }
                    } else {
                        path = path.move_to(from.truncate()).line_to(to.truncate());
                    }
                    path.project(ctx);
                }
            }
        }

        // ── nodes ─────────────────────────────────────────────────────────
        for (i, step) in self.model.steps.iter().enumerate() {
            let step_state = &state.steps[i];
            let pos = node_positions[i];

            // Determine signal-based indication for this node
            let (signal_intensity, is_visited) = self.node_signal_state(i);
            let scale_mod = 1.0 + style.pulse_scale * signal_intensity;

            ctx.with_offset(pos, |ctx| match &step.content {
                None => render_default_node(
                    ctx,
                    style,
                    Vec3::ZERO,
                    step_state,
                    &step.label,
                    scale_mod,
                    is_visited,
                ),
                Some(content) => {
                    if !content.draws_own_background() {
                        render_background(
                            ctx,
                            style,
                            Vec3::ZERO,
                            step_state,
                            &step.label,
                            scale_mod,
                            is_visited,
                        );
                    }
                    ctx.with_offset(Vec3::ZERO, |ctx| {
                        if signal_intensity > 0.001 {
                            content.project_indicated(ctx, step_state, signal_intensity);
                        } else {
                            content.project(ctx, step_state);
                        }
                    });
                }
            });
        }

        // ── signal dot ────────────────────────────────────────────────────
        if self.signal_progress > 0.001 {
            let n = self.model.sequence.len();
            if n >= 1 {
                let total_segments = (2 * n - 1) as f32;
                let raw_pos = self.signal_progress.clamp(0.0, 1.0) * total_segments;
                let segment = (raw_pos.floor() as usize).min(2 * n - 2);
                let segment_t = (raw_pos - segment as f32).clamp(0.0, 1.0);

                // Even segment: Node Pulse (Node i = segment / 2)
                // Odd segment:  Transit (h = (segment - 1) / 2)
                if segment % 2 == 1 && n >= 2 {
                    let hop = (segment - 1) / 2;
                    let from_idx = self.model.sequence[hop];
                    let to_idx = self.model.sequence[hop + 1];

                    let from_pos = node_positions[from_idx];
                    let to_pos = node_positions[to_idx];

                    // Find the transition for this hop to check for routing
                    let transition = self
                        .model
                        .transitions
                        .iter()
                        .find(|t| t.from == from_idx && t.to == to_idx);

                    let p = if let Some(t_obj) = transition {
                        if let Some(route) = &t_obj.route {
                            let points = self.calculate_route_points(
                                from_idx,
                                to_idx,
                                route,
                                &node_positions,
                                &node_sizes,
                            );
                            let (p, _) =
                                sample_polyline(&points, Ease::InOutSmooth.eval(segment_t));
                            p
                        } else {
                            let (from, to) = clip_edge_rect(
                                from_pos,
                                to_pos,
                                node_sizes[from_idx],
                                node_sizes[to_idx],
                                0.0,
                            );
                            from.lerp(to, Ease::InOutSmooth.eval(segment_t))
                        }
                    } else {
                        let (from, to) = clip_edge_rect(
                            from_pos,
                            to_pos,
                            node_sizes[from_idx],
                            node_sizes[to_idx],
                            0.0,
                        );
                        from.lerp(to, Ease::InOutSmooth.eval(segment_t))
                    };

                    ctx.with_offset(Vec3::new(p.x, p.y, 0.2), |ctx| {
                        crate::frontend::collection::primitives::circle::Circle::new(
                            style.signal_radius,
                            20,
                            style.signal_color,
                        )
                        .project(ctx);
                    });
                }
            }
        }

        // ── debug overlay ─────────────────────────────────────────────────
        if self.debug {
            let base_y = 3.0_f32;
            let step_y = -0.5_f32;
            let x = -8.0_f32;

            for (i, step) in self.model.steps.iter().enumerate() {
                let text = match &state.steps[i] {
                    StepState::Completed => format!("[✓] {}", step.label),
                    StepState::Active { t } => format!("[→] {} ({:.2})", step.label, t),
                    StepState::Pending => format!("[ ] {}", step.label),
                };
                ctx.with_offset(Vec3::new(x, base_y + i as f32 * step_y, 0.5), |ctx| {
                    Label::new(text, 0.25)
                        .with_color(style.debug_color)
                        .project(ctx);
                });
            }
        }
    }
}

/// Ray-rectangle intersection.
/// Returns clipped (start, end) points.
fn clip_edge_rect(
    from: Vec3,
    to: Vec3,
    from_size: Vec2,
    to_size: Vec2,
    padding: f32,
) -> (Vec3, Vec3) {
    let dir = to - from;
    let dist = dir.length();
    if dist < 1e-6 {
        return (from, to);
    }
    let d = dir / dist;

    let get_t = |dir: Vec3, size: Vec2| {
        let mag_x = dir.x.abs();
        let mag_y = dir.y.abs();

        let tx = if mag_x > 1e-6 {
            (size.x * 0.5 + padding) / mag_x
        } else {
            f32::INFINITY
        };
        let ty = if mag_y > 1e-6 {
            (size.y * 0.5 + padding) / mag_y
        } else {
            f32::INFINITY
        };

        tx.min(ty)
    };

    let t_start = get_t(d, from_size);
    let t_end = get_t(-d, to_size);

    (from + d * t_start, to - d * t_end)
}

impl Stepwise {
    fn calculate_route_points(
        &self,
        from_idx: usize,
        to_idx: usize,
        route: &[Direction],
        node_positions: &[Vec3],
        node_sizes: &[Vec2],
    ) -> Vec<Vec3> {
        let from_pos = node_positions[from_idx];
        let to_pos = node_positions[to_idx];
        let from_size = node_sizes[from_idx];
        let to_size = node_sizes[to_idx];
        let z = from_pos.z;
        let n = node_positions.len();

        if route.is_empty() {
            let (a, b) = clip_edge_rect(from_pos, to_pos, from_size, to_size, 0.05);
            return vec![a, b];
        }

        let lane_margin = self.layout.spacing * 0.8;
        let low_idx = from_idx.min(to_idx);
        let high_idx = from_idx.max(to_idx);
        let spanned_sizes = &node_sizes[low_idx..=high_idx];
        let max_spanned_half_w = spanned_sizes
            .iter()
            .map(|size| size.x * 0.5)
            .fold(0.0_f32, f32::max);
        let max_spanned_half_h = spanned_sizes
            .iter()
            .map(|size| size.y * 0.5)
            .fold(0.0_f32, f32::max);
        let side_lane_x = |go_right: bool| {
            if go_right {
                from_pos.x.max(to_pos.x) + max_spanned_half_w + lane_margin
            } else {
                from_pos.x.min(to_pos.x) - max_spanned_half_w - lane_margin
            }
        };
        let side_lane_y = |go_up: bool| {
            if go_up {
                from_pos.y.max(to_pos.y) + max_spanned_half_h + lane_margin
            } else {
                from_pos.y.min(to_pos.y) - max_spanned_half_h - lane_margin
            }
        };

        // 1. Start anchor — intelligent: exit face from first direction
        let start = match route[0] {
            Direction::Up => Vec3::new(from_pos.x, from_pos.y + from_size.y * 0.5, z),
            Direction::Down => Vec3::new(from_pos.x, from_pos.y - from_size.y * 0.5, z),
            Direction::Left => Vec3::new(from_pos.x - from_size.x * 0.5, from_pos.y, z),
            Direction::Right => Vec3::new(from_pos.x + from_size.x * 0.5, from_pos.y, z),
        };

        // 2. Walk directions. The primary layout axis steps across node ranks,
        // while the secondary axis moves through routing lanes.
        let mut points = vec![start];
        let mut virtual_idx = from_idx as isize;
        let mut cur_x = start.x;
        let mut cur_y = start.y;

        for dir in route {
            match self.layout.direction {
                StepwiseDirection::Horizontal => match dir {
                    Direction::Left => {
                        virtual_idx -= 1;
                        let clamped = virtual_idx.clamp(0, n as isize - 1) as usize;
                        cur_x = node_positions[clamped].x;
                        points.push(Vec3::new(cur_x, cur_y, z));
                    }
                    Direction::Right => {
                        virtual_idx += 1;
                        let clamped = virtual_idx.clamp(0, n as isize - 1) as usize;
                        cur_x = node_positions[clamped].x;
                        points.push(Vec3::new(cur_x, cur_y, z));
                    }
                    Direction::Up => {
                        cur_y = side_lane_y(true);
                        points.push(Vec3::new(cur_x, cur_y, z));
                    }
                    Direction::Down => {
                        cur_y = side_lane_y(false);
                        points.push(Vec3::new(cur_x, cur_y, z));
                    }
                },
                StepwiseDirection::Vertical => match dir {
                    Direction::Up => {
                        virtual_idx -= 1;
                        let clamped = virtual_idx.clamp(0, n as isize - 1) as usize;
                        cur_y = node_positions[clamped].y;
                        points.push(Vec3::new(cur_x, cur_y, z));
                    }
                    Direction::Down => {
                        virtual_idx += 1;
                        let clamped = virtual_idx.clamp(0, n as isize - 1) as usize;
                        cur_y = node_positions[clamped].y;
                        points.push(Vec3::new(cur_x, cur_y, z));
                    }
                    Direction::Left => {
                        cur_x = side_lane_x(false);
                        points.push(Vec3::new(cur_x, cur_y, z));
                    }
                    Direction::Right => {
                        cur_x = side_lane_x(true);
                        points.push(Vec3::new(cur_x, cur_y, z));
                    }
                },
            }
        }

        // 3. Spatial arrival — look at where last_p ended up vs target bounding box
        let last_p = *points.last().unwrap();
        let half_w = to_size.x * 0.5;
        let half_h = to_size.y * 0.5;
        let dx = last_p.x - to_pos.x;
        let dy = last_p.y - to_pos.y;

        let end = if dy.abs() > half_h + 0.05 {
            // Clearly above or below → enter top or bottom face
            let face_y = if dy > 0.0 {
                to_pos.y + half_h
            } else {
                to_pos.y - half_h
            };
            Vec3::new(to_pos.x, face_y, z)
        } else if dx.abs() > half_w + 0.05 {
            // Clearly left or right → enter side face
            let face_x = if dx > 0.0 {
                to_pos.x + half_w
            } else {
                to_pos.x - half_w
            };
            Vec3::new(face_x, to_pos.y, z)
        } else {
            // Fallback: closest face
            if dy.abs() >= dx.abs() {
                Vec3::new(to_pos.x, to_pos.y + half_h * dy.signum(), z)
            } else {
                Vec3::new(to_pos.x + half_w * dx.signum(), to_pos.y, z)
            }
        };

        // 4. Manhattan elbow to reach end cleanly
        if (last_p.x - end.x).abs() > 0.001 && (last_p.y - end.y).abs() > 0.001 {
            let arrives_top_or_bottom = (end.x - to_pos.x).abs() < 0.001;
            if arrives_top_or_bottom {
                points.push(Vec3::new(end.x, last_p.y, z)); // align X first, then drop
            } else {
                points.push(Vec3::new(last_p.x, end.y, z)); // align Y first, then go sideways
            }
        }

        points.push(end);

        // 5. Dedup zero-length segments
        let mut out: Vec<Vec3> = Vec::new();
        for p in points {
            if out.last().map_or(true, |&prev| (p - prev).length() > 0.001) {
                out.push(p);
            }
        }
        out
    }
}

fn sample_polyline(points: &[Vec3], t: f32) -> (Vec3, Vec3) {
    if points.len() < 2 {
        return (points.get(0).copied().unwrap_or(Vec3::ZERO), Vec3::ZERO);
    }

    let mut total_len = 0.0;
    let mut segment_lens = Vec::with_capacity(points.len() - 1);
    for i in 0..points.len() - 1 {
        let l = (points[i + 1] - points[i]).length();
        total_len += l;
        segment_lens.push(l);
    }

    let target_len = t * total_len;
    let mut current_len = 0.0;

    for i in 0..points.len() - 1 {
        let l = segment_lens[i];
        if current_len + l >= target_len || i == points.len() - 2 {
            let local_t = if l < 1e-6 {
                0.0
            } else {
                (target_len - current_len) / l
            };
            let pos = points[i].lerp(points[i + 1], local_t);
            let dir = if l < 1e-6 {
                Vec3::ZERO
            } else {
                (points[i + 1] - points[i]) / l
            };
            return (pos, dir);
        }
        current_len += l;
    }

    (points[points.len() - 1], Vec3::ZERO)
}

// ── Bounded ───────────────────────────────────────────────────────────────────

impl Bounded for Stepwise {
    fn local_bounds(&self) -> Bounds {
        let n = self.model.steps.len();
        if n == 0 {
            return Bounds::from_center_size(glam::Vec2::ZERO, glam::vec2(0.01, 0.01));
        }
        let pad = self.style.node_height * 0.5 + self.style.signal_radius;
        let last = self.layout.position_for(n - 1);
        match self.layout.direction {
            StepwiseDirection::Horizontal => Bounds::new(
                glam::Vec2::new(-pad, -pad),
                glam::Vec2::new(last.x + pad, pad),
            ),
            StepwiseDirection::Vertical => Bounds::new(
                glam::Vec2::new(-pad, last.y - pad),
                glam::Vec2::new(pad, pad),
            ),
        }
    }
}
