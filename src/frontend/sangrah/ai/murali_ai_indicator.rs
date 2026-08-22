use crate::colors::*;
use crate::engine::scene::Scene;
use crate::engine::timeline::Timeline;
use crate::frontend::TattvaId;
use crate::frontend::animation::Ease;
use crate::frontend::sangrah::primitives::circle::Circle;
use crate::frontend::sangrah::primitives::line::Line;
use crate::frontend::sangrah::text::label::Label;
use glam::{Vec3, Vec4};

pub const MURALI_AI_INDICATOR_DURATION: f32 = 5.6;
pub const MURALI_AI_INDICATOR_LOOP_START: f32 = 2.78;
pub const MURALI_AI_INDICATOR_LOOP_CYCLE: f32 = 2.10;

#[derive(Debug, Clone)]
pub struct MuraliAiIndicator {
    pub title: String,
    pub subtitle: String,
    pub lower_claim: String,
    pub built_with: String,
}

#[derive(Debug, Clone)]
pub struct MuraliAiIndicatorIds {
    pub title: TattvaId,
    pub subtitle: TattvaId,
    pub outer: TattvaId,
    pub mid: TattvaId,
    pub inner: TattvaId,
    pub core_glow: TattvaId,
    pub core: TattvaId,
    pub core_label: TattvaId,
    pub spokes: Vec<TattvaId>,
    pub nodes: Vec<TattvaId>,
    pub signal: TattvaId,
    pub lower_claim: TattvaId,
    pub built_with: TattvaId,
    signal_path: Vec<Vec3>,
}

impl MuraliAiIndicator {
    pub fn new() -> Self {
        Self {
            title: "Murali AI".to_string(),
            subtitle: "intelligence is active".to_string(),
            lower_claim: "authored with Murali Engine".to_string(),
            built_with: "Built with Murali Engine".to_string(),
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = subtitle.into();
        self
    }

    pub fn with_lower_claim(mut self, lower_claim: impl Into<String>) -> Self {
        self.lower_claim = lower_claim.into();
        self
    }

    pub fn with_built_with(mut self, built_with: impl Into<String>) -> Self {
        self.built_with = built_with.into();
        self
    }

    pub fn add_to_scene(self, scene: &mut Scene, origin: Vec3) -> MuraliAiIndicatorIds {
        let cyan = Vec4::new(0.20, 0.84, 0.96, 1.0);
        let teal = TEAL_B;
        let blue = BLUE_B;
        let green = GREEN_B;
        let amber = GOLD_B;
        let soft_text = rgba(WHITE, 0.82);

        let title = add_label(
            scene,
            &self.title,
            0.46,
            soft_text,
            at(origin, 0.0, 2.98, 0.1),
        );
        let subtitle = add_label(
            scene,
            &self.subtitle,
            0.2,
            rgba(BLUE_A, 0.68),
            at(origin, 0.0, 2.55, 0.1),
        );

        let outer = add_ring(scene, origin, 1.86, 0.025, rgba(cyan, 0.34));
        let mid = add_ring(scene, origin, 1.32, 0.032, rgba(teal, 0.52));
        let inner = add_ring(scene, origin, 0.74, 0.045, rgba(cyan, 0.78));

        let core_glow = scene.add_tattva(
            Circle::new(0.56, 96, rgba(cyan, 0.12)).with_stroke(0.025, rgba(cyan, 0.54)),
            at(origin, 0.0, 0.25, 0.04),
        );
        let core = scene.add_tattva(
            Circle::new(0.26, 64, rgba(cyan, 0.92)).with_stroke(0.02, WHITE),
            at(origin, 0.0, 0.25, 0.12),
        );
        let core_label = add_label(
            scene,
            "AI",
            0.18,
            Vec4::new(0.03, 0.06, 0.08, 1.0),
            at(origin, 0.0, 0.25, 0.2),
        );

        let node_specs = [
            (Vec3::new(-1.55, 1.02, 0.1), teal),
            (Vec3::new(-1.20, -0.86, 0.1), blue),
            (Vec3::new(0.0, -1.36, 0.1), amber),
            (Vec3::new(1.20, -0.86, 0.1), green),
            (Vec3::new(1.55, 1.02, 0.1), cyan),
        ];
        let mut nodes = Vec::new();
        let mut spokes = Vec::new();
        for (pos, color) in node_specs {
            spokes.push(scene.add_tattva(
                Line::new(
                    at(Vec3::ZERO, 0.0, 0.25, 0.0),
                    at(Vec3::ZERO, pos.x, pos.y, 0.0),
                    0.018,
                    rgba(color, 0.38),
                ),
                origin,
            ));
            nodes.push(add_dot(scene, origin + pos, 0.075, rgba(color, 0.92)));
        }

        let signal = add_dot(
            scene,
            at(origin, -1.55, 1.02, 0.18),
            0.055,
            rgba(WHITE, 0.95),
        );

        let lower_claim = add_label(
            scene,
            &self.lower_claim,
            0.18,
            rgba(WHITE, 0.46),
            at(origin, 0.0, -2.72, 0.1),
        );
        let built_with = add_label(
            scene,
            &self.built_with,
            0.15,
            rgba(WHITE, 0.5),
            at(origin, 3.2, -3.4, 0.1),
        );
        let signal_path = [
            Vec3::new(-1.55, 1.02, 0.18),
            Vec3::new(1.55, 1.02, 0.18),
            Vec3::new(1.20, -0.86, 0.18),
            Vec3::new(0.0, -1.36, 0.18),
            Vec3::new(-1.20, -0.86, 0.18),
            Vec3::new(-1.55, 1.02, 0.18),
        ]
        .into_iter()
        .map(|pos| origin + pos)
        .collect();

        MuraliAiIndicatorIds {
            title,
            subtitle,
            outer,
            mid,
            inner,
            core_glow,
            core,
            core_label,
            spokes,
            nodes,
            signal,
            lower_claim,
            built_with,
            signal_path,
        }
    }
}

impl Default for MuraliAiIndicator {
    fn default() -> Self {
        Self::new()
    }
}

impl MuraliAiIndicatorIds {
    pub fn all(&self) -> Vec<TattvaId> {
        let mut ids = vec![
            self.title,
            self.subtitle,
            self.outer,
            self.mid,
            self.inner,
            self.core_glow,
            self.core,
            self.core_label,
            self.signal,
            self.lower_claim,
            self.built_with,
        ];
        ids.extend(self.spokes.iter().copied());
        ids.extend(self.nodes.iter().copied());
        ids
    }

    pub fn hide_all(&self, scene: &mut Scene) {
        for id in self.all() {
            scene.hide_tattva(id);
        }
    }

    pub fn animate(&self, timeline: &mut Timeline, loop_until: f32) {
        let loop_until = loop_until.max(MURALI_AI_INDICATOR_DURATION);

        write_text(timeline, &[self.title], 0.1, 0.7);
        write_text(timeline, &[self.subtitle], 0.62, 0.5);
        appear(timeline, &[self.built_with], 0.2, 0.6);

        draw(timeline, &[self.inner], 0.95, 0.5);
        appear(timeline, &[self.core_glow, self.core], 1.05, 0.42);
        write_text(timeline, &[self.core_label], 1.22, 0.25);

        draw(timeline, &[self.mid], 1.42, 0.55);
        draw(timeline, &[self.outer], 1.72, 0.65);

        for (index, &spoke) in self.spokes.iter().enumerate() {
            draw(timeline, &[spoke], 2.05 + index as f32 * 0.08, 0.24);
        }
        for (index, &node) in self.nodes.iter().enumerate() {
            appear(timeline, &[node], 2.18 + index as f32 * 0.1, 0.26);
        }

        appear(timeline, &[self.signal], 2.55, 0.2);
        self.loop_active(timeline, loop_until);

        write_text(timeline, &[self.lower_claim], 4.25, 0.58);
    }

    pub fn loop_active(&self, timeline: &mut Timeline, loop_until: f32) {
        let mut cycle_start = MURALI_AI_INDICATOR_LOOP_START;

        while cycle_start < loop_until {
            for (index, window) in self.signal_path.windows(2).enumerate() {
                let at_time = cycle_start + index as f32 * 0.34;
                if at_time >= loop_until {
                    break;
                }

                timeline
                    .animate(self.signal)
                    .at(at_time)
                    .for_duration(0.28)
                    .ease(Ease::InOutCubic)
                    .move_to(window[1])
                    .from_vec3(window[0])
                    .spawn();

                if let Some(&node) = self.nodes.get(index % self.nodes.len()) {
                    timeline
                        .animate(node)
                        .at(at_time + 0.08)
                        .for_duration(0.18)
                        .ease(Ease::OutCubic)
                        .scale_to(Vec3::splat(1.34))
                        .from_vec3(Vec3::ONE)
                        .spawn();
                    timeline
                        .animate(node)
                        .at(at_time + 0.26)
                        .for_duration(0.22)
                        .ease(Ease::InOutQuad)
                        .scale_to(Vec3::ONE)
                        .from_vec3(Vec3::splat(1.34))
                        .spawn();
                }
            }

            for (index, &ring) in [self.inner, self.mid, self.outer].iter().enumerate() {
                let at_time = cycle_start + 0.28 + index as f32 * 0.18;
                if at_time >= loop_until {
                    continue;
                }

                let high_scale = 1.10 + index as f32 * 0.08;
                timeline
                    .animate(ring)
                    .at(at_time)
                    .for_duration(0.58)
                    .ease(Ease::OutCubic)
                    .scale_to(Vec3::splat(high_scale))
                    .from_vec3(Vec3::ONE)
                    .spawn();
                timeline
                    .animate(ring)
                    .at(at_time + 0.58)
                    .for_duration(0.54)
                    .ease(Ease::InOutQuad)
                    .scale_to(Vec3::ONE)
                    .from_vec3(Vec3::splat(high_scale))
                    .spawn();
                timeline
                    .animate(ring)
                    .at(at_time + 0.18)
                    .for_duration(0.68)
                    .ease(Ease::OutCubic)
                    .fade_to(0.34)
                    .from(0.86)
                    .spawn();
                timeline
                    .animate(ring)
                    .at(at_time + 0.86)
                    .for_duration(0.34)
                    .ease(Ease::InOutQuad)
                    .fade_to(0.86)
                    .from(0.34)
                    .spawn();
            }

            for (index, &id) in [self.core_glow, self.core].iter().enumerate() {
                let at_time = cycle_start + 1.46 + index as f32 * 0.04;
                if at_time >= loop_until {
                    continue;
                }

                let high_scale = 1.18 - index as f32 * 0.06;
                timeline
                    .animate(id)
                    .at(at_time)
                    .for_duration(0.36)
                    .ease(Ease::OutCubic)
                    .scale_to(Vec3::splat(high_scale))
                    .from_vec3(Vec3::ONE)
                    .spawn();
                timeline
                    .animate(id)
                    .at(at_time + 0.36)
                    .for_duration(0.42)
                    .ease(Ease::InOutQuad)
                    .scale_to(Vec3::ONE)
                    .from_vec3(Vec3::splat(high_scale))
                    .spawn();
            }

            cycle_start += MURALI_AI_INDICATOR_LOOP_CYCLE;
        }
    }
}

fn rgba(color: Vec4, alpha: f32) -> Vec4 {
    Vec4::new(color.x, color.y, color.z, alpha)
}

fn at(origin: Vec3, x: f32, y: f32, z: f32) -> Vec3 {
    origin + Vec3::new(x, y, z)
}

fn add_label(scene: &mut Scene, text: &str, size: f32, color: Vec4, pos: Vec3) -> TattvaId {
    scene.add_tattva(Label::new(text, size).with_color(color), pos)
}

fn add_ring(scene: &mut Scene, origin: Vec3, radius: f32, stroke: f32, color: Vec4) -> TattvaId {
    scene.add_tattva(
        Circle::new(radius, 96, rgba(color, 0.0)).with_stroke(stroke, color),
        at(origin, 0.0, 0.25, 0.0),
    )
}

fn add_dot(scene: &mut Scene, pos: Vec3, radius: f32, color: Vec4) -> TattvaId {
    scene.add_tattva(
        Circle::new(radius, 36, color).with_stroke(0.014, rgba(WHITE, 0.72)),
        pos,
    )
}

fn appear(timeline: &mut Timeline, ids: &[TattvaId], at: f32, duration: f32) {
    for &id in ids {
        timeline
            .animate(id)
            .at(at)
            .for_duration(duration)
            .ease(Ease::OutCubic)
            .appear()
            .spawn();
    }
}

fn draw(timeline: &mut Timeline, ids: &[TattvaId], at: f32, duration: f32) {
    for &id in ids {
        timeline
            .animate(id)
            .at(at)
            .for_duration(duration)
            .ease(Ease::OutCubic)
            .draw()
            .spawn();
    }
}

fn write_text(timeline: &mut Timeline, ids: &[TattvaId], at: f32, duration: f32) {
    for &id in ids {
        timeline
            .animate(id)
            .at(at)
            .for_duration(duration)
            .ease(Ease::OutCubic)
            .typewrite_text()
            .spawn();
    }
}
