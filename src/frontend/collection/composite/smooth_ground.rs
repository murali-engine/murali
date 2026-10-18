//! Filled dune sheet for floors and opening backgrounds.
//!
//! Unlike [`WaveMesh`](super::WaveMesh), this draws no wires and no dots.
//! Vertex alpha stays exactly 1. A lower alpha would mark the whole mesh
//! transparent, drop the depth write, and let the sheet paint over objects
//! that sit on it. The horizon is a color mix into `palette.horizon`, which
//! should match the scene clear color.

use crate::backend::renderer::vertex::mesh::MeshVertex;
use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Mesh, Project, ProjectionCtx, RenderPrimitive};
use glam::{Vec3, Vec4, vec2};

pub type SmoothGroundProfile = fn(f32, f32, f32) -> f32;

#[derive(Debug, Clone, Copy)]
pub struct SmoothGroundPalette {
    pub far: Vec4,
    pub near: Vec4,
    pub peak: Vec4,
    /// Color the far ground dissolves into. Match the scene clear color.
    pub horizon: Vec4,
}

impl Default for SmoothGroundPalette {
    fn default() -> Self {
        Self {
            far: Vec4::new(0.08, 0.10, 0.34, 1.0),
            near: Vec4::new(0.32, 0.15, 0.56, 1.0),
            peak: Vec4::new(0.72, 0.46, 0.90, 1.0),
            horizon: Vec4::new(7.0 / 255.0, 10.0 / 255.0, 22.0 / 255.0, 1.0),
        }
    }
}

/// A reusable filled ground. Long rolls, lit crests, no lattice.
///
/// `eye` is in the tattva's local space. Pass the camera position minus the
/// tattva position when the ground itself is translated. Advance `phase` to
/// drift the dunes. The default profile does not repeat every cycle, because
/// the shorter rolls move at their own rate.
#[derive(Clone, Copy)]
pub struct SmoothGround {
    pub width: f32,
    pub depth: f32,
    pub amplitude: f32,
    pub columns: usize,
    pub rows: usize,
    pub phase: f32,
    pub energy: f32,
    pub palette: SmoothGroundPalette,
    pub light: Vec3,
    pub eye: Vec3,
    pub profile: SmoothGroundProfile,
    pub shade_base: f32,
    pub shade_gain: f32,
    /// How far toward `palette.peak` a crest is allowed to move.
    pub crest_mix: f32,
    /// Fog begins at `eye_distance * fog_start_scale`.
    pub fog_start_scale: f32,
    /// Fog ends at `eye_distance * fog_end_scale + fog_margin`.
    pub fog_end_scale: f32,
    pub fog_margin: f32,
    /// Distances closer than this still use this reach, so a camera over the
    /// origin does not collapse the fog into a point.
    pub min_eye_distance: f32,
    /// Outer fraction of the rectangle that dissolves into the horizon color.
    pub edge_fade: f32,
    pub normal_eps: f32,
}

impl Default for SmoothGround {
    fn default() -> Self {
        Self::new()
    }
}

impl SmoothGround {
    pub fn new() -> Self {
        Self {
            width: 80.0,
            depth: 56.0,
            amplitude: 1.85,
            columns: 121,
            rows: 85,
            phase: 0.0,
            energy: 1.0,
            palette: SmoothGroundPalette::default(),
            light: Vec3::new(0.78, 0.36, 0.50),
            eye: Vec3::ZERO,
            profile: default_smooth_ground_profile,
            shade_base: 0.34,
            shade_gain: 0.66,
            crest_mix: 0.72,
            fog_start_scale: 0.55,
            fog_end_scale: 1.65,
            fog_margin: 24.0,
            min_eye_distance: 12.0,
            edge_fade: 0.28,
            normal_eps: 1.1,
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

    pub fn with_palette(mut self, palette: SmoothGroundPalette) -> Self {
        self.palette = palette;
        self
    }

    pub fn with_horizon(mut self, horizon: Vec4) -> Self {
        self.palette.horizon = horizon;
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

    pub fn with_profile(mut self, profile: SmoothGroundProfile) -> Self {
        self.profile = profile;
        self
    }

    pub fn with_light(mut self, light: Vec3) -> Self {
        self.light = light;
        self
    }

    /// `eye` is local to this tattva.
    pub fn with_eye(mut self, eye: Vec3) -> Self {
        self.eye = eye;
        self
    }

    pub fn with_edge_fade(mut self, edge_fade: f32) -> Self {
        self.edge_fade = edge_fade.clamp(0.0, 0.49);
        self
    }
}

impl Project for SmoothGround {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let columns = self.columns.max(2);
        let rows = self.rows.max(2);
        let mut vertices = Vec::with_capacity(columns * rows);
        for row in 0..rows {
            let z = -self.depth * 0.5 + self.depth * row as f32 / (rows - 1) as f32;
            for column in 0..columns {
                let x = -self.width * 0.5 + self.width * column as f32 / (columns - 1) as f32;
                let y = ground_height(self, x, z);
                vertices.push(MeshVertex {
                    position: [x, y, z],
                    color: ground_color(self, x, y, z),
                });
            }
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
        ctx.emit(RenderPrimitive::Mesh(Mesh::from_tessellation(
            vertices, indices,
        )));
    }
}

impl Bounded for SmoothGround {
    fn local_bounds(&self) -> Bounds {
        let reach = (self.amplitude * self.energy).abs().max(0.2);
        Bounds::new(
            vec2(-self.width * 0.5, -reach),
            vec2(self.width * 0.5, reach),
        )
    }
}

fn ground_height(mesh: &SmoothGround, x: f32, z: f32) -> f32 {
    (mesh.profile)(x, z, mesh.phase) * mesh.amplitude * mesh.energy
}

fn ground_color(mesh: &SmoothGround, x: f32, y: f32, z: f32) -> [f32; 4] {
    let eps = mesh.normal_eps.abs().max(0.05);
    let dx = ground_height(mesh, x + eps, z) - ground_height(mesh, x - eps, z);
    let dz = ground_height(mesh, x, z + eps) - ground_height(mesh, x, z - eps);
    let normal = Vec3::new(-dx, eps * 2.0, -dz).normalize();
    let light = {
        if mesh.light.length_squared() < 1.0e-6 {
            Vec3::Y
        } else {
            mesh.light.normalize()
        }
    };
    let shade = mesh.shade_base + mesh.shade_gain * normal.dot(light).max(0.0);
    let nearness = (z / mesh.depth + 0.5).clamp(0.0, 1.0);
    let reach = (mesh.amplitude * mesh.energy).abs().max(0.001);
    let crest = (y / reach * 0.5 + 0.5).clamp(0.0, 1.0);
    let far = mesh.palette.far.truncate();
    let near = mesh.palette.near.truncate();
    let peak = mesh.palette.peak.truncate();
    let base = far.lerp(near, 0.18 + nearness * 0.82);
    let ink = base.lerp(peak, crest * mesh.crest_mix);
    let lit = ink * shade;
    let horizontal = ((x - mesh.eye.x).powi(2) + (z - mesh.eye.z).powi(2)).sqrt();
    let eye_dist = (mesh.eye.x * mesh.eye.x + mesh.eye.z * mesh.eye.z)
        .sqrt()
        .max(mesh.min_eye_distance.max(0.001));
    let start = eye_dist * mesh.fog_start_scale;
    let end = eye_dist * mesh.fog_end_scale + mesh.fog_margin;
    let fog = if end <= start + 0.001 {
        0.0
    } else {
        smoothstep(((horizontal - start) / (end - start)).clamp(0.0, 1.0))
    };
    let edge_x = x.abs() / (mesh.width * 0.5);
    let edge_z = z.abs() / (mesh.depth * 0.5);
    let edge = if mesh.edge_fade <= 0.0 {
        0.0
    } else {
        let start = 1.0 - mesh.edge_fade;
        smoothstep(((edge_x.max(edge_z) - start) / mesh.edge_fade).clamp(0.0, 1.0))
    };
    let horizon = mesh.palette.horizon.truncate();
    let rgb = lit.lerp(horizon, fog.max(edge));
    [rgb.x, rgb.y, rgb.z, 1.0]
}

/// Long dunes. The two shorter terms drift against the first, so `phase + 1`
/// is a new shape. Pass [`SmoothGround::with_profile`] for a loop that closes.
pub fn default_smooth_ground_profile(x: f32, z: f32, phase: f32) -> f32 {
    let turn = phase * std::f32::consts::TAU;
    (x * 0.085 + z * 0.04 + turn).sin() * 0.52
        + (z * 0.15 - x * 0.03 - turn * 0.55).cos() * 0.32
        + (x * 0.17 + z * 0.09 + turn * 0.85).sin() * 0.16
}

fn smoothstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

#[cfg(test)]
mod tests {
    use super::{SmoothGround, default_smooth_ground_profile, ground_color};

    #[test]
    fn default_profile_stays_within_unit_height() {
        for x in [-40.0, 0.0, 18.0] {
            for z in [-20.0, 1.0, 9.0] {
                for phase in [0.0, 0.37, 1.2] {
                    let height = default_smooth_ground_profile(x, z, phase);
                    assert!(height.abs() <= 1.0 + 1.0e-4, "{height}");
                }
            }
        }
    }

    #[test]
    fn sheet_stays_opaque_and_corners_match_the_horizon() {
        let mesh = SmoothGround::new();
        let center = ground_color(&mesh, 0.0, 0.0, 0.0);
        assert_eq!(center[3], 1.0);
        let horizon = mesh.palette.horizon;
        assert!((center[0] - horizon.x).abs() > 0.02);

        let corner = ground_color(&mesh, mesh.width * 0.5, 0.0, mesh.depth * 0.5);
        assert_eq!(corner[3], 1.0);
        assert!((corner[0] - horizon.x).abs() < 1.0e-5);
        assert!((corner[1] - horizon.y).abs() < 1.0e-5);
        assert!((corner[2] - horizon.z).abs() < 1.0e-5);
    }
}
