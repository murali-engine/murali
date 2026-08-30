use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI, SQRT_2, TAU};

use glam::{Vec2, Vec3, vec2, vec3};

use crate::frontend::collection::primitives::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapProjectionKind {
    Equirectangular,
    Sinusoidal,
    Mollweide,
    Hammer,
    Mercator,
}

impl MapProjectionKind {
    pub fn from_name(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "equirectangular" => Some(Self::Equirectangular),
            "sinusoidal" => Some(Self::Sinusoidal),
            "mollweide" => Some(Self::Mollweide),
            "hammer" => Some(Self::Hammer),
            "mercator" => Some(Self::Mercator),
            _ => None,
        }
    }
}

pub fn project_point(
    kind: MapProjectionKind,
    lon: f32,
    lat: f32,
    half_width: f32,
    half_height: f32,
) -> Vec2 {
    let (x, y) = match kind {
        MapProjectionKind::Equirectangular => (lon / PI, lat / FRAC_PI_2),
        MapProjectionKind::Sinusoidal => ((lon * lat.cos()) / PI, lat / FRAC_PI_2),
        MapProjectionKind::Mollweide => {
            let theta = solve_mollweide_theta(lat);
            ((lon * theta.cos()) / PI, theta.sin())
        }
        MapProjectionKind::Hammer => {
            let denom = (1.0 + lat.cos() * (lon * 0.5).cos()).sqrt().max(1e-4);
            let x = (2.0 * SQRT_2 * lat.cos() * (lon * 0.5).sin()) / denom;
            let y = (SQRT_2 * lat.sin()) / denom;
            (x / (2.0 * SQRT_2), y / SQRT_2)
        }
        MapProjectionKind::Mercator => {
            let lat = lat.clamp((-80.0_f32).to_radians(), 80.0_f32.to_radians());
            let y = (FRAC_PI_4 + lat * 0.5).tan().ln();
            let y_max = (FRAC_PI_4 + 80.0_f32.to_radians() * 0.5).tan().ln();
            (lon / PI, y / y_max)
        }
    };
    vec2(x * half_width, y * half_height)
}

pub fn project_blend(
    source: MapProjectionKind,
    target: MapProjectionKind,
    mix: f32,
    lon: f32,
    lat: f32,
    half_width: f32,
    half_height: f32,
) -> Vec2 {
    let start = project_point(source, lon, lat, half_width, half_height);
    let end = project_point(target, lon, lat, half_width, half_height);
    start.lerp(end, mix.clamp(0.0, 1.0))
}

pub fn surface_point(
    source: MapProjectionKind,
    target: MapProjectionKind,
    mix: f32,
    u: f32,
    v: f32,
    half_width: f32,
    half_height: f32,
) -> Vec3 {
    let lat = FRAC_PI_2 - u;
    let lon = v - PI;
    let point = project_blend(source, target, mix, lon, lat, half_width, half_height);
    vec3(point.x, point.y, 0.0)
}

pub fn surface_function(
    source: MapProjectionKind,
    target: MapProjectionKind,
    mix: f32,
    half_width: f32,
    half_height: f32,
) -> impl Fn(f32, f32) -> Vec3 + Send + Sync + 'static {
    move |u, v| surface_point(source, target, mix, u, v, half_width, half_height)
}

pub fn u_range() -> (f32, f32) {
    (0.0, PI)
}

pub fn v_range() -> (f32, f32) {
    (0.0, TAU)
}

pub fn graticule_path(
    source: MapProjectionKind,
    target: MapProjectionKind,
    mix: f32,
    half_width: f32,
    half_height: f32,
) -> Path {
    let mut path = Path::new();
    for latitude_deg in [-60.0_f32, -30.0, 0.0, 30.0, 60.0] {
        let latitude = latitude_deg.to_radians();
        for step in 0..=160 {
            let lon = (-180.0 + step as f32 * 360.0 / 160.0).to_radians();
            let point = project_blend(source, target, mix, lon, latitude, half_width, half_height);
            path = if step == 0 {
                path.move_to(point)
            } else {
                path.line_to(point)
            };
        }
    }
    for longitude_deg in [-150.0_f32, -90.0, -30.0, 30.0, 90.0, 150.0] {
        let longitude = longitude_deg.to_radians();
        for step in 0..=120 {
            let lat = (-85.0 + step as f32 * 170.0 / 120.0).to_radians();
            let point = project_blend(source, target, mix, longitude, lat, half_width, half_height);
            path = if step == 0 {
                path.move_to(point)
            } else {
                path.line_to(point)
            };
        }
    }
    path
}

fn solve_mollweide_theta(lat: f32) -> f32 {
    if (FRAC_PI_2 - lat.abs()) < 1e-4 {
        return lat.signum() * FRAC_PI_2;
    }
    let mut theta = lat;
    for _ in 0..8 {
        let numerator = 2.0 * theta + (2.0 * theta).sin() - PI * lat.sin();
        let denominator = 2.0 + 2.0 * (2.0 * theta).cos();
        theta -= numerator / denominator.max(1e-4);
    }
    theta
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equirectangular_maps_origin_and_axes() {
        let origin = project_point(MapProjectionKind::Equirectangular, 0.0, 0.0, 5.9, 3.5);
        let east = project_point(MapProjectionKind::Equirectangular, PI, 0.0, 5.9, 3.5);
        let north = project_point(MapProjectionKind::Equirectangular, 0.0, FRAC_PI_2, 5.9, 3.5);

        assert_eq!(origin, Vec2::ZERO);
        assert!((east.x - 5.9).abs() < 1e-5);
        assert!((north.y - 3.5).abs() < 1e-5);
    }

    #[test]
    fn sinusoidal_collapses_x_at_the_pole() {
        let pole = project_point(MapProjectionKind::Sinusoidal, PI, FRAC_PI_2, 5.9, 3.5);
        assert!(pole.x.abs() < 1e-5);
        assert!((pole.y - 3.5).abs() < 1e-5);
    }
}
