use glam::{Vec2, Vec4, vec2, vec3};

use crate::frontend::layout::{Bounded, Bounds};
use crate::frontend::style::{StrokeParams, Style};
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};

/// A circular contour displaced by polar Perlin noise so the loop closes cleanly.
///
/// `NoisyCircle` is the short, friendly name for authoring.
/// `PerlinNoiseCircle` is provided as an alias for discoverability.
#[derive(Debug, Clone)]
pub enum NoisyCircleColorMode {
    Monochrome(Vec4),
    Multicolor(NoisyCircleGradient),
}

#[derive(Debug, Clone)]
pub struct NoisyCircleGradient {
    pub palette: Vec<Vec4>,
    pub cycles: f32,
    pub motion_rate: f32,
}

impl NoisyCircleGradient {
    pub fn new(palette: Vec<Vec4>) -> Self {
        Self {
            palette,
            cycles: 1.0,
            motion_rate: 0.35,
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
}

#[derive(Debug, Clone)]
pub struct NoisyCircle {
    pub radius: f32,
    pub samples: usize,
    pub noise_space_radius: f32,
    pub noise_amplitude: f32,
    pub noise_offset: Vec2,
    pub phase: f32,
    pub morph_speed: f32,
    pub color_mode: NoisyCircleColorMode,
    pub style: Style,
}

impl NoisyCircle {
    pub fn new(radius: f32, color: Vec4) -> Self {
        Self {
            radius,
            samples: 180,
            noise_space_radius: 1.0,
            noise_amplitude: 0.18,
            noise_offset: Vec2::ZERO,
            phase: 0.0,
            morph_speed: 0.8,
            color_mode: NoisyCircleColorMode::Monochrome(color),
            style: Style::new().with_stroke(StrokeParams {
                thickness: 0.04,
                color,
                ..Default::default()
            }),
        }
    }

    pub fn with_samples(mut self, samples: usize) -> Self {
        self.samples = samples.max(12);
        self
    }

    pub fn with_noise_space_radius(mut self, radius: f32) -> Self {
        self.noise_space_radius = radius.max(0.001);
        self
    }

    pub fn with_noise_frequency(self, frequency: f32) -> Self {
        self.with_noise_space_radius(frequency)
    }

    pub fn with_noise_amplitude(mut self, amplitude: f32) -> Self {
        self.noise_amplitude = amplitude.max(0.0);
        self
    }

    pub fn with_noise_offset(mut self, offset: Vec2) -> Self {
        self.noise_offset = offset;
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

    pub fn with_noise_seed(mut self, seed: f32) -> Self {
        self.noise_offset = vec2(seed * 12.9898, seed * 78.233);
        self
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
        if matches!(self.color_mode, NoisyCircleColorMode::Monochrome(_)) {
            self.color_mode = NoisyCircleColorMode::Monochrome(color);
        }
        self
    }

    pub fn monochrome(mut self, color: Vec4) -> Self {
        self.color_mode = NoisyCircleColorMode::Monochrome(color);
        if let Some(stroke) = self.style.stroke.as_mut() {
            stroke.color = color;
        }
        self
    }

    pub fn multicolor(mut self, palette: Vec<Vec4>) -> Self {
        self.color_mode = NoisyCircleColorMode::Multicolor(NoisyCircleGradient::new(palette));
        self
    }

    pub fn with_gradient(mut self, gradient: NoisyCircleGradient) -> Self {
        self.color_mode = NoisyCircleColorMode::Multicolor(gradient);
        self
    }

    pub fn with_gradient_cycles(mut self, cycles: f32) -> Self {
        match &mut self.color_mode {
            NoisyCircleColorMode::Monochrome(_) => {}
            NoisyCircleColorMode::Multicolor(gradient) => {
                gradient.cycles = cycles.max(0.1);
            }
        }
        self
    }

    pub fn with_gradient_motion_rate(mut self, motion_rate: f32) -> Self {
        match &mut self.color_mode {
            NoisyCircleColorMode::Monochrome(_) => {}
            NoisyCircleColorMode::Multicolor(gradient) => {
                gradient.motion_rate = motion_rate;
            }
        }
        self
    }

    pub fn sample_points(&self) -> Vec<Vec2> {
        let count = self.samples.max(12);
        let mut points = Vec::with_capacity(count + 1);
        for i in 0..=count {
            let theta = (i as f32 / count as f32) * std::f32::consts::TAU;
            let nx = theta.cos() * self.noise_space_radius + self.noise_offset.x;
            let ny = theta.sin() * self.noise_space_radius + self.noise_offset.y;
            let displacement = perlin3(nx, ny, self.phase) * self.noise_amplitude;
            let display_r = self.radius + displacement;
            points.push(vec2(display_r * theta.cos(), display_r * theta.sin()));
        }
        points
    }

    fn segment_color(
        &self,
        stroke_color: Vec4,
        segment_index: usize,
        segment_count: usize,
    ) -> Vec4 {
        match &self.color_mode {
            NoisyCircleColorMode::Monochrome(color) => *color,
            NoisyCircleColorMode::Multicolor(gradient) => {
                if gradient.palette.is_empty() {
                    return stroke_color;
                }
                if gradient.palette.len() == 1 {
                    return gradient.palette[0];
                }

                let u = segment_index as f32 / segment_count.max(1) as f32;
                let shifted = (u * gradient.cycles + self.phase * gradient.motion_rate).fract();
                sample_palette(&gradient.palette, shifted)
            }
        }
    }
}

impl Project for NoisyCircle {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let Some(stroke) = &self.style.stroke else {
            return;
        };

        let points = self.sample_points();
        let segment_count = points.len().saturating_sub(1);
        for (segment_index, pair) in points.windows(2).enumerate() {
            let a = pair[0];
            let b = pair[1];
            ctx.emit(RenderPrimitive::Line {
                start: vec3(a.x, a.y, 0.0),
                end: vec3(b.x, b.y, 0.0),
                thickness: stroke.thickness,
                color: self.segment_color(stroke.color, segment_index, segment_count),
                dash_length: stroke.dash_length,
                gap_length: stroke.gap_length,
                dash_offset: stroke.dash_offset,
            });
        }
    }
}

impl Bounded for NoisyCircle {
    fn local_bounds(&self) -> Bounds {
        let pad = self.noise_amplitude
            + self
                .style
                .stroke
                .as_ref()
                .map(|stroke| stroke.thickness)
                .unwrap_or(0.0);
        let extent = self.radius + pad;
        Bounds::new(vec2(-extent, -extent), vec2(extent, extent))
    }
}

fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn grad3(hash: u32, x: f32, y: f32, z: f32) -> f32 {
    match hash & 15 {
        0 => x + y,
        1 => -x + y,
        2 => x - y,
        3 => -x - y,
        4 => x + z,
        5 => -x + z,
        6 => x - z,
        7 => -x - z,
        8 => y + z,
        9 => -y + z,
        10 => y - z,
        11 => -y - z,
        12 => x + y,
        13 => -x + y,
        14 => -y + z,
        _ => -y - z,
    }
}

fn hash3(x: i32, y: i32, z: i32) -> u32 {
    let mut h = x as u32;
    h = h
        .wrapping_mul(374_761_393)
        .wrapping_add((y as u32).wrapping_mul(668_265_263));
    h ^= (z as u32).wrapping_mul(2_147_483_647);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^ (h >> 16)
}

fn perlin3(x: f32, y: f32, z: f32) -> f32 {
    let xi0 = x.floor() as i32;
    let yi0 = y.floor() as i32;
    let zi0 = z.floor() as i32;
    let xi1 = xi0 + 1;
    let yi1 = yi0 + 1;
    let zi1 = zi0 + 1;

    let xf = x - xi0 as f32;
    let yf = y - yi0 as f32;
    let zf = z - zi0 as f32;

    let u = fade(xf);
    let v = fade(yf);
    let w = fade(zf);

    let n000 = grad3(hash3(xi0, yi0, zi0), xf, yf, zf);
    let n100 = grad3(hash3(xi1, yi0, zi0), xf - 1.0, yf, zf);
    let n010 = grad3(hash3(xi0, yi1, zi0), xf, yf - 1.0, zf);
    let n110 = grad3(hash3(xi1, yi1, zi0), xf - 1.0, yf - 1.0, zf);
    let n001 = grad3(hash3(xi0, yi0, zi1), xf, yf, zf - 1.0);
    let n101 = grad3(hash3(xi1, yi0, zi1), xf - 1.0, yf, zf - 1.0);
    let n011 = grad3(hash3(xi0, yi1, zi1), xf, yf - 1.0, zf - 1.0);
    let n111 = grad3(hash3(xi1, yi1, zi1), xf - 1.0, yf - 1.0, zf - 1.0);

    let x00 = lerp(n000, n100, u);
    let x10 = lerp(n010, n110, u);
    let x01 = lerp(n001, n101, u);
    let x11 = lerp(n011, n111, u);
    let y0 = lerp(x00, x10, v);
    let y1 = lerp(x01, x11, v);
    lerp(y0, y1, w)
}

fn sample_palette(palette: &[Vec4], t: f32) -> Vec4 {
    let wrapped = t.rem_euclid(1.0);
    let scaled = wrapped * palette.len() as f32;
    let idx0 = scaled.floor() as usize % palette.len();
    let idx1 = (idx0 + 1) % palette.len();
    let local_t = scaled.fract();
    palette[idx0].lerp(palette[idx1], local_t)
}

pub type PerlinNoiseCircle = NoisyCircle;
pub type PerlinNoiseCircleGradient = NoisyCircleGradient;
pub type PerlinNoiseCircleColorMode = NoisyCircleColorMode;
