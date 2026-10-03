// murali-example-tags: math,3d,fractal

use std::sync::Arc;

use glam::{Quat, Vec3, Vec4, vec2};
use murali::backend::renderer::vertex::mesh::MeshVertex;
use murali::colors::*;
use murali::engine::camera::Projection;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::animation::Ease;
use murali::frontend::collection::primitives::ellipse::Ellipse;
use murali::frontend::collection::text::label::Label;
use murali::frontend::layout::{Bounded, Bounds};
use murali::projection::{Mesh, Project, ProjectionCtx, RenderPrimitive};
use murali::{App, DepthMode};

const FRACTAL_ITERATIONS: usize = 14;
const LATITUDE_SEGMENTS: usize = 96;
const LONGITUDE_SEGMENTS: usize = 152;
const RADIAL_SAMPLES: usize = 38;
const SEARCH_RADIUS: f32 = 1.55;
const FRACTAL_POWERS: [f32; 3] = [6.0, 8.0, 10.0];

#[derive(Clone, Copy, Debug)]
pub enum ColorPalette {
    Cycle,
    Aurora,
    Solar,
    Ocean,
    Candy,
}

impl ColorPalette {
    pub fn from_args() -> Self {
        match std::env::args().nth(1).as_deref() {
            Some("aurora") => Self::Aurora,
            Some("solar") => Self::Solar,
            Some("ocean") => Self::Ocean,
            Some("candy") => Self::Candy,
            Some("cycle") | None => Self::Cycle,
            Some(unknown) => {
                eprintln!(
                    "Unknown palette '{unknown}'. Try: cycle, aurora, solar, ocean, or candy. Using cycle."
                );
                Self::Cycle
            }
        }
    }
}

pub struct Mandelbulb {
    directions: Vec<Vec3>,
    radii: Vec<[f32; 3]>,
    indices: Vec<u32>,
    mesh: Arc<Mesh>,
    palette: ColorPalette,
}

impl Mandelbulb {
    pub fn new(palette: ColorPalette) -> Self {
        let rows = LATITUDE_SEGMENTS + 1;
        let columns = LONGITUDE_SEGMENTS + 1;
        let mut directions = Vec::with_capacity(rows * columns);
        let mut radii = Vec::with_capacity(rows * columns);

        // Precompute three related Mandelbulbs. Runtime animation only has to
        // interpolate their radii, which keeps the evolving fractal responsive.
        for latitude in 0..=LATITUDE_SEGMENTS {
            let theta = std::f32::consts::PI * latitude as f32 / LATITUDE_SEGMENTS as f32;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();

            for longitude in 0..=LONGITUDE_SEGMENTS {
                let phi = std::f32::consts::TAU * longitude as f32 / LONGITUDE_SEGMENTS as f32;
                let direction = Vec3::new(sin_theta * phi.cos(), cos_theta, sin_theta * phi.sin());
                directions.push(direction);
                radii.push(FRACTAL_POWERS.map(|power| surface_radius(direction, power)));
            }
        }

        let mut indices = Vec::with_capacity(LATITUDE_SEGMENTS * LONGITUDE_SEGMENTS * 6);
        for latitude in 0..LATITUDE_SEGMENTS {
            for longitude in 0..LONGITUDE_SEGMENTS {
                let a = (latitude * columns + longitude) as u32;
                let b = a + 1;
                let c = ((latitude + 1) * columns + longitude) as u32;
                let d = c + 1;
                indices.extend_from_slice(&[a, c, b, b, c, d]);
            }
        }

        let mut bulb = Self {
            directions,
            radii,
            indices,
            mesh: Arc::new(Mesh::empty()),
            palette,
        };
        bulb.evolve(0.0);
        bulb
    }

    pub fn evolve(&mut self, time: f32) {
        let columns = LONGITUDE_SEGMENTS + 1;
        let morph = (time * 0.48).sin();
        let positions: Vec<Vec3> = self
            .directions
            .iter()
            .zip(&self.radii)
            .map(|(direction, radii)| {
                let fractal_radius = if morph >= 0.0 {
                    lerp(radii[1], radii[2], smoothstep(morph))
                } else {
                    lerp(radii[1], radii[0], smoothstep(-morph))
                };
                let longitude = direction.z.atan2(direction.x);
                let latitude = direction.y.asin();
                let pulse = 1.0
                    + 0.035 * (time * 1.7 + longitude * 3.0 + latitude * 2.0).sin()
                    + 0.018 * (time * 2.6 - longitude * 5.0 + fractal_radius * 8.0).sin();
                *direction * fractal_radius * pulse
            })
            .collect();

        let light = Vec3::new(-0.45, 0.8, 0.55).normalize();
        let mut vertices = Vec::with_capacity(positions.len());
        for latitude in 0..=LATITUDE_SEGMENTS {
            for longitude in 0..=LONGITUDE_SEGMENTS {
                let index = latitude * columns + longitude;
                let position = positions[index];
                let direction = self.directions[index];

                let previous_latitude = latitude.saturating_sub(1);
                let next_latitude = (latitude + 1).min(LATITUDE_SEGMENTS);
                let previous_longitude = if longitude == 0 {
                    LONGITUDE_SEGMENTS - 1
                } else {
                    longitude - 1
                };
                let next_longitude = if longitude == LONGITUDE_SEGMENTS {
                    1
                } else {
                    longitude + 1
                };

                let tangent_latitude = positions[next_latitude * columns + longitude]
                    - positions[previous_latitude * columns + longitude];
                let tangent_longitude = positions[latitude * columns + next_longitude]
                    - positions[latitude * columns + previous_longitude];
                let mut normal = tangent_latitude
                    .cross(tangent_longitude)
                    .normalize_or_zero();
                if normal.length_squared() < 0.5 {
                    normal = direction;
                }
                if normal.dot(position) < 0.0 {
                    normal = -normal;
                }

                let diffuse = normal.dot(light).max(0.0);
                let rim = (1.0 - normal.z.abs()).powf(2.0);
                let longitude_phase = direction.z.atan2(direction.x) / std::f32::consts::TAU;
                let latitude_phase = direction.y * 0.16;
                let color_phase = (0.58
                    + time * 0.055
                    + longitude_phase
                    + latitude_phase
                    + position.length() * 0.11)
                    .rem_euclid(1.0);
                let value = (0.48 + diffuse * 0.48 + rim * 0.12).clamp(0.0, 1.0);
                let palette_color = animated_palette(self.palette, color_phase, time);
                let color = Vec4::new(
                    palette_color.x * value,
                    palette_color.y * value,
                    palette_color.z * value,
                    1.0,
                )
                .lerp(WHITE, diffuse * 0.16);

                vertices.push(MeshVertex {
                    position: position.to_array(),
                    color: color.to_array(),
                });
            }
        }

        self.mesh = Mesh::from_tessellation(vertices, self.indices.clone());
    }
}

impl Project for Mandelbulb {
    fn project(&self, ctx: &mut ProjectionCtx) {
        ctx.emit(RenderPrimitive::Mesh(self.mesh.clone()));
    }
}

impl Bounded for Mandelbulb {
    fn local_bounds(&self) -> Bounds {
        Bounds::new(vec2(-1.55, -1.55), vec2(1.55, 1.55))
    }
}

fn surface_radius(direction: Vec3, power: f32) -> f32 {
    let mut outside_radius = SEARCH_RADIUS;

    for sample in 1..=RADIAL_SAMPLES {
        let radius = SEARCH_RADIUS * (1.0 - sample as f32 / RADIAL_SAMPLES as f32);
        if is_in_mandelbulb(direction * radius, power) {
            let mut inside = radius;
            let mut outside = outside_radius;

            for _ in 0..9 {
                let midpoint = 0.5 * (inside + outside);
                if is_in_mandelbulb(direction * midpoint, power) {
                    inside = midpoint;
                } else {
                    outside = midpoint;
                }
            }

            return 0.5 * (inside + outside);
        }
        outside_radius = radius;
    }

    0.0
}

fn is_in_mandelbulb(point: Vec3, power: f32) -> bool {
    let mut z = Vec3::ZERO;

    for _ in 0..FRACTAL_ITERATIONS {
        let radius = z.length();
        if radius > 2.0 {
            return false;
        }

        if radius < 1.0e-6 {
            z = point;
            continue;
        }

        let theta = (z.y / radius).clamp(-1.0, 1.0).acos();
        let phi = z.z.atan2(z.x);
        let powered_radius = radius.powf(power);
        let powered_theta = theta * power;
        let powered_phi = phi * power;
        let sin_theta = powered_theta.sin();

        z = powered_radius
            * Vec3::new(
                sin_theta * powered_phi.cos(),
                powered_theta.cos(),
                sin_theta * powered_phi.sin(),
            )
            + point;
    }

    true
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn animated_palette(palette: ColorPalette, phase: f32, time: f32) -> Vec4 {
    if !matches!(palette, ColorPalette::Cycle) {
        return palette_gradient(palette, phase);
    }

    let palettes = [
        ColorPalette::Aurora,
        ColorPalette::Solar,
        ColorPalette::Ocean,
        ColorPalette::Candy,
    ];
    let cycle = time * 0.12;
    let current = cycle.floor() as usize % palettes.len();
    let next = (current + 1) % palettes.len();
    let blend = smoothstep(cycle.fract());
    palette_gradient(palettes[current], phase).lerp(palette_gradient(palettes[next], phase), blend)
}

fn palette_gradient(palette: ColorPalette, phase: f32) -> Vec4 {
    let colors = match palette {
        ColorPalette::Aurora | ColorPalette::Cycle => [
            Vec4::new(0.18, 0.04, 0.40, 1.0),
            Vec4::new(0.48, 0.22, 0.96, 1.0),
            Vec4::new(0.00, 0.76, 1.00, 1.0),
            Vec4::new(0.10, 0.95, 0.64, 1.0),
            Vec4::new(0.78, 1.00, 0.30, 1.0),
        ],
        ColorPalette::Solar => [
            Vec4::new(0.20, 0.02, 0.36, 1.0),
            Vec4::new(0.86, 0.02, 0.52, 1.0),
            Vec4::new(1.00, 0.18, 0.34, 1.0),
            Vec4::new(1.00, 0.48, 0.05, 1.0),
            Vec4::new(1.00, 0.92, 0.16, 1.0),
        ],
        ColorPalette::Ocean => [
            Vec4::new(0.01, 0.08, 0.30, 1.0),
            Vec4::new(0.02, 0.25, 0.80, 1.0),
            Vec4::new(0.00, 0.72, 1.00, 1.0),
            Vec4::new(0.00, 0.95, 0.82, 1.0),
            Vec4::new(0.66, 1.00, 0.90, 1.0),
        ],
        ColorPalette::Candy => [
            Vec4::new(0.35, 0.08, 0.82, 1.0),
            Vec4::new(1.00, 0.08, 0.68, 1.0),
            Vec4::new(1.00, 0.34, 0.52, 1.0),
            Vec4::new(1.00, 0.88, 0.20, 1.0),
            Vec4::new(0.18, 0.84, 1.00, 1.0),
        ],
    };

    let scaled = phase.rem_euclid(1.0) * colors.len() as f32;
    let index = scaled.floor() as usize % colors.len();
    let next = (index + 1) % colors.len();
    colors[index].lerp(colors[next], smoothstep(scaled.fract()))
}

pub fn tilted_rotation(y_angle: f32) -> Quat {
    Quat::from_rotation_y(y_angle) * Quat::from_rotation_x(-0.18)
}

fn main() -> anyhow::Result<()> {
    let palette = ColorPalette::from_args();
    println!("Mandelbulb palette: {palette:?}");

    let mut scene = Scene::new().with_background(Vec4::new(0.012, 0.014, 0.022, 1.0));

    let ground_id = scene.add_tattva(
        Ellipse::new(5.5, 3.7, Vec4::new(0.028, 0.032, 0.045, 1.0)).with_segments(96),
        Vec3::new(0.0, -1.72, 0.0),
    );
    scene.set_rotation(
        ground_id,
        Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
    );

    let shadow_id = scene.add_tattva(
        Ellipse::new(1.18, 0.62, Vec4::new(0.0, 0.0, 0.0, 0.62)).with_segments(72),
        Vec3::new(0.0, -1.69, 0.1),
    );
    scene.set_rotation(
        shadow_id,
        Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
    );

    let bulb_id = scene.add_tattva(Mandelbulb::new(palette), Vec3::new(0.0, -0.05, 0.0));
    scene.set_rotation(bulb_id, tilted_rotation(-0.55));
    scene.set_scale(bulb_id, Vec3::splat(0.08));
    scene.set_opacity(bulb_id, 0.0);

    scene.add_updater(bulb_id, move |scene, id, _dt| {
        let time = scene.scene_time;
        if let Some(bulb) = scene.get_tattva_typed_mut::<Mandelbulb>(id) {
            bulb.state.evolve(time);
        }

        scene.set_rotation(id, tilted_rotation(-0.55 + time * 0.34));
        scene.set_position_3d(id, Vec3::new(0.0, -0.05 + 0.09 * (time * 1.15).sin(), 0.0));
        let shadow_pulse = 1.0 - 0.07 * (time * 1.15).sin();
        scene.set_scale(
            shadow_id,
            Vec3::new(shadow_pulse, shadow_pulse, shadow_pulse),
        );
    });

    let title_id = scene.add_tattva(
        Label::new("MANDELBULB", 0.34).with_color(WHITE),
        Vec3::new(0.0, 2.9, 0.0),
    );
    let subtitle_id = scene.add_tattva(
        Label::new("z <- z^8 + c", 0.17).with_color(GRAY_B),
        Vec3::new(0.0, 2.48, 0.0),
    );

    scene.camera_mut().projection = Projection::Perspective {
        fov_y_rad: 38.0_f32.to_radians(),
        aspect: 16.0 / 9.0,
        near: 0.1,
        far: 100.0,
    };
    scene.camera_mut().position = Vec3::new(0.15, 0.35, 6.2);
    scene.camera_mut().target = Vec3::new(0.0, -0.05, 0.0);
    for id in [title_id, subtitle_id] {
        scene.set_depth_mode(id, DepthMode::Overlay);
    }

    let mut timeline = Timeline::new();
    timeline
        .animate(title_id)
        .at(0.0)
        .for_duration(0.8)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(subtitle_id)
        .at(0.3)
        .for_duration(0.9)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(bulb_id)
        .at(0.45)
        .for_duration(1.6)
        .ease(Ease::OutCubic)
        .fade_to(1.0)
        .spawn();
    timeline
        .animate(bulb_id)
        .at(0.45)
        .for_duration(1.6)
        .ease(Ease::OutCubic)
        .scale_to(Vec3::splat(1.48))
        .spawn();

    timeline
        .animate_camera()
        .at(1.8)
        .for_duration(6.0)
        .ease(Ease::InOutCubic)
        .frame_to(Vec3::new(-0.65, 0.58, 5.7), Vec3::new(0.0, -0.05, 0.0))
        .spawn();
    timeline
        .animate_camera()
        .at(7.8)
        .for_duration(6.0)
        .ease(Ease::InOutCubic)
        .frame_to(Vec3::new(0.85, 0.18, 5.45), Vec3::new(0.0, -0.08, 0.0))
        .spawn();

    scene.play(timeline)?;

    App::new()?.with_scene(scene).run_app()
}
