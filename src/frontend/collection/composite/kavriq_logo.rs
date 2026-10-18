//! Kavriq's water-drop mark and the seekable splash ident.
//!
//! The splash is one continuous motion: fill, sag, detach, fall, then a puddle
//! and eight satellite droplets. Set [`KavriqSplashLogo::time`] in seconds.
//! Wrapping it with the duration loops the ident.

use crate::backend::renderer::vertex::mesh::MeshVertex;
use crate::frontend::layout::{Bounded, Bounds};
use crate::math::bezier::cubic_bezier;
use crate::projection::{Mesh, Project, ProjectionCtx, RenderPrimitive};
use glam::{Vec2, Vec4, vec2};

/// `#08A9C4`
pub const KAVRIQ_BLUE: Vec4 = Vec4::new(8.0 / 255.0, 169.0 / 255.0, 196.0 / 255.0, 1.0);
pub const KAVRIQ_SPLASH_DURATION: f32 = 4.2;

const REFERENCE_DURATION: f32 = 4.2;
const GROW_START: f32 = 0.08;
const DETACH_START: f32 = 1.34;
const FALL_START: f32 = 1.7;
const IMPACT_TIME: f32 = 2.25;
const SURFACE_Y: f32 = -1.08;
const DETACH_ACCELERATION: f32 = 2.4;
const RELEASE_SPEED: f32 = 2.0 * DETACH_ACCELERATION * (FALL_START - DETACH_START);
const FALL_ACCELERATION: f32 = 6.58;
const VIEW_CENTER_Y: f32 = 0.03;

#[derive(Debug, Clone, Copy)]
struct SplashParticle {
    origin_x: f32,
    velocity_x: f32,
    velocity_y: f32,
    delay: f32,
    size: f32,
    gravity: f32,
}

const SPLASH_PARTICLES: [SplashParticle; 8] = [
    SplashParticle {
        origin_x: -0.38,
        velocity_x: -3.28,
        velocity_y: 2.18,
        delay: 0.015,
        size: 0.72,
        gravity: 7.7,
    },
    SplashParticle {
        origin_x: -0.29,
        velocity_x: -2.58,
        velocity_y: 3.34,
        delay: 0.0,
        size: 1.02,
        gravity: 7.5,
    },
    SplashParticle {
        origin_x: -0.19,
        velocity_x: -1.72,
        velocity_y: 4.02,
        delay: 0.035,
        size: 0.82,
        gravity: 8.0,
    },
    SplashParticle {
        origin_x: -0.07,
        velocity_x: -0.72,
        velocity_y: 3.02,
        delay: 0.075,
        size: 0.58,
        gravity: 7.2,
    },
    SplashParticle {
        origin_x: 0.05,
        velocity_x: 0.58,
        velocity_y: 3.48,
        delay: 0.055,
        size: 0.64,
        gravity: 7.6,
    },
    SplashParticle {
        origin_x: 0.17,
        velocity_x: 1.46,
        velocity_y: 4.22,
        delay: 0.018,
        size: 0.9,
        gravity: 8.2,
    },
    SplashParticle {
        origin_x: 0.28,
        velocity_x: 2.38,
        velocity_y: 3.06,
        delay: 0.0,
        size: 1.06,
        gravity: 7.4,
    },
    SplashParticle {
        origin_x: 0.39,
        velocity_x: 3.18,
        velocity_y: 2.04,
        delay: 0.028,
        size: 0.68,
        gravity: 7.8,
    },
];

#[derive(Debug, Clone, Copy)]
pub struct KavriqPiecePose {
    pub x: f32,
    pub y: f32,
    pub rotation_z: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub opacity: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct KavriqSplashFrame {
    pub drop: KavriqPiecePose,
    pub puddle: KavriqPiecePose,
    pub splashes: [KavriqPiecePose; 8],
}

/// Standalone drop, centered, with `height` as its full vertical size.
#[derive(Debug, Clone, Copy)]
pub struct KavriqLogoMark {
    pub height: f32,
    pub color: Vec4,
}

impl Default for KavriqLogoMark {
    fn default() -> Self {
        Self::new()
    }
}

impl KavriqLogoMark {
    pub fn new() -> Self {
        Self {
            height: 4.8,
            color: KAVRIQ_BLUE,
        }
    }

    pub fn with_height(mut self, height: f32) -> Self {
        self.height = height.abs().max(0.01);
        self
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }
}

impl Project for KavriqLogoMark {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let points = drop_local_points();
        let (min_y, max_y) = points.iter().fold((f32::MAX, f32::MIN), |(lo, hi), point| {
            (lo.min(point.y), hi.max(point.y))
        });
        let span = (max_y - min_y).max(0.001);
        let scale = self.height / span;
        let mid_y = (min_y + max_y) * 0.5;
        let placed = points
            .into_iter()
            .map(|point| vec2(point.x * scale, (point.y - mid_y) * scale))
            .collect::<Vec<_>>();
        emit_fan(ctx, &placed, 0.0, self.color);
    }
}

impl Bounded for KavriqLogoMark {
    fn local_bounds(&self) -> Bounds {
        let half_h = self.height * 0.5;
        let half_w = self.height * (2.12 / 3.1) * 0.5;
        Bounds::new(vec2(-half_w, -half_h), vec2(half_w, half_h))
    }
}

/// Growing drop, impact, puddle, and satellite splash.
///
/// `time` is in seconds. The pose is the reference 4.2 second ident scaled to
/// [`KavriqSplashLogo::duration`].
#[derive(Debug, Clone, Copy)]
pub struct KavriqSplashLogo {
    pub color: Vec4,
    pub time: f32,
    pub duration: f32,
}

impl Default for KavriqSplashLogo {
    fn default() -> Self {
        Self::new()
    }
}

impl KavriqSplashLogo {
    pub fn new() -> Self {
        Self {
            color: KAVRIQ_BLUE,
            time: 0.0,
            duration: KAVRIQ_SPLASH_DURATION,
        }
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }

    pub fn with_time(mut self, time: f32) -> Self {
        self.time = time;
        self
    }

    pub fn with_duration(mut self, duration: f32) -> Self {
        self.duration = duration.abs().max(0.05);
        self
    }
}

impl Project for KavriqSplashLogo {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let frame = sample_kavriq_splash(self.time, self.duration);
        let color = self.color;
        let outline = drop_local_points();
        emit_pose_shape(ctx, &outline, frame.drop, 0.04, color);
        emit_pose_ellipse(ctx, 0.76, 0.14, frame.puddle, 0.0, color);
        for splash in frame.splashes {
            emit_pose_ellipse(ctx, 0.075, 0.19, splash, 0.02, color);
        }
    }
}

impl Bounded for KavriqSplashLogo {
    fn local_bounds(&self) -> Bounds {
        Bounds::new(vec2(-3.4, -1.7), vec2(3.4, 3.3))
    }
}

pub fn sample_kavriq_splash(time: f32, duration: f32) -> KavriqSplashFrame {
    let duration = if duration.is_finite() && duration > 0.0 {
        duration
    } else {
        REFERENCE_DURATION
    };
    let reference = (time / duration).clamp(0.0, 1.0) * REFERENCE_DURATION;
    KavriqSplashFrame {
        drop: drop_pose(reference),
        puddle: puddle_pose(reference),
        splashes: std::array::from_fn(|index| splash_pose(reference, SPLASH_PARTICLES[index])),
    }
}

fn drop_pose(time: f32) -> KavriqPiecePose {
    if time < GROW_START {
        return pose(0.0, 1.46, 0.0, 0.18, 0.18, 0.0);
    }
    if time < DETACH_START {
        let progress = smoothstep((time - GROW_START) / (DETACH_START - GROW_START));
        let scale = mix(0.18, 0.75, progress);
        let weight = smoothstep((progress - 0.48) / 0.52);
        return pose(
            0.0,
            1.46 - 0.2 * weight,
            0.0,
            scale * (1.0 - 0.045 * weight),
            scale * (1.0 + 0.08 * weight),
            smoothstep(((time - GROW_START) / 0.16).min(1.0)),
        );
    }
    if time < FALL_START {
        let elapsed = time - DETACH_START;
        let weighted = (elapsed / (FALL_START - DETACH_START)).powi(2);
        return pose(
            0.0,
            1.26 - DETACH_ACCELERATION * elapsed * elapsed,
            0.0,
            mix(0.716, 0.61, weighted),
            mix(0.81, 0.92, weighted),
            1.0,
        );
    }
    if time < IMPACT_TIME {
        let elapsed = time - FALL_START;
        let progress = elapsed / (IMPACT_TIME - FALL_START);
        let scale_y = 0.92 + 0.15 * progress * progress;
        let conserved = 0.61 * 0.92;
        return pose(
            0.0,
            0.949 - RELEASE_SPEED * elapsed - 0.5 * FALL_ACCELERATION * elapsed * elapsed,
            0.0,
            conserved / scale_y,
            scale_y,
            1.0,
        );
    }
    let impact = ((time - IMPACT_TIME) / 0.105).clamp(0.0, 1.0);
    let compression = 1.0 - (1.0 - impact).powi(3);
    pose(
        0.0,
        mix(-0.99, SURFACE_Y, compression),
        0.0,
        mix(0.525, 1.36, compression),
        mix(1.07, 0.12, compression),
        1.0 - smoothstep((impact - 0.04) / 0.38),
    )
}

fn puddle_pose(time: f32) -> KavriqPiecePose {
    let elapsed = time - IMPACT_TIME;
    if elapsed < 0.0 {
        return pose(0.0, SURFACE_Y, 0.0, 0.12, 0.12, 0.0);
    }
    let spread = 1.0 - (-8.5 * elapsed).exp();
    let ripple = (-4.2 * elapsed).exp() * (15.0 * elapsed).sin();
    let fade = 1.0 - smoothstep((elapsed - 1.12) / 0.72);
    pose(
        0.0,
        SURFACE_Y,
        0.0,
        0.16 + 2.08 * spread,
        0.46 + 0.42 * (-5.2 * elapsed).exp() + 0.11 * ripple,
        fade.clamp(0.0, 1.0) * 0.95,
    )
}

fn splash_pose(time: f32, particle: SplashParticle) -> KavriqPiecePose {
    let elapsed = time - IMPACT_TIME - particle.delay;
    let start_y = -0.94;
    if elapsed < 0.0 {
        let planted = particle.size * 0.35;
        return pose(particle.origin_x, start_y, 0.0, planted, planted, 0.0);
    }
    let flight = flight_time(particle, start_y);
    if elapsed <= flight {
        let vertical = particle.velocity_y - particle.gravity * elapsed;
        let speed = (particle.velocity_x * particle.velocity_x + vertical * vertical).sqrt();
        let stretch = 1.0 + (speed * 0.065).min(0.48);
        return pose(
            particle.origin_x + particle.velocity_x * elapsed * (1.0 - 0.055 * elapsed),
            start_y + particle.velocity_y * elapsed - 0.5 * particle.gravity * elapsed * elapsed,
            particle.velocity_x.atan2(vertical),
            particle.size / stretch.sqrt(),
            particle.size * stretch,
            smoothstep((elapsed / 0.055).min(1.0)),
        );
    }
    let landing_x = particle.origin_x + particle.velocity_x * flight * (1.0 - 0.055 * flight);
    let skid = ((elapsed - flight) / 0.34).clamp(0.0, 1.0);
    let flat: f32 = if particle.velocity_x < 0.0 {
        -88.0
    } else {
        88.0
    };
    pose(
        landing_x + particle.velocity_x * 0.12 * smoothstep(skid),
        SURFACE_Y,
        flat.to_radians(),
        particle.size * mix(0.88, 1.45, skid),
        particle.size * mix(0.62, 0.12, skid),
        1.0 - smoothstep(skid),
    )
}

fn flight_time(particle: SplashParticle, start_y: f32) -> f32 {
    let rise =
        particle.velocity_y * particle.velocity_y + 2.0 * particle.gravity * (start_y - SURFACE_Y);
    (particle.velocity_y + rise.max(0.0).sqrt()) / particle.gravity
}

fn pose(
    x: f32,
    y: f32,
    rotation_z: f32,
    scale_x: f32,
    scale_y: f32,
    opacity: f32,
) -> KavriqPiecePose {
    KavriqPiecePose {
        x,
        y,
        rotation_z,
        scale_x,
        scale_y,
        opacity,
    }
}

fn emit_pose_shape(
    ctx: &mut ProjectionCtx,
    local: &[Vec2],
    pose: KavriqPiecePose,
    z: f32,
    color: Vec4,
) {
    if pose.opacity <= 0.01 {
        return;
    }
    let placed = local
        .iter()
        .map(|point| {
            vec2(
                pose.x + point.x * pose.scale_x,
                pose.y + point.y * pose.scale_y,
            )
        })
        .collect::<Vec<_>>();
    ctx.with_opacity(pose.opacity, |ctx| emit_fan(ctx, &placed, z, color));
}

fn emit_pose_ellipse(
    ctx: &mut ProjectionCtx,
    radius_x: f32,
    radius_y: f32,
    pose: KavriqPiecePose,
    z: f32,
    color: Vec4,
) {
    if pose.opacity <= 0.01 {
        return;
    }
    // `rotation_z` is the angle from local +Y toward +X. An ellipse is symmetric,
    // so the long axis lands on the velocity line either way.
    let (sine, cosine) = pose.rotation_z.sin_cos();
    let segments = 28u32;
    let mut points = Vec::with_capacity(segments as usize);
    for index in 0..segments {
        let t = index as f32 / segments as f32 * std::f32::consts::TAU;
        let x = radius_x * pose.scale_x * t.cos();
        let y = radius_y * pose.scale_y * t.sin();
        points.push(vec2(
            pose.x + x * cosine - y * sine,
            pose.y + x * sine + y * cosine,
        ));
    }
    ctx.with_opacity(pose.opacity, |ctx| emit_fan(ctx, &points, z, color));
}

fn emit_fan(ctx: &mut ProjectionCtx, points: &[Vec2], z: f32, color: Vec4) {
    if points.len() < 3 {
        return;
    }
    let rgba = [color.x, color.y, color.z, 1.0];
    let mut vertices = Vec::with_capacity(points.len() + 1);
    let origin = points.iter().fold(Vec2::ZERO, |sum, point| sum + *point) / points.len() as f32;
    vertices.push(MeshVertex {
        position: [origin.x, origin.y, z],
        color: rgba,
    });
    for point in points {
        vertices.push(MeshVertex {
            position: [point.x, point.y, z],
            color: rgba,
        });
    }
    let mut indices = Vec::with_capacity(points.len() * 3);
    let count = points.len() as u32;
    for index in 0..count {
        indices.extend([0, index + 1, if index + 1 == count { 1 } else { index + 2 }]);
    }
    ctx.emit(RenderPrimitive::Mesh(Mesh::from_tessellation(
        vertices, indices,
    )));
}

/// Drop outline in y-up space, centered on the SVG view box.
fn drop_local_points() -> Vec<Vec2> {
    const CUBICS: [[(f32, f32); 4]; 4] = [
        [(0.0, -1.42), (-0.14, -1.02), (-0.96, -0.30), (-0.96, 0.48)],
        [(-0.96, 0.48), (-0.96, 1.08), (-0.54, 1.48), (0.0, 1.48)],
        [(0.0, 1.48), (0.54, 1.48), (0.96, 1.08), (0.96, 0.48)],
        [(0.96, 0.48), (0.96, -0.30), (0.14, -1.02), (0.0, -1.42)],
    ];
    let mut points = Vec::with_capacity(48);
    for cubic in CUBICS {
        let p0 = vec2(cubic[0].0, cubic[0].1);
        let p1 = vec2(cubic[1].0, cubic[1].1);
        let p2 = vec2(cubic[2].0, cubic[2].1);
        let p3 = vec2(cubic[3].0, cubic[3].1);
        for step in 0..12 {
            let point = cubic_bezier(p0, p1, p2, p3, step as f32 / 12.0);
            points.push(vec2(point.x, -point.y + VIEW_CENTER_Y));
        }
    }
    points
}

fn smoothstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

fn mix(from: f32, to: f32, amount: f32) -> f32 {
    from + (to - from) * amount
}

#[cfg(test)]
mod tests {
    use super::{IMPACT_TIME, sample_kavriq_splash};

    #[test]
    fn splash_grows_then_impacts() {
        let hidden = sample_kavriq_splash(0.0, 4.2);
        assert!(hidden.drop.opacity < 0.01);
        assert!(hidden.puddle.opacity < 0.01);

        let grown = sample_kavriq_splash(1.0, 4.2);
        assert!(grown.drop.opacity > 0.9);
        assert!(grown.drop.y > 1.0);
        assert!(grown.puddle.opacity < 0.01);

        let falling = sample_kavriq_splash(2.0, 4.2);
        assert!(falling.drop.opacity > 0.99);
        assert!(falling.drop.y < grown.drop.y);

        let impacted = sample_kavriq_splash(IMPACT_TIME + 0.2, 4.2);
        assert!(impacted.puddle.opacity > 0.5);
        assert!(impacted.splashes.iter().any(|splash| splash.opacity > 0.5));
        assert!(impacted.drop.opacity < falling.drop.opacity);
    }

    #[test]
    fn duration_scales_the_same_pose() {
        let reference = sample_kavriq_splash(2.0, 4.2);
        let slowed = sample_kavriq_splash(4.0, 8.4);
        assert!((reference.drop.y - slowed.drop.y).abs() < 1.0e-4);
        assert!((reference.drop.scale_y - slowed.drop.scale_y).abs() < 1.0e-4);
    }
}
