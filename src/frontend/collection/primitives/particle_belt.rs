use crate::backend::renderer::vertex::mesh::MeshVertex;
use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Mesh, Project, ProjectionCtx, RenderPrimitive};
use glam::{Vec2, Vec4, vec2};

#[derive(Debug, Clone)]
pub struct ParticleBelt {
    pub radius: f32,
    pub band_width: f32,
    pub particle_count: usize,
    pub min_particle_radius: f32,
    pub max_particle_radius: f32,
    pub palette: Vec<Vec4>,
    pub phase: f32,
    pub orbit_speed: f32,
    pub clockwise_ratio: f32,
    pub band_breathing_amplitude: f32,
    pub band_breathing_rate: f32,
    pub radial_jitter_amplitude: f32,
    pub radial_jitter_rate: f32,
    pub angular_spread: f32,
    pub seed: f32,
    pub segments_per_particle: u32,
}

impl ParticleBelt {
    pub fn new(radius: f32) -> Self {
        Self {
            radius,
            band_width: 0.5,
            particle_count: 160,
            min_particle_radius: 0.012,
            max_particle_radius: 0.05,
            palette: vec![
                Vec4::new(0.25, 0.89, 0.98, 0.95),
                Vec4::new(0.56, 0.54, 0.99, 0.90),
                Vec4::new(0.98, 0.45, 0.79, 0.88),
                Vec4::new(0.99, 0.81, 0.34, 0.92),
            ],
            phase: 0.0,
            orbit_speed: 0.8,
            clockwise_ratio: 0.5,
            band_breathing_amplitude: 0.08,
            band_breathing_rate: 1.2,
            radial_jitter_amplitude: 0.10,
            radial_jitter_rate: 2.4,
            angular_spread: std::f32::consts::TAU,
            seed: 1.0,
            segments_per_particle: 12,
        }
    }

    pub fn with_band_width(mut self, band_width: f32) -> Self {
        self.band_width = band_width.max(0.0);
        self
    }

    pub fn with_particle_count(mut self, particle_count: usize) -> Self {
        self.particle_count = particle_count.max(1);
        self
    }

    pub fn with_particle_size_range(mut self, min_radius: f32, max_radius: f32) -> Self {
        self.min_particle_radius = min_radius.max(0.001);
        self.max_particle_radius = max_radius.max(self.min_particle_radius);
        self
    }

    pub fn with_palette(mut self, palette: Vec<Vec4>) -> Self {
        if !palette.is_empty() {
            self.palette = palette;
        }
        self
    }

    pub fn with_phase(mut self, phase: f32) -> Self {
        self.phase = phase;
        self
    }

    pub fn with_orbit_speed(mut self, orbit_speed: f32) -> Self {
        self.orbit_speed = orbit_speed;
        self
    }

    pub fn with_clockwise_ratio(mut self, clockwise_ratio: f32) -> Self {
        self.clockwise_ratio = clockwise_ratio.clamp(0.0, 1.0);
        self
    }

    pub fn all_clockwise(self) -> Self {
        self.with_clockwise_ratio(1.0)
    }

    pub fn all_anticlockwise(self) -> Self {
        self.with_clockwise_ratio(0.0)
    }

    pub fn with_band_breathing(mut self, amplitude: f32, rate: f32) -> Self {
        self.band_breathing_amplitude = amplitude.max(0.0);
        self.band_breathing_rate = rate;
        self
    }

    pub fn with_radial_jitter(mut self, amplitude: f32, rate: f32) -> Self {
        self.radial_jitter_amplitude = amplitude.max(0.0);
        self.radial_jitter_rate = rate;
        self
    }

    pub fn with_seed(mut self, seed: f32) -> Self {
        self.seed = seed;
        self
    }

    pub fn with_angular_spread(mut self, angular_spread: f32) -> Self {
        self.angular_spread = angular_spread.clamp(0.0, std::f32::consts::TAU);
        self
    }

    pub fn particles(&self) -> Vec<BeltParticle> {
        let mut particles = Vec::with_capacity(self.particle_count);
        let breathing =
            self.band_breathing_amplitude * (self.phase * self.band_breathing_rate).sin();

        for idx in 0..self.particle_count {
            let h0 = hash01(self.seed + idx as f32 * 17.371);
            let h1 = hash01(self.seed + idx as f32 * 41.927 + 0.37);
            let h2 = hash01(self.seed + idx as f32 * 91.117 + 1.77);
            let h3 = hash01(self.seed + idx as f32 * 13.731 + 2.41);
            let h4 = hash01(self.seed + idx as f32 * 63.337 + 3.13);

            let base_angle = if self.angular_spread >= std::f32::consts::TAU - 1e-4 {
                h0 * std::f32::consts::TAU
            } else {
                (h0 - 0.5) * self.angular_spread
            };

            let orbit_direction = if h1 < self.clockwise_ratio { -1.0 } else { 1.0 };
            let orbit_rate = self.orbit_speed * (0.55 + 1.15 * h2);
            let angle = base_angle + orbit_direction * self.phase * orbit_rate;

            let base_radius = self.radius + (h3 - 0.5) * self.band_width + breathing;
            let radial_wobble = self.radial_jitter_amplitude
                * (self.phase * (self.radial_jitter_rate * (0.7 + h4))
                    + h0 * std::f32::consts::TAU)
                    .sin();
            let orbit_radius = (base_radius + radial_wobble).max(0.01);
            let size = self.min_particle_radius
                + (self.max_particle_radius - self.min_particle_radius) * h1;
            let color = sample_palette(&self.palette, h2);
            let alpha = 0.55 + 0.45 * h4;

            particles.push(BeltParticle {
                center: vec2(orbit_radius * angle.cos(), orbit_radius * angle.sin()),
                radius: size,
                color: Vec4::new(color.x, color.y, color.z, color.w * alpha),
            });
        }

        particles
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BeltParticle {
    pub center: Vec2,
    pub radius: f32,
    pub color: Vec4,
}

impl Project for ParticleBelt {
    fn project(&self, ctx: &mut ProjectionCtx) {
        for particle in self.particles() {
            ctx.emit(RenderPrimitive::Mesh(particle_mesh(
                particle.center,
                particle.radius,
                self.segments_per_particle,
                particle.color,
            )));
        }
    }
}

impl Bounded for ParticleBelt {
    fn local_bounds(&self) -> Bounds {
        let extent = self.radius
            + self.band_width * 0.5
            + self.band_breathing_amplitude
            + self.radial_jitter_amplitude
            + self.max_particle_radius;
        Bounds::new(vec2(-extent, -extent), vec2(extent, extent))
    }
}

pub type AsteroidBelt = ParticleBelt;

fn particle_mesh(center: Vec2, radius: f32, segments: u32, color: Vec4) -> std::sync::Arc<Mesh> {
    let seg = segments.max(6);
    let mut vertices = Vec::with_capacity((seg + 1) as usize);
    vertices.push(MeshVertex {
        position: [center.x, center.y, 0.0],
        color: [color.x, color.y, color.z, color.w],
    });

    for i in 0..seg {
        let theta = (i as f32 / seg as f32) * std::f32::consts::TAU;
        let px = center.x + radius * theta.cos();
        let py = center.y + radius * theta.sin();
        vertices.push(MeshVertex {
            position: [px, py, 0.0],
            color: [color.x, color.y, color.z, color.w],
        });
    }

    let mut indices = Vec::with_capacity((seg * 3) as usize);
    for i in 0..seg {
        indices.push(0);
        indices.push(i + 1);
        indices.push(if i + 2 <= seg { i + 2 } else { 1 });
    }

    Mesh::from_tessellation(vertices, indices)
}

fn hash01(x: f32) -> f32 {
    (x.sin() * 43_758.547).fract().abs()
}

fn sample_palette(palette: &[Vec4], t: f32) -> Vec4 {
    if palette.is_empty() {
        return Vec4::ONE;
    }
    if palette.len() == 1 {
        return palette[0];
    }

    let wrapped = t.rem_euclid(1.0);
    let scaled = wrapped * palette.len() as f32;
    let idx0 = scaled.floor() as usize % palette.len();
    let idx1 = (idx0 + 1) % palette.len();
    palette[idx0].lerp(palette[idx1], scaled.fract())
}
