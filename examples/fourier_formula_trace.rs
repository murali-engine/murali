use anyhow::Context;
use glam::{Vec2, Vec3, Vec4, vec2};
use murali::App;
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::DirtyFlags;
use murali::frontend::animation::Ease;
use murali::frontend::layout::{Bounded, Bounds};
use murali::frontend::sangrah::primitives::circle::Circle;
use murali::frontend::sangrah::primitives::path::{Path, PathSegment};
use murali::frontend::sangrah::text::label::Label;
use murali::frontend::sangrah::utility::TracedPath;
use murali::frontend::style::{StrokeParams, Style};
use murali::math::bezier::{cubic_bezier, quadratic_bezier};
use murali::positions::CAMERA_DEFAULT_POS;
use murali::projection::{Project, ProjectionCtx, RenderPrimitive};
use murali::resource::typst_resource::compiler::TypstBackend;
use murali::resource::typst_resource::vector::parse_svg_to_paths;
use std::f32::consts::TAU;

const SAMPLE_COUNT: usize = 760;
const FOURIER_TERMS: usize = 34;
const FORMULA_HEIGHT: f32 = 2.65;
const TRACE_START: f32 = 1.1;
const TRACE_DURATION: f32 = 30.0;
const INITIAL_VIEW_WIDTH: f32 = 11.2;
const CLOSEUP_ZOOM: f32 = 4.0;
const OUTLINE_COLOR: Vec4 = Vec4::new(0.36, 0.92, 0.96, 0.28);
const CIRCLE_COLOR: Vec4 = Vec4::new(0.93, 0.96, 1.0, 0.24);
const SPOKE_COLOR: Vec4 = Vec4::new(0.98, 0.92, 0.72, 0.80);
const TRACE_COLOR: Vec4 = Vec4::new(1.0, 0.69, 0.31, 0.96);

#[derive(Debug, Clone, Copy)]
struct Complex32 {
    re: f32,
    im: f32,
}

impl Complex32 {
    const ZERO: Self = Self { re: 0.0, im: 0.0 };

    fn new(re: f32, im: f32) -> Self {
        Self { re, im }
    }

    fn from_vec2(point: Vec2) -> Self {
        Self::new(point.x, point.y)
    }

    fn to_vec2(self) -> Vec2 {
        vec2(self.re, self.im)
    }

    fn magnitude(self) -> f32 {
        self.re.hypot(self.im)
    }

    fn rotate(self, angle: f32) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self::new(self.re * cos - self.im * sin, self.re * sin + self.im * cos)
    }
}

impl std::ops::Add for Complex32 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.re + rhs.re, self.im + rhs.im)
    }
}

impl std::ops::AddAssign for Complex32 {
    fn add_assign(&mut self, rhs: Self) {
        self.re += rhs.re;
        self.im += rhs.im;
    }
}

impl std::ops::Mul<f32> for Complex32 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::new(self.re * rhs, self.im * rhs)
    }
}

#[derive(Debug, Clone, Copy)]
struct FourierCoeff {
    frequency: i32,
    value: Complex32,
}

#[derive(Debug, Clone)]
struct FourierEpicycles {
    coeffs: Vec<FourierCoeff>,
    outline_bounds: Bounds,
    phase: f32,
    build_progress: f32,
    circle_color: Vec4,
    spoke_color: Vec4,
    circle_thickness: f32,
    spoke_thickness: f32,
}

impl FourierEpicycles {
    fn new(coeffs: Vec<FourierCoeff>, outline_points: &[Vec2]) -> Self {
        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = Vec2::splat(f32::NEG_INFINITY);
        for point in outline_points {
            min = min.min(*point);
            max = max.max(*point);
        }

        Self {
            coeffs,
            outline_bounds: Bounds::new(min, max),
            phase: 0.0,
            build_progress: 0.0,
            circle_color: CIRCLE_COLOR,
            spoke_color: SPOKE_COLOR,
            circle_thickness: 0.015,
            spoke_thickness: 0.023,
        }
    }

    fn total_radius(&self) -> f32 {
        self.coeffs
            .iter()
            .skip(1)
            .map(|coeff| coeff.value.magnitude())
            .sum()
    }

    fn active_coeffs(&self) -> Vec<Complex32> {
        let total_radius = self.total_radius();
        let mut budget = self.build_progress.clamp(0.0, 1.0) * total_radius;
        let mut active = Vec::with_capacity(self.coeffs.len());

        for (index, coeff) in self.coeffs.iter().enumerate() {
            let rotated = coeff
                .value
                .rotate(TAU * coeff.frequency as f32 * self.phase);
            if index == 0 {
                active.push(rotated);
                continue;
            }

            let magnitude = coeff.value.magnitude();
            if budget <= 0.0 {
                break;
            }

            if magnitude <= budget {
                active.push(rotated);
                budget -= magnitude;
            } else {
                active.push(rotated * (budget / magnitude.max(f32::EPSILON)));
                break;
            }
        }

        active
    }

    fn tip(&self) -> Vec2 {
        self.active_coeffs()
            .into_iter()
            .fold(Complex32::ZERO, |sum, coeff| sum + coeff)
            .to_vec2()
    }
}

impl Project for FourierEpicycles {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let mut origin = Vec2::ZERO;
        for coeff in self.active_coeffs() {
            let offset = coeff.to_vec2();
            let radius = coeff.magnitude();
            let next = origin + offset;

            if radius > 0.001 {
                let segments = 56;
                for i in 0..segments {
                    let a0 = TAU * (i as f32 / segments as f32);
                    let a1 = TAU * ((i + 1) as f32 / segments as f32);
                    let p0 = origin + vec2(radius * a0.cos(), radius * a0.sin());
                    let p1 = origin + vec2(radius * a1.cos(), radius * a1.sin());
                    ctx.emit(RenderPrimitive::Line {
                        start: Vec3::new(p0.x, p0.y, 0.0),
                        end: Vec3::new(p1.x, p1.y, 0.0),
                        thickness: self.circle_thickness,
                        color: self.circle_color,
                        dash_length: 0.0,
                        gap_length: 0.0,
                        dash_offset: 0.0,
                    });
                }
            }

            ctx.emit(RenderPrimitive::Line {
                start: Vec3::new(origin.x, origin.y, 0.0),
                end: Vec3::new(next.x, next.y, 0.0),
                thickness: self.spoke_thickness,
                color: self.spoke_color,
                dash_length: 0.0,
                gap_length: 0.0,
                dash_offset: 0.0,
            });

            origin = next;
        }
    }
}

impl Bounded for FourierEpicycles {
    fn local_bounds(&self) -> Bounds {
        let padding = self.total_radius().max(0.5);
        Bounds::new(
            self.outline_bounds.min - Vec2::splat(padding),
            self.outline_bounds.max + Vec2::splat(padding),
        )
    }
}

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    let title_id = scene.add_tattva(
        Label::new("Fourier Transform Trace", 0.3).with_color(WHITE),
        Vec3::new(0.0, 2.86, 0.0),
    );

    let outline_points = formula_outline_points("$pi$", FORMULA_HEIGHT, SAMPLE_COUNT)
        .context("failed to build Fourier outline from Typst formula")?;
    let coeffs = fourier_coefficients(&outline_points, FOURIER_TERMS);
    let mut epicycles = FourierEpicycles::new(coeffs, &outline_points);
    epicycles.build_progress = 1.0;
    let initial_tip = epicycles.tip();

    let outline_id = scene.add_tattva(outline_path(&outline_points), Vec3::ZERO);
    scene.hide(outline_id);

    let epicycle_id = scene.add_tattva(epicycles, Vec3::ZERO);

    let tip_id = scene.add_tattva(
        Circle::new(0.075, 32, TRACE_COLOR).with_stroke(0.02, GOLD_A),
        Vec3::new(initial_tip.x, initial_tip.y, 0.0),
    );

    let trace_id = scene.add_tattva(
        TracedPath::new(tip_id, |pos, _rot| pos, TRACE_COLOR, 0.045)
            .with_min_distance(0.01)
            .with_max_points(SAMPLE_COUNT * 2),
        Vec3::ZERO,
    );

    let mut timeline = Timeline::new();
    timeline
        .animate(title_id)
        .at(0.0)
        .for_duration(0.95)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(tip_id)
        .at(0.7)
        .for_duration(0.45)
        .ease(Ease::OutCubic)
        .appear()
        .spawn();
    timeline.call_during(TRACE_START, TRACE_DURATION, move |scene, t| {
        let eased = ease_in_out_cubic(t);
        if let Some(epicycle) = scene.get_tattva_typed_mut::<FourierEpicycles>(epicycle_id) {
            epicycle.state.phase = eased;
            epicycle.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
            let tip = epicycle.state.tip();
            scene.set_position_2d(tip_id, tip);
        }
    });
    timeline
        .animate(outline_id)
        .at(TRACE_START + TRACE_DURATION * 0.82)
        .for_duration(2.8)
        .ease(Ease::InOutCubic)
        .draw()
        .spawn();
    timeline
        .animate_camera()
        .at(TRACE_START + 8.0)
        .for_duration(3.0)
        .ease(Ease::InOutCubic)
        .zoom_to(CLOSEUP_ZOOM)
        .spawn();
    timeline
        .animate_camera()
        .at(TRACE_START + 17.0)
        .for_duration(3.0)
        .ease(Ease::InOutCubic)
        .zoom_to(1.0 / CLOSEUP_ZOOM)
        .spawn();
    timeline.call_at(TRACE_START + TRACE_DURATION, move |scene| {
        let final_tip =
            if let Some(epicycle) = scene.get_tattva_typed_mut::<FourierEpicycles>(epicycle_id) {
                epicycle.state.phase = 1.0;
                epicycle.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
                Some(epicycle.state.tip())
            } else {
                None
            };
        if let Some(tip) = final_tip {
            scene.set_position_2d(tip_id, tip);
        }
        if let Some(trace) = scene.get_tattva_typed_mut::<TracedPath>(trace_id) {
            trace.state.stop_recording();
            trace.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS);
        }
    });

    scene.play(timeline)?;
    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    scene.camera_mut().set_view_width(INITIAL_VIEW_WIDTH);

    App::new()?.with_scene(scene).run_app()
}

fn formula_outline_points(
    typst_formula: &str,
    world_height: f32,
    target_points: usize,
) -> anyhow::Result<Vec<Vec2>> {
    let backend = TypstBackend::new()?;
    let base_size = 32.0;
    let svg = backend.render_to_svg(typst_formula, base_size)?;
    let symbols = parse_svg_to_paths(&svg, OUTLINE_COLOR)?;

    let mut sampled = Vec::new();
    let point_budget_per_symbol = (target_points / symbols.len().max(1)).max(24);
    let world_scale = world_height / base_size;

    for symbol in symbols {
        let mut symbol_points = sample_path(&symbol.path, point_budget_per_symbol);
        for point in &mut symbol_points {
            *point *= world_scale;
        }
        sampled.extend(symbol_points);
    }

    if sampled.len() < 3 {
        anyhow::bail!("formula sampling produced too few points");
    }

    let center = sampled
        .iter()
        .copied()
        .fold(Vec2::ZERO, |acc, point| acc + point)
        / sampled.len() as f32;
    for point in &mut sampled {
        *point -= center;
    }

    Ok(resample_polyline(&sampled, target_points))
}

fn sample_path(path: &Path, samples_per_segment: usize) -> Vec<Vec2> {
    let mut points = Vec::new();
    let mut start = Vec2::ZERO;
    let mut current = Vec2::ZERO;

    for segment in &path.segments {
        match *segment {
            PathSegment::MoveTo(point) => {
                start = point;
                current = point;
                points.push(point);
            }
            PathSegment::LineTo(point) => {
                points.push(point);
                current = point;
            }
            PathSegment::QuadTo(control, end) => {
                for step in 1..=samples_per_segment {
                    let t = step as f32 / samples_per_segment as f32;
                    points.push(quadratic_bezier(current, control, end, t));
                }
                current = end;
            }
            PathSegment::CubicTo(control1, control2, end) => {
                for step in 1..=samples_per_segment {
                    let t = step as f32 / samples_per_segment as f32;
                    points.push(cubic_bezier(current, control1, control2, end, t));
                }
                current = end;
            }
        }
    }

    if path.closed && current.distance(start) > 0.001 {
        points.push(start);
    }

    points
}

fn resample_polyline(points: &[Vec2], target_count: usize) -> Vec<Vec2> {
    if points.len() <= 1 || target_count <= 1 {
        return points.to_vec();
    }

    let mut lengths = Vec::with_capacity(points.len());
    lengths.push(0.0);
    let mut total_length = 0.0;
    for pair in points.windows(2) {
        total_length += pair[0].distance(pair[1]);
        lengths.push(total_length);
    }

    if total_length <= f32::EPSILON {
        return points.to_vec();
    }

    let mut resampled = Vec::with_capacity(target_count);
    for index in 0..target_count {
        let target = total_length * index as f32 / (target_count - 1) as f32;
        let segment = lengths.partition_point(|length| *length < target);
        let upper = segment.min(points.len() - 1);
        let lower = upper.saturating_sub(1);
        let span = (lengths[upper] - lengths[lower]).max(f32::EPSILON);
        let local_t = (target - lengths[lower]) / span;
        resampled.push(points[lower].lerp(points[upper], local_t));
    }

    resampled
}

fn outline_path(points: &[Vec2]) -> Path {
    let mut path = Path::new()
        .with_style(Style::new().with_stroke(StrokeParams {
            thickness: 0.032,
            color: OUTLINE_COLOR,
            ..Default::default()
        }))
        .move_to(points[0]);

    for point in &points[1..] {
        path = path.line_to(*point);
    }

    path
}

fn fourier_coefficients(points: &[Vec2], harmonics: usize) -> Vec<FourierCoeff> {
    let n = points.len() as f32;
    let samples: Vec<Complex32> = points.iter().copied().map(Complex32::from_vec2).collect();
    let mut coeffs = Vec::with_capacity(harmonics * 2 + 1);

    coeffs.push(FourierCoeff {
        frequency: 0,
        value: dft_coefficient(&samples, 0, n),
    });

    for frequency in 1..=harmonics as i32 {
        coeffs.push(FourierCoeff {
            frequency,
            value: dft_coefficient(&samples, frequency, n),
        });
        coeffs.push(FourierCoeff {
            frequency: -frequency,
            value: dft_coefficient(&samples, -frequency, n),
        });
    }

    coeffs.sort_by(|a, b| {
        b.value
            .magnitude()
            .partial_cmp(&a.value.magnitude())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if let Some(dc_index) = coeffs.iter().position(|coeff| coeff.frequency == 0) {
        coeffs.swap(0, dc_index);
    }

    coeffs
}

fn dft_coefficient(samples: &[Complex32], frequency: i32, sample_count: f32) -> Complex32 {
    let mut sum = Complex32::ZERO;
    for (index, sample) in samples.iter().enumerate() {
        let theta = -TAU * frequency as f32 * index as f32 / sample_count;
        sum += sample.rotate(theta);
    }
    sum * (1.0 / sample_count)
}

fn ease_in_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) * 0.5
    }
}
