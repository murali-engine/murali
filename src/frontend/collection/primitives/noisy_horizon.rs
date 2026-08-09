use crate::backend::renderer::vertex::mesh::MeshVertex;
use crate::frontend::layout::{Bounded, Bounds};
use crate::frontend::style::{StrokeParams, Style};
use crate::projection::{Mesh, Project, ProjectionCtx, RenderPrimitive};
use glam::{Vec2, Vec4, vec2, vec3};

#[derive(Debug, Clone)]
pub struct NoisyHorizonGradient {
    pub palette: Vec<Vec4>,
    pub cycles: f32,
    pub motion_rate: f32,
    pub vertical_shift: f32,
}

impl NoisyHorizonGradient {
    pub fn new(palette: Vec<Vec4>) -> Self {
        Self {
            palette,
            cycles: 1.2,
            motion_rate: 0.28,
            vertical_shift: 0.32,
        }
    }

    pub fn with_cycles(mut self, cycles: f32) -> Self {
        self.cycles = cycles.max(0.1);
        self
    }

    pub fn with_motion_rate(mut self, motion_rate: f32) -> Self {
        self.motion_rate = motion_rate;
        self
    }

    pub fn with_vertical_shift(mut self, vertical_shift: f32) -> Self {
        self.vertical_shift = vertical_shift;
        self
    }
}

#[derive(Debug, Clone)]
pub struct NoisyHorizon {
    pub x_start: f32,
    pub x_end: f32,
    pub baseline_y: f32,
    pub bottom_y: f32,
    pub samples: usize,
    pub noise_frequency: f32,
    pub noise_amplitude: f32,
    pub noise_offset: Vec2,
    pub phase: f32,
    pub morph_speed: f32,
    pub gradient: NoisyHorizonGradient,
    pub style: Style,
}

impl NoisyHorizon {
    pub fn new(width: f32) -> Self {
        let stroke_color = Vec4::new(0.95, 0.97, 1.0, 0.95);
        Self {
            x_start: -width * 0.5,
            x_end: width * 0.5,
            baseline_y: -2.7,
            bottom_y: -4.2,
            samples: 180,
            noise_frequency: 0.22,
            noise_amplitude: 0.55,
            noise_offset: Vec2::ZERO,
            phase: 0.0,
            morph_speed: 0.85,
            gradient: NoisyHorizonGradient::new(vec![
                Vec4::new(0.12, 0.78, 0.95, 0.92),
                Vec4::new(0.37, 0.58, 0.99, 0.90),
                Vec4::new(0.82, 0.38, 0.95, 0.88),
                Vec4::new(0.99, 0.68, 0.32, 0.86),
            ]),
            style: Style::new().with_stroke(StrokeParams {
                thickness: 0.045,
                color: stroke_color,
                ..Default::default()
            }),
        }
    }

    pub fn with_x_range(mut self, x_start: f32, x_end: f32) -> Self {
        self.x_start = x_start.min(x_end);
        self.x_end = x_start.max(x_end);
        self
    }

    pub fn with_baseline_y(mut self, baseline_y: f32) -> Self {
        self.baseline_y = baseline_y;
        self
    }

    pub fn with_bottom_y(mut self, bottom_y: f32) -> Self {
        self.bottom_y = bottom_y;
        self
    }

    pub fn with_samples(mut self, samples: usize) -> Self {
        self.samples = samples.max(8);
        self
    }

    pub fn with_noise_frequency(mut self, noise_frequency: f32) -> Self {
        self.noise_frequency = noise_frequency.max(0.001);
        self
    }

    pub fn with_noise_amplitude(mut self, noise_amplitude: f32) -> Self {
        self.noise_amplitude = noise_amplitude.max(0.0);
        self
    }

    pub fn with_noise_offset(mut self, noise_offset: Vec2) -> Self {
        self.noise_offset = noise_offset;
        self
    }

    pub fn with_noise_seed(mut self, seed: f32) -> Self {
        self.noise_offset = vec2(seed * 12.9898, seed * 78.233);
        self
    }

    pub fn with_phase(mut self, phase: f32) -> Self {
        self.phase = phase;
        self
    }

    pub fn with_morph_speed(mut self, morph_speed: f32) -> Self {
        self.morph_speed = morph_speed;
        self
    }

    pub fn with_gradient(mut self, gradient: NoisyHorizonGradient) -> Self {
        self.gradient = gradient;
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

    pub fn sample_points(&self) -> Vec<Vec2> {
        let count = self.samples.max(8);
        let width = (self.x_end - self.x_start).abs();
        let mut points = Vec::with_capacity(count + 1);

        for i in 0..=count {
            let u = i as f32 / count as f32;
            let x = self.x_start + u * width;
            let nx = x * self.noise_frequency + self.noise_offset.x;
            let ny = self.phase + self.noise_offset.y;
            let y = self.baseline_y + perlin2(nx, ny) * self.noise_amplitude;
            points.push(vec2(x, y));
        }

        points
    }

    fn fill_color(&self, u: f32, v: f32) -> Vec4 {
        if self.gradient.palette.is_empty() {
            return Vec4::new(0.3, 0.6, 0.95, 0.8);
        }
        if self.gradient.palette.len() == 1 {
            return self.gradient.palette[0];
        }

        let shifted = (u * self.gradient.cycles
            + self.phase * self.gradient.motion_rate
            + v * self.gradient.vertical_shift)
            .fract();
        sample_palette(&self.gradient.palette, shifted)
    }

    fn fill_mesh(&self, points: &[Vec2]) -> std::sync::Arc<Mesh> {
        let mut vertices = Vec::with_capacity(points.len() * 2);
        let mut indices = Vec::with_capacity((points.len().saturating_sub(1)) * 6);

        for (i, top) in points.iter().enumerate() {
            let u = i as f32 / points.len().saturating_sub(1).max(1) as f32;
            let top_color = self.fill_color(u, 0.0);
            let bottom_color = self.fill_color(u, 1.0);

            vertices.push(MeshVertex {
                position: [top.x, top.y, 0.0],
                color: [top_color.x, top_color.y, top_color.z, top_color.w],
            });
            vertices.push(MeshVertex {
                position: [top.x, self.bottom_y, 0.0],
                color: [
                    bottom_color.x,
                    bottom_color.y,
                    bottom_color.z,
                    bottom_color.w,
                ],
            });
        }

        for i in 0..points.len().saturating_sub(1) {
            let base = (i * 2) as u32;
            indices.extend_from_slice(&[base, base + 1, base + 3, base, base + 3, base + 2]);
        }

        Mesh::from_tessellation(vertices, indices)
    }
}

impl Project for NoisyHorizon {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let points = self.sample_points();
        if points.len() < 2 {
            return;
        }

        ctx.emit(RenderPrimitive::Mesh(self.fill_mesh(&points)));

        if let Some(stroke) = &self.style.stroke {
            for pair in points.windows(2) {
                let a = pair[0];
                let b = pair[1];
                ctx.emit(RenderPrimitive::Line {
                    start: vec3(a.x, a.y, 0.0),
                    end: vec3(b.x, b.y, 0.0),
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

impl Bounded for NoisyHorizon {
    fn local_bounds(&self) -> Bounds {
        let top = self.baseline_y
            + self.noise_amplitude
            + self
                .style
                .stroke
                .as_ref()
                .map(|s| s.thickness)
                .unwrap_or(0.0);
        let bottom = self.bottom_y.min(self.baseline_y - self.noise_amplitude);
        Bounds::new(vec2(self.x_start, bottom), vec2(self.x_end, top))
    }
}

pub type PerlinNoiseHorizon = NoisyHorizon;
pub type GenerativeHorizon = NoisyHorizon;
pub type PerlinNoiseTerrain = NoisyHorizon;
pub type PerlinNoiseHorizonGradient = NoisyHorizonGradient;

#[derive(Debug, Clone)]
pub struct PerlinFieldLayer {
    pub y_offset: f32,
    pub amplitude_scale: f32,
    pub frequency_scale: f32,
    pub phase_offset: f32,
    pub opacity: f32,
    pub stroke_thickness: f32,
}

impl PerlinFieldLayer {
    pub fn new(y_offset: f32) -> Self {
        Self {
            y_offset,
            amplitude_scale: 1.0,
            frequency_scale: 1.0,
            phase_offset: 0.0,
            opacity: 0.18,
            stroke_thickness: 0.018,
        }
    }

    pub fn with_amplitude_scale(mut self, amplitude_scale: f32) -> Self {
        self.amplitude_scale = amplitude_scale.max(0.0);
        self
    }

    pub fn with_frequency_scale(mut self, frequency_scale: f32) -> Self {
        self.frequency_scale = frequency_scale.max(0.001);
        self
    }

    pub fn with_phase_offset(mut self, phase_offset: f32) -> Self {
        self.phase_offset = phase_offset;
        self
    }

    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    pub fn with_stroke_thickness(mut self, stroke_thickness: f32) -> Self {
        self.stroke_thickness = stroke_thickness.max(0.0);
        self
    }
}

#[derive(Debug, Clone)]
pub struct LayeredPerlinField {
    pub x_start: f32,
    pub x_end: f32,
    pub baseline_y: f32,
    pub bottom_y: f32,
    pub samples: usize,
    pub noise_frequency: f32,
    pub noise_amplitude: f32,
    pub noise_offset: Vec2,
    pub phase: f32,
    pub morph_speed: f32,
    pub gradient: NoisyHorizonGradient,
    pub layers: Vec<PerlinFieldLayer>,
    pub stroke_color: Vec4,
}

impl LayeredPerlinField {
    pub fn new(width: f32) -> Self {
        Self {
            x_start: -width * 0.5,
            x_end: width * 0.5,
            baseline_y: -1.8,
            bottom_y: -4.3,
            samples: 220,
            noise_frequency: 0.22,
            noise_amplitude: 0.55,
            noise_offset: Vec2::ZERO,
            phase: 0.0,
            morph_speed: 0.9,
            gradient: NoisyHorizonGradient::new(vec![
                Vec4::new(0.08, 0.82, 0.98, 0.95),
                Vec4::new(0.39, 0.58, 1.00, 0.88),
                Vec4::new(0.90, 0.42, 0.95, 0.84),
                Vec4::new(1.00, 0.76, 0.42, 0.82),
            ])
            .with_cycles(1.6)
            .with_motion_rate(0.24)
            .with_vertical_shift(0.28),
            layers: vec![
                PerlinFieldLayer::new(0.0)
                    .with_opacity(0.16)
                    .with_stroke_thickness(0.018),
                PerlinFieldLayer::new(-0.18)
                    .with_amplitude_scale(0.95)
                    .with_frequency_scale(1.05)
                    .with_phase_offset(0.65)
                    .with_opacity(0.14)
                    .with_stroke_thickness(0.016),
                PerlinFieldLayer::new(-0.36)
                    .with_amplitude_scale(0.88)
                    .with_frequency_scale(1.12)
                    .with_phase_offset(1.25)
                    .with_opacity(0.12)
                    .with_stroke_thickness(0.015),
                PerlinFieldLayer::new(-0.54)
                    .with_amplitude_scale(0.82)
                    .with_frequency_scale(1.20)
                    .with_phase_offset(1.95)
                    .with_opacity(0.10)
                    .with_stroke_thickness(0.014),
                PerlinFieldLayer::new(-0.72)
                    .with_amplitude_scale(0.74)
                    .with_frequency_scale(1.30)
                    .with_phase_offset(2.55)
                    .with_opacity(0.08)
                    .with_stroke_thickness(0.013),
            ],
            stroke_color: Vec4::new(0.96, 0.98, 1.0, 0.22),
        }
    }

    pub fn with_x_range(mut self, x_start: f32, x_end: f32) -> Self {
        self.x_start = x_start.min(x_end);
        self.x_end = x_start.max(x_end);
        self
    }

    pub fn with_baseline_y(mut self, baseline_y: f32) -> Self {
        self.baseline_y = baseline_y;
        self
    }

    pub fn with_bottom_y(mut self, bottom_y: f32) -> Self {
        self.bottom_y = bottom_y;
        self
    }

    pub fn with_samples(mut self, samples: usize) -> Self {
        self.samples = samples.max(8);
        self
    }

    pub fn with_noise_frequency(mut self, noise_frequency: f32) -> Self {
        self.noise_frequency = noise_frequency.max(0.001);
        self
    }

    pub fn with_noise_amplitude(mut self, noise_amplitude: f32) -> Self {
        self.noise_amplitude = noise_amplitude.max(0.0);
        self
    }

    pub fn with_noise_offset(mut self, noise_offset: Vec2) -> Self {
        self.noise_offset = noise_offset;
        self
    }

    pub fn with_noise_seed(mut self, seed: f32) -> Self {
        self.noise_offset = vec2(seed * 12.9898, seed * 78.233);
        self
    }

    pub fn with_phase(mut self, phase: f32) -> Self {
        self.phase = phase;
        self
    }

    pub fn with_morph_speed(mut self, morph_speed: f32) -> Self {
        self.morph_speed = morph_speed;
        self
    }

    pub fn with_gradient(mut self, gradient: NoisyHorizonGradient) -> Self {
        self.gradient = gradient;
        self
    }

    pub fn with_layers(mut self, layers: Vec<PerlinFieldLayer>) -> Self {
        if !layers.is_empty() {
            self.layers = layers;
        }
        self
    }

    pub fn with_layer_count(mut self, layer_count: usize) -> Self {
        let count = layer_count.max(1);
        let mut layers = Vec::with_capacity(count);
        for idx in 0..count {
            let t = idx as f32 / count.max(1) as f32;
            layers.push(
                PerlinFieldLayer::new(-0.16 * idx as f32)
                    .with_amplitude_scale(1.0 - 0.35 * t)
                    .with_frequency_scale(1.0 + 0.35 * t)
                    .with_phase_offset(0.55 * idx as f32)
                    .with_opacity((0.18 - 0.1 * t).max(0.04))
                    .with_stroke_thickness((0.018 - 0.008 * t).max(0.008)),
            );
        }
        self.layers = layers;
        self
    }

    pub fn with_stroke_color(mut self, stroke_color: Vec4) -> Self {
        self.stroke_color = stroke_color;
        self
    }

    fn sample_layer_points(&self, layer: &PerlinFieldLayer) -> Vec<Vec2> {
        let count = self.samples.max(8);
        let width = (self.x_end - self.x_start).abs();
        let mut points = Vec::with_capacity(count + 1);

        for i in 0..=count {
            let u = i as f32 / count as f32;
            let x = self.x_start + u * width;
            let nx = x * self.noise_frequency * layer.frequency_scale + self.noise_offset.x;
            let ny = self.phase + layer.phase_offset + self.noise_offset.y;
            let y = self.baseline_y
                + layer.y_offset
                + perlin2(nx, ny) * self.noise_amplitude * layer.amplitude_scale;
            points.push(vec2(x, y));
        }

        points
    }

    fn fill_color(&self, u: f32, v: f32, layer_idx: usize) -> Vec4 {
        if self.gradient.palette.is_empty() {
            return Vec4::new(0.3, 0.6, 0.95, 0.15);
        }
        let layer_shift = layer_idx as f32 * 0.11;
        let shifted = (u * self.gradient.cycles
            + self.phase * self.gradient.motion_rate
            + v * self.gradient.vertical_shift
            + layer_shift)
            .fract();
        sample_palette(&self.gradient.palette, shifted)
    }

    fn fill_mesh(&self, points: &[Vec2], layer_idx: usize, opacity: f32) -> std::sync::Arc<Mesh> {
        let mut vertices = Vec::with_capacity(points.len() * 2);
        let mut indices = Vec::with_capacity((points.len().saturating_sub(1)) * 6);

        for (i, top) in points.iter().enumerate() {
            let u = i as f32 / points.len().saturating_sub(1).max(1) as f32;
            let top_color = self.fill_color(u, 0.0, layer_idx);
            let bottom_color = self.fill_color(u, 1.0, layer_idx);

            vertices.push(MeshVertex {
                position: [top.x, top.y, 0.0],
                color: [top_color.x, top_color.y, top_color.z, top_color.w * opacity],
            });
            vertices.push(MeshVertex {
                position: [top.x, self.bottom_y, 0.0],
                color: [
                    bottom_color.x,
                    bottom_color.y,
                    bottom_color.z,
                    bottom_color.w * opacity * 0.75,
                ],
            });
        }

        for i in 0..points.len().saturating_sub(1) {
            let base = (i * 2) as u32;
            indices.extend_from_slice(&[base, base + 1, base + 3, base, base + 3, base + 2]);
        }

        Mesh::from_tessellation(vertices, indices)
    }
}

impl Project for LayeredPerlinField {
    fn project(&self, ctx: &mut ProjectionCtx) {
        for (layer_idx, layer) in self.layers.iter().enumerate() {
            let points = self.sample_layer_points(layer);
            if points.len() < 2 {
                continue;
            }

            ctx.emit(RenderPrimitive::Mesh(self.fill_mesh(
                &points,
                layer_idx,
                layer.opacity,
            )));

            let stroke_color = Vec4::new(
                self.stroke_color.x,
                self.stroke_color.y,
                self.stroke_color.z,
                self.stroke_color.w * (0.5 + layer.opacity),
            );

            for pair in points.windows(2) {
                let a = pair[0];
                let b = pair[1];
                ctx.emit(RenderPrimitive::Line {
                    start: vec3(a.x, a.y, 0.0),
                    end: vec3(b.x, b.y, 0.0),
                    thickness: layer.stroke_thickness,
                    color: stroke_color,
                    dash_length: 0.0,
                    gap_length: 0.0,
                    dash_offset: 0.0,
                });
            }
        }
    }
}

impl Bounded for LayeredPerlinField {
    fn local_bounds(&self) -> Bounds {
        let min_layer = self
            .layers
            .iter()
            .map(|layer| {
                self.baseline_y + layer.y_offset - self.noise_amplitude * layer.amplitude_scale
            })
            .fold(self.baseline_y - self.noise_amplitude, f32::min);
        let max_layer = self
            .layers
            .iter()
            .map(|layer| {
                self.baseline_y
                    + layer.y_offset
                    + self.noise_amplitude * layer.amplitude_scale
                    + layer.stroke_thickness
            })
            .fold(self.baseline_y + self.noise_amplitude, f32::max);

        Bounds::new(
            vec2(self.x_start, self.bottom_y.min(min_layer)),
            vec2(self.x_end, max_layer),
        )
    }
}

pub type MultiLayeredPerlinField = LayeredPerlinField;
pub type AINoiseField = LayeredPerlinField;

fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn grad2(hash: u32, x: f32, y: f32) -> f32 {
    match hash & 7 {
        0 => x + y,
        1 => -x + y,
        2 => x - y,
        3 => -x - y,
        4 => x,
        5 => -x,
        6 => y,
        _ => -y,
    }
}

fn hash2(x: i32, y: i32) -> u32 {
    let mut h = x as u32;
    h = h
        .wrapping_mul(374_761_393)
        .wrapping_add((y as u32).wrapping_mul(668_265_263));
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^ (h >> 16)
}

fn perlin2(x: f32, y: f32) -> f32 {
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let x1 = x0 + 1;
    let y1 = y0 + 1;

    let xf = x - x0 as f32;
    let yf = y - y0 as f32;

    let u = fade(xf);
    let v = fade(yf);

    let n00 = grad2(hash2(x0, y0), xf, yf);
    let n10 = grad2(hash2(x1, y0), xf - 1.0, yf);
    let n01 = grad2(hash2(x0, y1), xf, yf - 1.0);
    let n11 = grad2(hash2(x1, y1), xf - 1.0, yf - 1.0);

    let x0i = lerp(n00, n10, u);
    let x1i = lerp(n01, n11, u);
    lerp(x0i, x1i, v)
}

fn sample_palette(palette: &[Vec4], t: f32) -> Vec4 {
    let wrapped = t.rem_euclid(1.0);
    let scaled = wrapped * palette.len() as f32;
    let idx0 = scaled.floor() as usize % palette.len();
    let idx1 = (idx0 + 1) % palette.len();
    palette[idx0].lerp(palette[idx1], scaled.fract())
}
