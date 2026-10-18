//! Animated wire terrain for floors, lower-thirds, and opening backgrounds.
//!
//! The default height is seamless: `phase + 1` is the same shape. Point and
//! line sizes stay in world units until a view is supplied, and then they hold
//! a roughly constant screen size as the camera moves.

use crate::backend::renderer::vertex::mesh::MeshVertex;
use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Mesh, Project, ProjectionCtx, RenderPrimitive};
use glam::{Vec3, Vec4, vec2};

pub type WaveMeshProfile = fn(f32, f32, f32) -> f32;

#[derive(Debug, Clone, Copy)]
pub struct WaveMeshPalette {
    pub near: Vec4,
    pub far: Vec4,
    pub peak: Vec4,
    pub fill: Vec4,
    pub sparkle: Vec4,
}

impl Default for WaveMeshPalette {
    fn default() -> Self {
        Self {
            near: hex_rgb(0xf0, 0xa9, 0xff),
            far: hex_rgb(0x26, 0x37, 0xd4),
            peak: hex_rgb(0xff, 0xf5, 0xff),
            fill: hex_rgb(0x5a, 0x27, 0xd6),
            sparkle: Vec4::ONE,
        }
    }
}

/// A reusable animated wire terrain.
///
/// `eye` is in the tattva's local space. Pass the camera position minus the
/// tattva position when the mesh itself is translated.
#[derive(Clone, Copy)]
pub struct WaveMesh {
    pub width: f32,
    pub depth: f32,
    pub amplitude: f32,
    pub columns: usize,
    pub rows: usize,
    pub phase: f32,
    pub energy: f32,
    pub line_opacity: f32,
    pub fill_opacity: f32,
    pub point_opacity: f32,
    pub point_size: f32,
    pub sparkle_size: f32,
    pub sparkle_ratio: f32,
    pub far_fade: f32,
    /// Fraction of each edge that dissolves, so a finite mesh can read as open ground.
    pub edge_fade: f32,
    pub glow_variation: f32,
    pub line_thickness: f32,
    pub line_fraction: f32,
    pub point_fraction: f32,
    pub sparkle_fraction: f32,
    pub palette: WaveMeshPalette,
    pub eye: Option<Vec3>,
    pub fov_y_rad: f32,
    pub profile: WaveMeshProfile,
}

impl Default for WaveMesh {
    fn default() -> Self {
        Self::new()
    }
}

impl WaveMesh {
    pub fn new() -> Self {
        Self {
            width: 17.0,
            depth: 7.0,
            amplitude: 1.1,
            columns: 57,
            rows: 25,
            phase: 0.0,
            energy: 1.0,
            line_opacity: 0.82,
            fill_opacity: 0.1,
            point_opacity: 0.72,
            point_size: 0.035,
            sparkle_size: 0.075,
            sparkle_ratio: 0.035,
            far_fade: 0.42,
            edge_fade: 0.0,
            glow_variation: 0.2,
            line_thickness: 0.015,
            line_fraction: 0.0015,
            point_fraction: 0.0032,
            sparkle_fraction: 0.0065,
            palette: WaveMeshPalette::default(),
            eye: None,
            fov_y_rad: 0.0,
            profile: default_wave_mesh_profile,
        }
    }

    pub fn with_size(mut self, width: f32, depth: f32) -> Self {
        self.width = width.abs().max(0.01);
        self.depth = depth.abs().max(0.01);
        self
    }

    pub fn with_amplitude(mut self, amplitude: f32) -> Self {
        self.amplitude = amplitude.abs();
        self
    }

    pub fn with_samples(mut self, columns: usize, rows: usize) -> Self {
        self.columns = columns.max(2);
        self.rows = rows.max(2);
        self
    }

    pub fn with_palette(mut self, palette: WaveMeshPalette) -> Self {
        self.palette = palette;
        self
    }

    pub fn with_line_opacity(mut self, opacity: f32) -> Self {
        self.line_opacity = opacity.clamp(0.0, 1.0);
        self
    }

    pub fn with_fill_opacity(mut self, opacity: f32) -> Self {
        self.fill_opacity = opacity.clamp(0.0, 1.0);
        self
    }

    pub fn with_nodes(mut self, size: f32, opacity: f32) -> Self {
        self.point_size = size.abs().max(0.0001);
        self.point_opacity = opacity.clamp(0.0, 1.0);
        self
    }

    pub fn with_sparkles(mut self, size: f32, ratio: f32) -> Self {
        self.sparkle_size = size.abs().max(0.0001);
        self.sparkle_ratio = ratio.clamp(0.0, 1.0);
        self
    }

    pub fn with_far_fade(mut self, far_fade: f32) -> Self {
        self.far_fade = far_fade.clamp(0.0, 1.0);
        self
    }

    pub fn with_edge_fade(mut self, edge_fade: f32) -> Self {
        self.edge_fade = edge_fade.clamp(0.0, 0.49);
        self
    }

    pub fn with_glow_variation(mut self, glow_variation: f32) -> Self {
        self.glow_variation = glow_variation.clamp(0.0, 1.0);
        self
    }

    pub fn with_phase(mut self, phase: f32) -> Self {
        self.phase = phase;
        self
    }

    pub fn with_energy(mut self, energy: f32) -> Self {
        self.energy = energy.abs();
        self
    }

    pub fn with_profile(mut self, profile: WaveMeshProfile) -> Self {
        self.profile = profile;
        self
    }

    /// `eye` is local to this tattva. `fov_y_rad` switches line and point size
    /// to a fraction of the view height.
    pub fn with_view(mut self, eye: Vec3, fov_y_rad: f32) -> Self {
        self.eye = Some(eye);
        self.fov_y_rad = fov_y_rad.abs();
        self
    }
}

impl Project for WaveMesh {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let columns = self.columns.max(2);
        let rows = self.rows.max(2);
        let mut points = Vec::with_capacity(columns * rows);
        for row in 0..rows {
            let z = -self.depth * 0.5 + self.depth * row as f32 / (rows - 1) as f32;
            for column in 0..columns {
                let x = -self.width * 0.5 + self.width * column as f32 / (columns - 1) as f32;
                let height = (self.profile)(x, z, self.phase) * self.amplitude * self.energy;
                points.push(sample_point(self, x, height, z));
            }
        }

        emit_fill(self, ctx, &points, columns, rows);
        emit_wires(self, ctx, &points, columns, rows);
        emit_points(self, ctx, &points, false);
        emit_points(self, ctx, &points, true);
    }
}

impl Bounded for WaveMesh {
    fn local_bounds(&self) -> Bounds {
        let reach_y = (self.amplitude * self.energy).max(0.2);
        Bounds::new(
            vec2(-self.width * 0.5, -reach_y),
            vec2(self.width * 0.5, reach_y),
        )
    }
}

#[derive(Clone, Copy)]
struct Sample {
    position: Vec3,
    rgb: [f32; 3],
    alpha: f32,
}

fn sample_point(mesh: &WaveMesh, x: f32, y: f32, z: f32) -> Sample {
    let nearness = (z / mesh.depth + 0.5).clamp(0.0, 1.0);
    let depth_fade = if mesh.far_fade <= 0.0 {
        1.0
    } else {
        smoothstep(nearness / mesh.far_fade)
    };
    let border = if mesh.edge_fade <= 0.0 {
        1.0
    } else {
        let edge_x = 1.0 - (x.abs() / (mesh.width * 0.5)).clamp(0.0, 1.0);
        let edge_z = 1.0 - (z.abs() / (mesh.depth * 0.5)).clamp(0.0, 1.0);
        smoothstep(edge_x / mesh.edge_fade).min(smoothstep(edge_z / mesh.edge_fade))
    };
    let shimmer = 1.0 - mesh.glow_variation * 0.5
        + mesh.glow_variation
            * 0.5
            * (1.0 + (mesh.phase * std::f32::consts::TAU + x * 0.72 - z * 0.31).sin());
    let peak_scale = (mesh.amplitude * mesh.energy.max(0.001)).max(0.001);
    let peakness = (y / peak_scale).clamp(0.0, 1.0);
    let ground = mesh
        .palette
        .far
        .lerp(mesh.palette.near, 0.2 + nearness * 0.8);
    let ink = ground.lerp(mesh.palette.peak, peakness * 0.42);
    Sample {
        position: Vec3::new(x, y, z),
        rgb: [ink.x * shimmer, ink.y * shimmer, ink.z * shimmer],
        alpha: depth_fade * border,
    }
}

fn emit_fill(
    mesh: &WaveMesh,
    ctx: &mut ProjectionCtx,
    points: &[Sample],
    columns: usize,
    rows: usize,
) {
    if mesh.fill_opacity <= 0.0 {
        return;
    }
    let fill = mesh.palette.fill;
    let mut vertices = Vec::with_capacity(points.len());
    for point in points {
        vertices.push(MeshVertex {
            position: point.position.into(),
            color: [
                point.rgb[0] * fill.x,
                point.rgb[1] * fill.y,
                point.rgb[2] * fill.z,
                point.alpha,
            ],
        });
    }
    let mut indices = Vec::with_capacity((columns - 1) * (rows - 1) * 6);
    for row in 0..rows - 1 {
        for column in 0..columns - 1 {
            let a = (row * columns + column) as u32;
            let b = a + 1;
            let c = a + columns as u32;
            let d = c + 1;
            indices.extend([a, c, b, b, c, d]);
        }
    }
    ctx.with_opacity(mesh.fill_opacity, |ctx| {
        ctx.emit(RenderPrimitive::Mesh(Mesh::from_tessellation(
            vertices, indices,
        )));
    });
}

fn emit_wires(
    mesh: &WaveMesh,
    ctx: &mut ProjectionCtx,
    points: &[Sample],
    columns: usize,
    rows: usize,
) {
    if mesh.line_opacity <= 0.0 {
        return;
    }
    for row in 0..rows {
        for column in 0..columns {
            let index = row * columns + column;
            if column + 1 < columns {
                emit_span(mesh, ctx, &points[index], &points[index + 1]);
            }
            if row + 1 < rows {
                emit_span(mesh, ctx, &points[index], &points[index + columns]);
            }
        }
    }
}

fn emit_span(mesh: &WaveMesh, ctx: &mut ProjectionCtx, from: &Sample, to: &Sample) {
    let alpha = mesh.line_opacity * 0.5 * (from.alpha + to.alpha);
    if alpha <= 0.004 {
        return;
    }
    let lift = Vec3::Y * 0.01;
    let start = from.position + lift;
    let end = to.position + lift;
    ctx.emit(RenderPrimitive::Line {
        start,
        end,
        thickness: sized(mesh, start, mesh.line_fraction, mesh.line_thickness),
        color: Vec4::new(
            (from.rgb[0] + to.rgb[0]) * 0.5,
            (from.rgb[1] + to.rgb[1]) * 0.5,
            (from.rgb[2] + to.rgb[2]) * 0.5,
            alpha,
        ),
        dash_length: 0.0,
        gap_length: 0.0,
        dash_offset: 0.0,
    });
}

fn emit_points(mesh: &WaveMesh, ctx: &mut ProjectionCtx, points: &[Sample], sparkles: bool) {
    let opacity = if sparkles { 0.92 } else { mesh.point_opacity };
    if opacity <= 0.0 || (sparkles && mesh.sparkle_ratio <= 0.0) {
        return;
    }
    let eye = mesh.eye.unwrap_or(Vec3::Y * 8.0);
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for (index, point) in points.iter().enumerate() {
        let sparkle = hash01(index as f32 * 91.73 + 17.1) < mesh.sparkle_ratio;
        if sparkles != sparkle {
            continue;
        }
        let twinkle = if sparkles {
            0.58 + 0.42
                * (1.0 + (mesh.phase * std::f32::consts::TAU * 2.0 + index as f32 * 1.618).sin())
                * 0.5
        } else {
            1.0
        };
        let alpha = opacity * point.alpha * twinkle;
        if alpha <= 0.004 {
            continue;
        }
        let (fraction, world, lift) = if sparkles {
            (mesh.sparkle_fraction, mesh.sparkle_size, 0.03)
        } else {
            (mesh.point_fraction, mesh.point_size, 0.018)
        };
        let radius = sized(mesh, point.position, fraction, world).clamp(0.004, 0.28);
        let rgb = if sparkles {
            [
                mesh.palette.sparkle.x,
                mesh.palette.sparkle.y,
                mesh.palette.sparkle.z,
            ]
        } else {
            point.rgb
        };
        push_billboard(
            &mut vertices,
            &mut indices,
            point.position + Vec3::Y * lift,
            radius,
            [rgb[0], rgb[1], rgb[2], alpha],
            eye,
        );
    }
    if vertices.is_empty() {
        return;
    }
    ctx.emit(RenderPrimitive::Mesh(Mesh::from_tessellation(
        vertices, indices,
    )));
}

fn push_billboard(
    vertices: &mut Vec<MeshVertex>,
    indices: &mut Vec<u32>,
    center: Vec3,
    radius: f32,
    color: [f32; 4],
    eye: Vec3,
) {
    let forward = {
        let toward = eye - center;
        if toward.length_squared() < 1.0e-6 {
            Vec3::Y
        } else {
            toward.normalize()
        }
    };
    let mut right = Vec3::Y.cross(forward);
    if right.length_squared() < 1.0e-6 {
        right = Vec3::X.cross(forward);
    }
    let right = right.normalize() * radius;
    let up = forward.cross(right).normalize() * radius;
    let sides = 7u32;
    let base = vertices.len() as u32;
    vertices.push(MeshVertex {
        position: center.into(),
        color,
    });
    for step in 0..sides {
        let angle = step as f32 / sides as f32 * std::f32::consts::TAU;
        let (sine, cosine) = angle.sin_cos();
        let corner = center + right * cosine + up * sine;
        vertices.push(MeshVertex {
            position: corner.into(),
            color,
        });
    }
    for step in 0..sides {
        let next = if step + 1 == sides { 1 } else { step + 2 };
        indices.extend([base, base + step + 1, base + next]);
    }
}

fn sized(mesh: &WaveMesh, point: Vec3, fraction: f32, world: f32) -> f32 {
    match mesh.eye {
        Some(eye) if mesh.fov_y_rad > 0.001 => {
            let distance = eye.distance(point).max(0.45);
            let view_height = 2.0 * distance * (mesh.fov_y_rad * 0.5).tan().max(0.08);
            (view_height * fraction).clamp(0.004, 0.16)
        }
        _ => world,
    }
}

/// Seamless deterministic terrain. `phase + 1` returns the same height.
pub fn default_wave_mesh_profile(x: f32, z: f32, phase: f32) -> f32 {
    let turn = phase * std::f32::consts::TAU;
    (x * 0.88 + turn).sin() * 0.29
        + (z * 1.17 - turn).cos() * 0.19
        + (x * 0.42 + z * 0.73 + turn * 2.0).sin() * 0.16
        + (x * 1.31 - z * 0.37 - turn).cos() * 0.1
        + (x * 1.72 + z * 0.52 + turn * 3.0).sin() * 0.11
        + (x * 0.26 - z * 1.85 + turn * 2.0).cos() * 0.09
        + (x * 2.2 - z * 0.85 - turn).sin() * 0.06
}

fn smoothstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

fn hash01(value: f32) -> f32 {
    let sine = (value * 12.9898).sin() * 43758.5453;
    sine - sine.floor()
}

fn hex_rgb(red: u8, green: u8, blue: u8) -> Vec4 {
    Vec4::new(
        red as f32 / 255.0,
        green as f32 / 255.0,
        blue as f32 / 255.0,
        1.0,
    )
}

#[cfg(test)]
mod tests {
    use super::default_wave_mesh_profile;

    #[test]
    fn default_profile_repeats_every_cycle() {
        for x in [-4.0, 0.0, 2.5] {
            for z in [-3.0, 0.2, 1.7] {
                let a = default_wave_mesh_profile(x, z, 0.37);
                let b = default_wave_mesh_profile(x, z, 1.37);
                assert!((a - b).abs() < 1.0e-5, "{a} vs {b}");
            }
        }
    }
}
