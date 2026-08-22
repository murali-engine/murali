use glam::{Vec2, Vec3, Vec4, vec2};
use murali::App;
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::DirtyFlags;
use murali::frontend::animation::Ease;
use murali::frontend::collection::maths::calculus::parametric_surface::ParametricSurface;
use murali::frontend::collection::text::label::Label;
use murali::frontend::layout::{Bounded, Bounds, Direction};
use murali::positions::CAMERA_DEFAULT_POS;
use murali::projection::{Project, ProjectionCtx, RenderPrimitive};
use murali::{BuiltinTexture, TextureImage};
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI, SQRT_2, TAU};
use std::sync::{Arc, Mutex};

const MAP_HALF_WIDTH: f32 = 5.9;
const MAP_HALF_HEIGHT: f32 = 3.5;
const TRANSITION_DURATION: f32 = 4.8;
const GRATICULE_COLOR: Vec4 = Vec4::new(0.78, 0.92, 1.0, 0.22);
const MAP_TINT: Vec4 = Vec4::new(1.0, 1.0, 1.0, 0.98);

#[derive(Debug, Clone, Copy)]
enum ProjectionKind {
    Equirectangular,
    Sinusoidal,
    Mollweide,
    Hammer,
    Mercator,
}

impl ProjectionKind {
    fn label(self) -> &'static str {
        match self {
            ProjectionKind::Equirectangular => "Equirectangular",
            ProjectionKind::Sinusoidal => "Sinusoidal",
            ProjectionKind::Mollweide => "Mollweide",
            ProjectionKind::Hammer => "Hammer",
            ProjectionKind::Mercator => "Mercator",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ProjectionBlend {
    from: ProjectionKind,
    to: ProjectionKind,
    mix: f32,
}

impl ProjectionBlend {
    fn new(initial: ProjectionKind) -> Self {
        Self {
            from: initial,
            to: initial,
            mix: 0.0,
        }
    }

    fn project_lon_lat(self, lon: f32, lat: f32) -> Vec2 {
        let start = project_point(self.from, lon, lat);
        let end = project_point(self.to, lon, lat);
        start.lerp(end, self.mix.clamp(0.0, 1.0))
    }
}

#[derive(Clone)]
struct ProjectionGraticule {
    state: Arc<Mutex<ProjectionBlend>>,
}

impl ProjectionGraticule {
    fn new(state: Arc<Mutex<ProjectionBlend>>) -> Self {
        Self { state }
    }

    fn emit_polyline(
        &self,
        ctx: &mut ProjectionCtx,
        blend: ProjectionBlend,
        points: &[(f32, f32)],
        color: Vec4,
        thickness: f32,
    ) {
        for pair in points.windows(2) {
            let a = blend.project_lon_lat(pair[0].0, pair[0].1);
            let b = blend.project_lon_lat(pair[1].0, pair[1].1);
            ctx.emit(RenderPrimitive::Line {
                start: Vec3::new(a.x, a.y, 0.04),
                end: Vec3::new(b.x, b.y, 0.04),
                thickness,
                color,
                dash_length: 0.0,
                gap_length: 0.0,
                dash_offset: 0.0,
            });
        }
    }
}

impl Project for ProjectionGraticule {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let blend = *self.state.lock().expect("projection state poisoned");

        for latitude in [-60.0_f32, -30.0, 0.0, 30.0, 60.0] {
            let mut line = Vec::with_capacity(161);
            for step in 0..=160 {
                let lon = -180.0 + step as f32 * 360.0 / 160.0;
                line.push((lon.to_radians(), latitude.to_radians()));
            }
            self.emit_polyline(ctx, blend, &line, GRATICULE_COLOR, 0.02);
        }

        for longitude in [-150.0_f32, -90.0, -30.0, 30.0, 90.0, 150.0] {
            let mut line = Vec::with_capacity(121);
            for step in 0..=120 {
                let lat = -85.0 + step as f32 * 170.0 / 120.0;
                line.push((longitude.to_radians(), lat.to_radians()));
            }
            self.emit_polyline(ctx, blend, &line, GRATICULE_COLOR, 0.02);
        }
    }
}

impl Bounded for ProjectionGraticule {
    fn local_bounds(&self) -> Bounds {
        Bounds::new(
            vec2(-MAP_HALF_WIDTH - 0.4, -MAP_HALF_HEIGHT - 0.4),
            vec2(MAP_HALF_WIDTH + 0.4, MAP_HALF_HEIGHT + 0.4),
        )
    }
}

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    let title_id = scene.add_tattva(
        Label::new("Map Projection Morph", 0.38).with_color(WHITE),
        Vec3::ZERO,
    );
    scene.to_edge(title_id, Direction::Up, 0.9);

    let subtitle_id = scene.add_tattva(
        Label::new(
            "The Earth texture itself bends through several projections so the distortion is visible immediately.",
            0.18,
        )
        .with_color(GRAY_B),
        Vec3::new(0.0, 2.42, 0.0),
    );

    scene.to_edge(subtitle_id, Direction::Down, 0.6);

    let sequence = [
        ProjectionKind::Equirectangular,
        ProjectionKind::Sinusoidal,
        ProjectionKind::Mollweide,
        ProjectionKind::Hammer,
        ProjectionKind::Mercator,
        ProjectionKind::Equirectangular,
    ];

    let projection_state = Arc::new(Mutex::new(ProjectionBlend::new(sequence[0])));

    let surface_state = projection_state.clone();
    let surface_id = scene.add_tattva(
        ParametricSurface::new((0.0, PI), (0.0, TAU), move |u, v| {
            let lat = FRAC_PI_2 - u;
            let lon = v - PI;
            let blend = *surface_state.lock().expect("projection state poisoned");
            let p = blend.project_lon_lat(lon, lat);
            Vec3::new(p.x, p.y, 0.0)
        })
        .with_samples(72, 144)
        .with_write_progress(0.0)
        .with_texture_flip_y(true)
        .with_color(MAP_TINT)
        .with_texture(TextureImage::builtin(BuiltinTexture::EarthMap)),
        Vec3::new(0.0, -0.08, 0.0),
    );

    let graticule_id = scene.add_tattva(
        ProjectionGraticule::new(projection_state.clone()),
        Vec3::new(0.0, -0.08, 0.0),
    );
    scene.hide(graticule_id);

    let label_id = scene.add_tattva(
        Label::new(sequence[0].label(), 0.24).with_color(GOLD_C),
        Vec3::new(0.0, -3.72, 0.0),
    );
    scene.hide(label_id);

    let footer_id = scene.add_tattva(
        Label::new(
            "This uses the Earth surface image directly, so stretched Greenland and swollen polar bands become obvious.",
            0.16,
        )
        .with_color(GRAY_A),
        Vec3::new(0.0, -3.12, 0.0),
    );
    scene.hide(footer_id);

    let mut timeline = Timeline::new();
    timeline
        .animate(title_id)
        .at(0.0)
        .for_duration(0.95)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(subtitle_id)
        .at(0.25)
        .for_duration(1.6)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(surface_id)
        .at(1.2)
        .for_duration(1.35)
        .ease(Ease::InOutCubic)
        .write_surface()
        .spawn();
    timeline
        .animate(graticule_id)
        .at(2.0)
        .for_duration(0.6)
        .ease(Ease::OutCubic)
        .appear()
        .spawn();
    timeline
        .animate(label_id)
        .at(1.55)
        .for_duration(0.65)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    for (index, pair) in sequence.windows(2).enumerate() {
        let from = pair[0];
        let to = pair[1];
        let start_time = 2.8 + index as f32 * TRANSITION_DURATION;
        let state = projection_state.clone();

        timeline.call_at(start_time, move |scene| {
            if let Ok(mut blend) = state.lock() {
                blend.from = from;
                blend.to = to;
                blend.mix = 0.0;
            }

            if let Some(surface) = scene.get_tattva_typed_mut::<ParametricSurface>(surface_id) {
                surface.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS);
            }
            if let Some(graticule) = scene.get_tattva_typed_mut::<ProjectionGraticule>(graticule_id)
            {
                graticule.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS);
            }
            if let Some(label) = scene.get_tattva_typed_mut::<Label>(label_id) {
                label.state.text = format!("{} to {}", from.label(), to.label());
                label.mark_dirty(DirtyFlags::REBUILD);
            }
        });

        let state = projection_state.clone();
        timeline.call_during(start_time, TRANSITION_DURATION, move |scene, t| {
            if let Ok(mut blend) = state.lock() {
                blend.from = from;
                blend.to = to;
                blend.mix = ease_in_out_cubic(t);
            }

            if let Some(surface) = scene.get_tattva_typed_mut::<ParametricSurface>(surface_id) {
                surface.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS);
            }
            if let Some(graticule) = scene.get_tattva_typed_mut::<ProjectionGraticule>(graticule_id)
            {
                graticule.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS);
            }
        });
    }

    let footer_start = 2.8 + (sequence.len() as f32 - 1.0) * TRANSITION_DURATION - 1.2;
    timeline
        .animate(footer_id)
        .at(footer_start)
        .for_duration(1.5)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();

    scene.play(timeline)?;
    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    scene.camera_mut().set_view_width(15.0);

    App::new()?.with_scene(scene).run_app()
}

fn project_point(kind: ProjectionKind, lon: f32, lat: f32) -> Vec2 {
    let (x, y) = match kind {
        ProjectionKind::Equirectangular => (lon / PI, lat / FRAC_PI_2),
        ProjectionKind::Sinusoidal => ((lon * lat.cos()) / PI, lat / FRAC_PI_2),
        ProjectionKind::Mollweide => {
            let theta = solve_mollweide_theta(lat);
            ((lon * theta.cos()) / PI, theta.sin())
        }
        ProjectionKind::Hammer => {
            let denom = (1.0 + lat.cos() * (lon * 0.5).cos()).sqrt().max(1e-4);
            let x = (2.0 * SQRT_2 * lat.cos() * (lon * 0.5).sin()) / denom;
            let y = (SQRT_2 * lat.sin()) / denom;
            (x / (2.0 * SQRT_2), y / SQRT_2)
        }
        ProjectionKind::Mercator => {
            let lat = lat.clamp((-80.0_f32).to_radians(), 80.0_f32.to_radians());
            let y = (FRAC_PI_4 + lat * 0.5).tan().ln();
            let y_max = (FRAC_PI_4 + 80.0_f32.to_radians() * 0.5).tan().ln();
            (lon / PI, y / y_max)
        }
    };

    vec2(x * MAP_HALF_WIDTH, y * MAP_HALF_HEIGHT)
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

fn ease_in_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) * 0.5
    }
}
