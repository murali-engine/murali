use crate::colors;
use crate::engine::app::App;
use crate::engine::camera::Projection;
use crate::engine::export::{ExportSettings, export_scene};
use crate::engine::frame::Frame;
use crate::engine::scene::Scene;
use crate::engine::scene_view::{SceneView, SceneViewPlayback};
use crate::engine::timeline::Timeline;
use crate::frontend::animation::Ease;
use crate::frontend::collection::composite::axes::Axes;
use crate::frontend::collection::composite::axes3d::Axes3D;
use crate::frontend::collection::composite::number_plane::NumberPlane;
use crate::frontend::collection::maths::calculus::parametric_curve3d::ParametricCurve3D;
use crate::frontend::collection::maths::calculus::parametric_surface::{
    ParametricSurface, SurfaceRenderMode,
};
use crate::frontend::collection::primitives::arrow::Arrow;
use crate::frontend::collection::primitives::circle::Circle;
use crate::frontend::collection::primitives::line::Line;
use crate::frontend::collection::primitives::path::Path;
use crate::frontend::collection::primitives::polygon::Polygon;
use crate::frontend::collection::primitives::prop3d::Prop3D;
use crate::frontend::collection::primitives::rectangle::Rectangle;
use crate::frontend::collection::primitives::square::Square;
use crate::frontend::collection::table::table::Table;
use crate::frontend::collection::text::code_block::{CodeBlock, CodeBlockSurface, CodeBlockTheme};
use crate::frontend::collection::text::label::Label;
use crate::frontend::collection::text::latex::Latex;
use crate::frontend::collection::text::typst::Typst;
use crate::frontend::layout::{Anchor, Direction};
use crate::frontend::props::DepthMode;
use glam::{Quat, Vec2, Vec3, Vec4, vec2, vec3, vec4};
use pyo3::exceptions::{PyNotImplementedError, PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyAny;
use std::path::PathBuf;

type ColorTuple = (f32, f32, f32, f32);
type Vec2Tuple = (f32, f32);
type Vec3Tuple = (f32, f32, f32);

#[derive(Clone, Debug)]
enum PyAnimationKind {
    Appear,
    MoveTo(Vec3),
    RotateTo(Quat),
    ScaleTo(Vec3),
    FadeTo(f32),
    Draw,
    Undraw,
    TypewriteText,
    UntypewriteText,
    RevealText,
    HideText,
    Indicate,
    WriteTable,
    UnwriteTable,
    WriteSurface,
    UnwriteSurface,
}

#[derive(Clone, Debug)]
struct PyAnimationSpec {
    target_id: usize,
    start_time: f32,
    duration: f32,
    ease: Ease,
    kind: Option<PyAnimationKind>,
}

fn color_from_tuple(color: ColorTuple) -> PyResult<Vec4> {
    let (r, g, b, a) = color;
    for channel in [r, g, b, a] {
        if !channel.is_finite() {
            return Err(PyValueError::new_err(
                "color channels must be finite numbers",
            ));
        }
    }
    Ok(vec4(r, g, b, a))
}

fn vec3_from_tuple(position: Option<Vec3Tuple>) -> PyResult<Vec3> {
    let Some((x, y, z)) = position else {
        return Ok(Vec3::ZERO);
    };
    for channel in [x, y, z] {
        if !channel.is_finite() {
            return Err(PyValueError::new_err(
                "position values must be finite numbers",
            ));
        }
    }
    Ok(vec3(x, y, z))
}

fn vec2_from_tuple(position: Vec2Tuple) -> PyResult<Vec2> {
    let (x, y) = position;
    if !x.is_finite() || !y.is_finite() {
        return Err(PyValueError::new_err(
            "2D position values must be finite numbers",
        ));
    }
    Ok(vec2(x, y))
}

fn frame_from_name(name: Option<&str>) -> PyResult<Frame> {
    match name.unwrap_or("landscape") {
        "landscape" | "wide" | "16:9" => Ok(Frame::landscape()),
        "portrait" | "vertical" | "9:16" => Ok(Frame::portrait()),
        "square" | "1:1" => Ok(Frame::square()),
        other => Err(PyValueError::new_err(format!(
            "unknown frame {other:?}; expected landscape, portrait, or square"
        ))),
    }
}

fn frame_name(frame: Frame) -> &'static str {
    match frame {
        Frame::Landscape => "landscape",
        Frame::Portrait => "portrait",
        Frame::Square => "square",
    }
}

fn direction_from_name(name: &str) -> PyResult<Direction> {
    match name {
        "up" | "top" => Ok(Direction::Up),
        "down" | "bottom" => Ok(Direction::Down),
        "left" => Ok(Direction::Left),
        "right" => Ok(Direction::Right),
        other => Err(PyValueError::new_err(format!(
            "unknown direction {other:?}; expected up, down, left, or right"
        ))),
    }
}

fn anchor_from_name(name: &str) -> PyResult<Anchor> {
    match name {
        "center" => Ok(Anchor::Center),
        "up" | "top" => Ok(Anchor::Up),
        "down" | "bottom" => Ok(Anchor::Down),
        "left" => Ok(Anchor::Left),
        "right" => Ok(Anchor::Right),
        "up_left" | "top_left" => Ok(Anchor::UpLeft),
        "up_right" | "top_right" => Ok(Anchor::UpRight),
        "down_left" | "bottom_left" => Ok(Anchor::DownLeft),
        "down_right" | "bottom_right" => Ok(Anchor::DownRight),
        other => Err(PyValueError::new_err(format!(
            "unknown anchor {other:?}; expected center, up, down, left, right, up_left, up_right, down_left, or down_right"
        ))),
    }
}

fn code_block_theme_from_name(name: &str) -> PyResult<CodeBlockTheme> {
    match name {
        "dark" => Ok(CodeBlockTheme::Dark),
        "light" => Ok(CodeBlockTheme::Light),
        other => Err(PyValueError::new_err(format!(
            "unknown code block theme {other:?}; expected dark or light"
        ))),
    }
}

fn code_block_surface_from_name(name: &str) -> PyResult<CodeBlockSurface> {
    match name {
        "dark" => Ok(CodeBlockSurface::Dark),
        "light" => Ok(CodeBlockSurface::Light),
        other => Err(PyValueError::new_err(format!(
            "unknown code block surface {other:?}; expected dark or light"
        ))),
    }
}

fn depth_mode_from_name(name: &str) -> PyResult<DepthMode> {
    match name {
        "world" => Ok(DepthMode::World),
        "overlay" => Ok(DepthMode::Overlay),
        other => Err(PyValueError::new_err(format!(
            "unknown depth mode {other:?}; expected world or overlay"
        ))),
    }
}

fn scene_view_playback_from_name(
    name: &str,
    loop_duration: Option<f32>,
) -> PyResult<SceneViewPlayback> {
    match name {
        "continuous" => Ok(SceneViewPlayback::Continuous),
        "once" => Ok(SceneViewPlayback::Once),
        "paused" => Ok(SceneViewPlayback::Paused),
        "loop" => {
            let duration = loop_duration.unwrap_or(1.0);
            if duration <= 0.0 || !duration.is_finite() {
                return Err(PyValueError::new_err(
                    "loop_duration must be a positive finite number",
                ));
            }
            Ok(SceneViewPlayback::Loop { duration })
        }
        other => Err(PyValueError::new_err(format!(
            "unknown SceneView playback {other:?}; expected continuous, once, loop, or paused"
        ))),
    }
}

fn surface_render_mode_from_name(name: &str) -> PyResult<SurfaceRenderMode> {
    match name {
        "solid" => Ok(SurfaceRenderMode::Solid),
        "wireframe" => Ok(SurfaceRenderMode::Wireframe),
        "solid_with_wireframe" | "both" => Ok(SurfaceRenderMode::SolidWithWireframe),
        other => Err(PyValueError::new_err(format!(
            "unknown surface render mode {other:?}; expected solid, wireframe, or solid_with_wireframe"
        ))),
    }
}

fn validate_non_negative_finite(name: &str, value: f32) -> PyResult<()> {
    if value < 0.0 || !value.is_finite() {
        return Err(PyValueError::new_err(format!(
            "{name} must be a non-negative finite number"
        )));
    }
    Ok(())
}

fn color_tuple(color: Vec4) -> ColorTuple {
    (color.x, color.y, color.z, color.w)
}

fn ease_from_name(name: &str) -> PyResult<Ease> {
    match name {
        "linear" => Ok(Ease::Linear),
        "in_quad" => Ok(Ease::InQuad),
        "out_quad" => Ok(Ease::OutQuad),
        "in_out_quad" => Ok(Ease::InOutQuad),
        "in_cubic" => Ok(Ease::InCubic),
        "out_cubic" => Ok(Ease::OutCubic),
        "in_out_cubic" => Ok(Ease::InOutCubic),
        "smooth" | "in_out_smooth" => Ok(Ease::InOutSmooth),
        _ => Err(PyValueError::new_err(format!(
            "unknown ease {name:?}; expected linear, in_quad, out_quad, in_out_quad, in_cubic, out_cubic, in_out_cubic, or smooth"
        ))),
    }
}

#[pyclass(name = "TattvaHandle", module = "murali_engine")]
#[derive(Clone, Copy, Debug)]
struct PyTattvaHandle {
    #[pyo3(get)]
    id: usize,
}

#[pymethods]
impl PyTattvaHandle {
    fn __repr__(&self) -> String {
        format!("TattvaHandle(id={})", self.id)
    }
}

#[pyclass(name = "AnimationBuilder", module = "murali_engine")]
struct PyAnimationBuilder {
    timeline: Py<PyTimeline>,
    spec: PyAnimationSpec,
}

#[pymethods]
impl PyAnimationBuilder {
    fn at<'py>(mut slf: PyRefMut<'py, Self>, start_time: f32) -> PyResult<PyRefMut<'py, Self>> {
        validate_non_negative_finite("start_time", start_time)?;
        slf.spec.start_time = start_time;
        Ok(slf)
    }

    fn for_duration<'py>(
        mut slf: PyRefMut<'py, Self>,
        duration: f32,
    ) -> PyResult<PyRefMut<'py, Self>> {
        validate_non_negative_finite("duration", duration)?;
        slf.spec.duration = duration;
        Ok(slf)
    }

    fn ease<'py>(mut slf: PyRefMut<'py, Self>, name: &str) -> PyResult<PyRefMut<'py, Self>> {
        slf.spec.ease = ease_from_name(name)?;
        Ok(slf)
    }

    fn appear<'py>(mut slf: PyRefMut<'py, Self>) -> PyResult<PyRefMut<'py, Self>> {
        slf.spec.kind = Some(PyAnimationKind::Appear);
        Ok(slf)
    }

    fn move_to<'py>(mut slf: PyRefMut<'py, Self>, to: Vec3Tuple) -> PyResult<PyRefMut<'py, Self>> {
        slf.spec.kind = Some(PyAnimationKind::MoveTo(vec3_from_tuple(Some(to))?));
        Ok(slf)
    }

    fn rotate_to<'py>(
        mut slf: PyRefMut<'py, Self>,
        angle_degrees: f32,
    ) -> PyResult<PyRefMut<'py, Self>> {
        if !angle_degrees.is_finite() {
            return Err(PyValueError::new_err(
                "angle_degrees must be a finite number",
            ));
        }
        slf.spec.kind = Some(PyAnimationKind::RotateTo(Quat::from_rotation_z(
            angle_degrees.to_radians(),
        )));
        Ok(slf)
    }

    fn scale_to<'py>(mut slf: PyRefMut<'py, Self>, to: Vec3Tuple) -> PyResult<PyRefMut<'py, Self>> {
        slf.spec.kind = Some(PyAnimationKind::ScaleTo(vec3_from_tuple(Some(to))?));
        Ok(slf)
    }

    fn fade_to<'py>(mut slf: PyRefMut<'py, Self>, opacity: f32) -> PyResult<PyRefMut<'py, Self>> {
        if !(0.0..=1.0).contains(&opacity) || !opacity.is_finite() {
            return Err(PyValueError::new_err(
                "opacity must be a finite number between 0 and 1",
            ));
        }
        slf.spec.kind = Some(PyAnimationKind::FadeTo(opacity));
        Ok(slf)
    }

    fn draw<'py>(mut slf: PyRefMut<'py, Self>) -> PyResult<PyRefMut<'py, Self>> {
        slf.spec.kind = Some(PyAnimationKind::Draw);
        Ok(slf)
    }

    fn undraw<'py>(mut slf: PyRefMut<'py, Self>) -> PyResult<PyRefMut<'py, Self>> {
        slf.spec.kind = Some(PyAnimationKind::Undraw);
        Ok(slf)
    }

    fn typewrite_text<'py>(mut slf: PyRefMut<'py, Self>) -> PyResult<PyRefMut<'py, Self>> {
        slf.spec.kind = Some(PyAnimationKind::TypewriteText);
        Ok(slf)
    }

    fn untypewrite_text<'py>(mut slf: PyRefMut<'py, Self>) -> PyResult<PyRefMut<'py, Self>> {
        slf.spec.kind = Some(PyAnimationKind::UntypewriteText);
        Ok(slf)
    }

    fn reveal_text<'py>(mut slf: PyRefMut<'py, Self>) -> PyResult<PyRefMut<'py, Self>> {
        slf.spec.kind = Some(PyAnimationKind::RevealText);
        Ok(slf)
    }

    fn hide_text<'py>(mut slf: PyRefMut<'py, Self>) -> PyResult<PyRefMut<'py, Self>> {
        slf.spec.kind = Some(PyAnimationKind::HideText);
        Ok(slf)
    }

    fn indicate<'py>(mut slf: PyRefMut<'py, Self>) -> PyResult<PyRefMut<'py, Self>> {
        slf.spec.kind = Some(PyAnimationKind::Indicate);
        Ok(slf)
    }

    fn write_table<'py>(mut slf: PyRefMut<'py, Self>) -> PyResult<PyRefMut<'py, Self>> {
        slf.spec.kind = Some(PyAnimationKind::WriteTable);
        Ok(slf)
    }

    fn unwrite_table<'py>(mut slf: PyRefMut<'py, Self>) -> PyResult<PyRefMut<'py, Self>> {
        slf.spec.kind = Some(PyAnimationKind::UnwriteTable);
        Ok(slf)
    }

    fn write_surface<'py>(mut slf: PyRefMut<'py, Self>) -> PyResult<PyRefMut<'py, Self>> {
        slf.spec.kind = Some(PyAnimationKind::WriteSurface);
        Ok(slf)
    }

    fn unwrite_surface<'py>(mut slf: PyRefMut<'py, Self>) -> PyResult<PyRefMut<'py, Self>> {
        slf.spec.kind = Some(PyAnimationKind::UnwriteSurface);
        Ok(slf)
    }

    fn spawn(&self, py: Python<'_>) -> PyResult<()> {
        if self.spec.kind.is_none() {
            return Err(PyValueError::new_err(
                "animation kind is missing; call appear, draw, move_to, rotate_to, scale_to, fade_to, typewrite_text, reveal_text, hide_text, indicate, write_table, or write_surface before spawn",
            ));
        }
        let mut timeline = self.timeline.borrow_mut(py);
        timeline.specs.push(self.spec.clone());
        Ok(())
    }
}

#[pyclass(name = "Timeline", module = "murali_engine")]
#[derive(Clone, Debug, Default)]
struct PyTimeline {
    specs: Vec<PyAnimationSpec>,
}

#[pymethods]
impl PyTimeline {
    #[new]
    fn new() -> Self {
        Self { specs: Vec::new() }
    }

    fn animate(slf: PyRefMut<'_, Self>, target: &PyTattvaHandle) -> PyResult<PyAnimationBuilder> {
        let py = slf.py();
        let timeline = slf.into_pyobject(py)?.unbind();
        Ok(PyAnimationBuilder {
            timeline,
            spec: PyAnimationSpec {
                target_id: target.id,
                start_time: 0.0,
                duration: 1.0,
                ease: Ease::Linear,
                kind: None,
            },
        })
    }

    fn len(&self) -> usize {
        self.specs.len()
    }

    fn is_empty(&self) -> bool {
        self.specs.is_empty()
    }
}

#[pyclass(name = "Label", module = "murali_engine")]
#[derive(Clone, Debug)]
struct PyLabel {
    inner: Label,
}

#[pymethods]
impl PyLabel {
    #[new]
    #[pyo3(signature = (text, height = 0.24, color = None))]
    fn new(text: String, height: f32, color: Option<ColorTuple>) -> PyResult<Self> {
        if height <= 0.0 || !height.is_finite() {
            return Err(PyValueError::new_err(
                "height must be a positive finite number",
            ));
        }
        let inner = Label::new(text, height).with_color(match color {
            Some(color) => color_from_tuple(color)?,
            None => colors::WHITE,
        });
        Ok(Self { inner })
    }

    fn with_color(&mut self, color: ColorTuple) -> PyResult<()> {
        self.inner.color = color_from_tuple(color)?;
        Ok(())
    }

    fn with_font(&mut self, font_name: String) {
        self.inner.font_name = Some(font_name);
    }

    fn __repr__(&self) -> String {
        format!(
            "Label(text={:?}, height={})",
            self.inner.text, self.inner.world_height
        )
    }
}

#[pyclass(name = "Circle", module = "murali_engine")]
#[derive(Clone, Debug)]
struct PyCircle {
    inner: Circle,
}

#[pymethods]
impl PyCircle {
    #[new]
    #[pyo3(signature = (radius = 1.0, color = None, segments = 32))]
    fn new(radius: f32, color: Option<ColorTuple>, segments: u32) -> PyResult<Self> {
        if radius <= 0.0 || !radius.is_finite() {
            return Err(PyValueError::new_err(
                "radius must be a positive finite number",
            ));
        }
        if segments < 3 {
            return Err(PyValueError::new_err("segments must be at least 3"));
        }
        Ok(Self {
            inner: Circle::new(
                radius,
                segments,
                color_from_tuple(color.unwrap_or(color_tuple(colors::WHITE)))?,
            ),
        })
    }

    fn with_stroke(&mut self, thickness: f32, color: ColorTuple) -> PyResult<()> {
        if thickness <= 0.0 || !thickness.is_finite() {
            return Err(PyValueError::new_err(
                "stroke thickness must be a positive finite number",
            ));
        }
        self.inner = self
            .inner
            .clone()
            .with_stroke(thickness, color_from_tuple(color)?);
        Ok(())
    }
}

#[pyclass(name = "Square", module = "murali_engine")]
#[derive(Clone, Debug)]
struct PySquare {
    inner: Square,
}

#[pymethods]
impl PySquare {
    #[new]
    #[pyo3(signature = (size = 1.0, color = None))]
    fn new(size: f32, color: Option<ColorTuple>) -> PyResult<Self> {
        if size <= 0.0 || !size.is_finite() {
            return Err(PyValueError::new_err(
                "size must be a positive finite number",
            ));
        }
        Ok(Self {
            inner: Square::new(
                size,
                color_from_tuple(color.unwrap_or(color_tuple(colors::WHITE)))?,
            ),
        })
    }

    fn with_stroke(&mut self, thickness: f32, color: ColorTuple) -> PyResult<()> {
        if thickness <= 0.0 || !thickness.is_finite() {
            return Err(PyValueError::new_err(
                "stroke thickness must be a positive finite number",
            ));
        }
        self.inner = self
            .inner
            .clone()
            .with_stroke(thickness, color_from_tuple(color)?);
        Ok(())
    }
}

#[pyclass(name = "Rectangle", module = "murali_engine")]
#[derive(Clone, Debug)]
struct PyRectangle {
    inner: Rectangle,
}

#[pymethods]
impl PyRectangle {
    #[new]
    #[pyo3(signature = (width = 1.0, height = 1.0, color = None))]
    fn new(width: f32, height: f32, color: Option<ColorTuple>) -> PyResult<Self> {
        if width <= 0.0 || height <= 0.0 || !width.is_finite() || !height.is_finite() {
            return Err(PyValueError::new_err(
                "width and height must be positive finite numbers",
            ));
        }
        Ok(Self {
            inner: Rectangle::new(
                width,
                height,
                color_from_tuple(color.unwrap_or(color_tuple(colors::WHITE)))?,
            ),
        })
    }

    fn with_stroke(&mut self, thickness: f32, color: ColorTuple) -> PyResult<()> {
        if thickness <= 0.0 || !thickness.is_finite() {
            return Err(PyValueError::new_err(
                "stroke thickness must be a positive finite number",
            ));
        }
        self.inner = self
            .inner
            .clone()
            .with_stroke(thickness, color_from_tuple(color)?);
        Ok(())
    }
}

#[pyclass(name = "Polygon", module = "murali_engine")]
#[derive(Clone, Debug)]
struct PyPolygon {
    inner: Polygon,
}

#[pymethods]
impl PyPolygon {
    #[new]
    #[pyo3(signature = (vertices, color = None))]
    fn new(vertices: Vec<Vec2Tuple>, color: Option<ColorTuple>) -> PyResult<Self> {
        if vertices.len() < 3 {
            return Err(PyValueError::new_err(
                "Polygon requires at least three vertices",
            ));
        }
        let vertices = vertices
            .into_iter()
            .map(vec2_from_tuple)
            .collect::<PyResult<Vec<_>>>()?;
        Ok(Self {
            inner: Polygon::new(
                vertices,
                color_from_tuple(color.unwrap_or(color_tuple(colors::WHITE)))?,
            ),
        })
    }

    #[staticmethod]
    #[pyo3(signature = (sides, radius = 1.0, color = None))]
    fn regular(sides: usize, radius: f32, color: Option<ColorTuple>) -> PyResult<Self> {
        if sides < 3 {
            return Err(PyValueError::new_err("sides must be at least 3"));
        }
        if radius <= 0.0 || !radius.is_finite() {
            return Err(PyValueError::new_err(
                "radius must be a positive finite number",
            ));
        }
        Ok(Self {
            inner: Polygon::regular(
                sides,
                radius,
                color_from_tuple(color.unwrap_or(color_tuple(colors::WHITE)))?,
            ),
        })
    }

    fn with_stroke(&mut self, thickness: f32, color: ColorTuple) -> PyResult<()> {
        if thickness <= 0.0 || !thickness.is_finite() {
            return Err(PyValueError::new_err(
                "stroke thickness must be a positive finite number",
            ));
        }
        self.inner = self
            .inner
            .clone()
            .with_stroke(thickness, color_from_tuple(color)?);
        Ok(())
    }
}

#[pyclass(name = "Line", module = "murali_engine")]
#[derive(Clone, Debug)]
struct PyLine {
    inner: Line,
}

#[pymethods]
impl PyLine {
    #[new]
    #[pyo3(signature = (start, end, thickness = 0.02, color = None))]
    fn new(
        start: Vec3Tuple,
        end: Vec3Tuple,
        thickness: f32,
        color: Option<ColorTuple>,
    ) -> PyResult<Self> {
        if thickness <= 0.0 || !thickness.is_finite() {
            return Err(PyValueError::new_err(
                "thickness must be a positive finite number",
            ));
        }
        Ok(Self {
            inner: Line::new(
                vec3_from_tuple(Some(start))?,
                vec3_from_tuple(Some(end))?,
                thickness,
                color_from_tuple(color.unwrap_or(color_tuple(colors::WHITE)))?,
            ),
        })
    }

    fn with_dash(&mut self, dash: f32, gap: f32) -> PyResult<()> {
        if dash < 0.0 || gap < 0.0 || !dash.is_finite() || !gap.is_finite() {
            return Err(PyValueError::new_err(
                "dash and gap must be non-negative finite numbers",
            ));
        }
        self.inner = self.inner.clone().with_dash(dash, gap);
        Ok(())
    }
}

#[pyclass(name = "Arrow", module = "murali_engine")]
#[derive(Clone, Debug)]
struct PyArrow {
    inner: Arrow,
}

#[pymethods]
impl PyArrow {
    #[new]
    #[pyo3(signature = (start, end, shaft_thickness = 0.05, color = None, tip_length = None, tip_width = None))]
    fn new(
        start: Vec2Tuple,
        end: Vec2Tuple,
        shaft_thickness: f32,
        color: Option<ColorTuple>,
        tip_length: Option<f32>,
        tip_width: Option<f32>,
    ) -> PyResult<Self> {
        if shaft_thickness <= 0.0 || !shaft_thickness.is_finite() {
            return Err(PyValueError::new_err(
                "shaft_thickness must be a positive finite number",
            ));
        }
        let tip_length = tip_length.unwrap_or(shaft_thickness * 3.0);
        let tip_width = tip_width.unwrap_or(shaft_thickness * 2.0);
        if tip_length <= 0.0
            || tip_width <= 0.0
            || !tip_length.is_finite()
            || !tip_width.is_finite()
        {
            return Err(PyValueError::new_err(
                "tip_length and tip_width must be positive finite numbers",
            ));
        }
        Ok(Self {
            inner: Arrow::new(
                vec2_from_tuple(start)?,
                vec2_from_tuple(end)?,
                shaft_thickness,
                tip_length,
                tip_width,
                color_from_tuple(color.unwrap_or(color_tuple(colors::WHITE)))?,
            ),
        })
    }
}

#[pyclass(name = "Path", module = "murali_engine")]
#[derive(Clone, Debug)]
struct PyPath {
    inner: Path,
}

#[pymethods]
impl PyPath {
    #[new]
    #[pyo3(signature = (color = None, thickness = 0.02))]
    fn new(color: Option<ColorTuple>, thickness: f32) -> PyResult<Self> {
        if thickness <= 0.0 || !thickness.is_finite() {
            return Err(PyValueError::new_err(
                "thickness must be a positive finite number",
            ));
        }
        Ok(Self {
            inner: Path::new()
                .with_color(color_from_tuple(
                    color.unwrap_or(color_tuple(colors::WHITE)),
                )?)
                .with_thickness(thickness),
        })
    }

    fn move_to(&mut self, point: Vec2Tuple) -> PyResult<()> {
        self.inner = self.inner.clone().move_to(vec2_from_tuple(point)?);
        Ok(())
    }

    fn line_to(&mut self, point: Vec2Tuple) -> PyResult<()> {
        self.inner = self.inner.clone().line_to(vec2_from_tuple(point)?);
        Ok(())
    }

    fn quad_to(&mut self, control: Vec2Tuple, end: Vec2Tuple) -> PyResult<()> {
        self.inner = self
            .inner
            .clone()
            .quad_to(vec2_from_tuple(control)?, vec2_from_tuple(end)?);
        Ok(())
    }

    fn cubic_to(
        &mut self,
        control1: Vec2Tuple,
        control2: Vec2Tuple,
        end: Vec2Tuple,
    ) -> PyResult<()> {
        self.inner = self.inner.clone().cubic_to(
            vec2_from_tuple(control1)?,
            vec2_from_tuple(control2)?,
            vec2_from_tuple(end)?,
        );
        Ok(())
    }

    fn close(&mut self) {
        self.inner = self.inner.clone().close();
    }

    fn with_color(&mut self, color: ColorTuple) -> PyResult<()> {
        self.inner = self.inner.clone().with_color(color_from_tuple(color)?);
        Ok(())
    }

    fn with_thickness(&mut self, thickness: f32) -> PyResult<()> {
        if thickness <= 0.0 || !thickness.is_finite() {
            return Err(PyValueError::new_err(
                "thickness must be a positive finite number",
            ));
        }
        self.inner = self.inner.clone().with_thickness(thickness);
        Ok(())
    }

    fn with_dash(&mut self, dash: f32, gap: f32) -> PyResult<()> {
        if dash < 0.0 || gap < 0.0 || !dash.is_finite() || !gap.is_finite() {
            return Err(PyValueError::new_err(
                "dash and gap must be non-negative finite numbers",
            ));
        }
        self.inner = self.inner.clone().with_dash(dash, gap);
        Ok(())
    }
}

#[pyclass(name = "CodeBlock", module = "murali_engine")]
#[derive(Clone, Debug)]
struct PyCodeBlock {
    inner: CodeBlock,
}

#[pymethods]
impl PyCodeBlock {
    #[new]
    #[pyo3(signature = (code, language = None, font_size = 0.22))]
    fn new(code: String, language: Option<String>, font_size: f32) -> PyResult<Self> {
        if font_size <= 0.0 || !font_size.is_finite() {
            return Err(PyValueError::new_err(
                "font_size must be a positive finite number",
            ));
        }
        Ok(Self {
            inner: CodeBlock::new(
                code,
                language.unwrap_or_else(|| "python".to_string()),
                font_size,
            ),
        })
    }

    fn with_color(&mut self, color: ColorTuple) -> PyResult<()> {
        self.inner = self.inner.clone().with_color(color_from_tuple(color)?);
        Ok(())
    }

    fn with_theme(&mut self, theme: &str) -> PyResult<()> {
        self.inner = self
            .inner
            .clone()
            .with_theme(code_block_theme_from_name(theme)?);
        Ok(())
    }

    fn with_surface(&mut self, surface: &str) -> PyResult<()> {
        self.inner = self
            .inner
            .clone()
            .with_surface(code_block_surface_from_name(surface)?);
        Ok(())
    }

    fn with_title(&mut self, title: String) {
        self.inner = self.inner.clone().with_title(title);
    }

    fn with_controls(&mut self, show: bool) {
        self.inner = self.inner.clone().with_controls(show);
    }

    fn with_line_numbers(&mut self, show: bool) {
        self.inner = self.inner.clone().with_line_numbers(show);
    }

    fn with_content_box_size(&mut self, width: f32, height: f32) -> PyResult<()> {
        if width <= 0.0 || height <= 0.0 || !width.is_finite() || !height.is_finite() {
            return Err(PyValueError::new_err(
                "width and height must be positive finite numbers",
            ));
        }
        self.inner = self.inner.clone().with_content_box_size(width, height);
        Ok(())
    }
}

#[pyclass(name = "Latex", module = "murali_engine")]
#[derive(Clone, Debug)]
struct PyLatex {
    inner: Latex,
}

#[pymethods]
impl PyLatex {
    #[new]
    #[pyo3(signature = (source, height = 0.32, color = None))]
    fn new(source: String, height: f32, color: Option<ColorTuple>) -> PyResult<Self> {
        if height <= 0.0 || !height.is_finite() {
            return Err(PyValueError::new_err(
                "height must be a positive finite number",
            ));
        }
        Ok(Self {
            inner: Latex::new(source, height).with_color(color_from_tuple(
                color.unwrap_or(color_tuple(colors::WHITE)),
            )?),
        })
    }

    fn with_color(&mut self, color: ColorTuple) -> PyResult<()> {
        self.inner = self.inner.clone().with_color(color_from_tuple(color)?);
        Ok(())
    }
}

#[pyclass(name = "Typst", module = "murali_engine")]
#[derive(Clone, Debug)]
struct PyTypst {
    inner: Typst,
}

#[pymethods]
impl PyTypst {
    #[new]
    #[pyo3(signature = (source, height = 0.32, color = None))]
    fn new(source: String, height: f32, color: Option<ColorTuple>) -> PyResult<Self> {
        if height <= 0.0 || !height.is_finite() {
            return Err(PyValueError::new_err(
                "height must be a positive finite number",
            ));
        }
        Ok(Self {
            inner: Typst::new(source, height).with_color(color_from_tuple(
                color.unwrap_or(color_tuple(colors::WHITE)),
            )?),
        })
    }

    fn with_color(&mut self, color: ColorTuple) -> PyResult<()> {
        self.inner = self.inner.clone().with_color(color_from_tuple(color)?);
        Ok(())
    }
}

#[pyclass(name = "Axes", module = "murali_engine")]
#[derive(Clone, Debug)]
struct PyAxes {
    inner: Axes,
}

#[pymethods]
impl PyAxes {
    #[new]
    #[pyo3(signature = (x_range = None, y_range = None, color = None))]
    fn new(
        x_range: Option<(f32, f32)>,
        y_range: Option<(f32, f32)>,
        color: Option<ColorTuple>,
    ) -> PyResult<Self> {
        let x_range = x_range.unwrap_or((-4.0, 4.0));
        let y_range = y_range.unwrap_or((-3.0, 3.0));
        if x_range.0 >= x_range.1 || y_range.0 >= y_range.1 {
            return Err(PyValueError::new_err(
                "range starts must be less than range ends",
            ));
        }
        Ok(Self {
            inner: Axes::new(x_range, y_range).with_color(color_from_tuple(
                color.unwrap_or(color_tuple(colors::WHITE)),
            )?),
        })
    }

    fn with_step(&mut self, step: f32) -> PyResult<()> {
        if step <= 0.0 || !step.is_finite() {
            return Err(PyValueError::new_err(
                "step must be a positive finite number",
            ));
        }
        self.inner = self.inner.clone().with_step(step);
        Ok(())
    }

    fn with_thickness(&mut self, thickness: f32) -> PyResult<()> {
        if thickness <= 0.0 || !thickness.is_finite() {
            return Err(PyValueError::new_err(
                "thickness must be a positive finite number",
            ));
        }
        self.inner = self.inner.clone().with_thickness(thickness);
        Ok(())
    }

    fn with_tick_size(&mut self, tick_size: f32) -> PyResult<()> {
        if tick_size < 0.0 || !tick_size.is_finite() {
            return Err(PyValueError::new_err(
                "tick_size must be a non-negative finite number",
            ));
        }
        self.inner = self.inner.clone().with_tick_size(tick_size);
        Ok(())
    }

    fn without_ticks(&mut self) {
        self.inner = self.inner.clone().without_ticks();
    }
}

#[pyclass(name = "NumberPlane", module = "murali_engine")]
#[derive(Clone, Debug)]
struct PyNumberPlane {
    inner: NumberPlane,
}

#[pymethods]
impl PyNumberPlane {
    #[new]
    #[pyo3(signature = (x_range = None, y_range = None, step = 1.0))]
    fn new(x_range: Option<(f32, f32)>, y_range: Option<(f32, f32)>, step: f32) -> PyResult<Self> {
        let x_range = x_range.unwrap_or((-4.0, 4.0));
        let y_range = y_range.unwrap_or((-3.0, 3.0));
        if x_range.0 >= x_range.1 || y_range.0 >= y_range.1 {
            return Err(PyValueError::new_err(
                "range starts must be less than range ends",
            ));
        }
        if step <= 0.0 || !step.is_finite() {
            return Err(PyValueError::new_err(
                "step must be a positive finite number",
            ));
        }
        Ok(Self {
            inner: NumberPlane::new(x_range, y_range).with_step(step),
        })
    }
}

#[pyclass(name = "Table", module = "murali_engine")]
#[derive(Clone, Debug)]
struct PyTable {
    inner: Table,
}

#[pymethods]
impl PyTable {
    #[new]
    fn new(data: Vec<Vec<String>>) -> PyResult<Self> {
        if data.is_empty() || data.iter().any(Vec::is_empty) {
            return Err(PyValueError::new_err(
                "Table data must contain at least one non-empty row",
            ));
        }
        Ok(Self {
            inner: Table::new(data),
        })
    }

    fn with_row_labels(&mut self, labels: Vec<String>) {
        self.inner = self.inner.clone().with_row_labels(labels);
    }

    fn with_col_labels(&mut self, labels: Vec<String>) {
        self.inner = self.inner.clone().with_col_labels(labels);
    }

    fn with_title(&mut self, title: String) {
        self.inner = self.inner.clone().with_title(title);
    }

    fn with_line_color(&mut self, color: ColorTuple) -> PyResult<()> {
        self.inner = self.inner.clone().with_line_color(color_from_tuple(color)?);
        Ok(())
    }

    fn with_text_color(&mut self, color: ColorTuple) -> PyResult<()> {
        self.inner = self.inner.clone().with_text_color(color_from_tuple(color)?);
        Ok(())
    }

    fn with_text_height(&mut self, height: f32) -> PyResult<()> {
        if height <= 0.0 || !height.is_finite() {
            return Err(PyValueError::new_err(
                "height must be a positive finite number",
            ));
        }
        self.inner = self.inner.clone().with_text_height(height);
        Ok(())
    }

    fn with_background_color(&mut self, color: ColorTuple) -> PyResult<()> {
        self.inner = self
            .inner
            .clone()
            .with_background_color(color_from_tuple(color)?);
        Ok(())
    }

    fn with_labels_inside(&mut self, inside: bool) {
        self.inner = self.inner.clone().with_labels_inside(inside);
    }

    fn num_rows(&self) -> usize {
        self.inner.num_rows()
    }

    fn num_cols(&self) -> usize {
        self.inner.num_cols()
    }
}

#[pyclass(name = "Axes3D", module = "murali_engine")]
#[derive(Clone, Debug)]
struct PyAxes3D {
    inner: Axes3D,
}

#[pymethods]
impl PyAxes3D {
    #[new]
    #[pyo3(signature = (x_range = None, y_range = None, z_range = None))]
    fn new(
        x_range: Option<(f32, f32)>,
        y_range: Option<(f32, f32)>,
        z_range: Option<(f32, f32)>,
    ) -> PyResult<Self> {
        let x_range = x_range.unwrap_or((-2.5, 2.5));
        let y_range = y_range.unwrap_or((-2.0, 2.0));
        let z_range = z_range.unwrap_or((-2.0, 2.0));
        if x_range.0 >= x_range.1 || y_range.0 >= y_range.1 || z_range.0 >= z_range.1 {
            return Err(PyValueError::new_err(
                "range starts must be less than range ends",
            ));
        }
        Ok(Self {
            inner: Axes3D::new(x_range, y_range, z_range),
        })
    }

    fn with_step(&mut self, step: f32) -> PyResult<()> {
        if step <= 0.0 || !step.is_finite() {
            return Err(PyValueError::new_err(
                "step must be a positive finite number",
            ));
        }
        self.inner = self.inner.clone().with_step(step);
        Ok(())
    }

    fn with_axis_thickness(&mut self, thickness: f32) -> PyResult<()> {
        if thickness <= 0.0 || !thickness.is_finite() {
            return Err(PyValueError::new_err(
                "thickness must be a positive finite number",
            ));
        }
        self.inner = self.inner.clone().with_axis_thickness(thickness);
        Ok(())
    }

    fn with_tick_size(&mut self, tick_size: f32) -> PyResult<()> {
        if tick_size < 0.0 || !tick_size.is_finite() {
            return Err(PyValueError::new_err(
                "tick_size must be a non-negative finite number",
            ));
        }
        self.inner = self.inner.clone().with_tick_size(tick_size);
        Ok(())
    }

    fn without_ticks(&mut self) {
        self.inner = self.inner.clone().without_ticks();
    }
}

#[pyclass(name = "ParametricCurve3D", module = "murali_engine")]
#[derive(Clone, Debug)]
struct PyParametricCurve3D {
    inner: ParametricCurve3D,
}

fn helix_curve(t: f32) -> Vec3 {
    vec3(1.5 * t.cos(), 1.5 * t.sin(), -1.5 + 0.22 * t)
}

fn lissajous_curve(t: f32) -> Vec3 {
    vec3(
        1.7 * (0.9 * t).cos(),
        0.85 * (1.4 * t).sin(),
        -1.5 + 0.48 * t + 0.22 * (1.1 * t).cos(),
    )
}

#[pymethods]
impl PyParametricCurve3D {
    #[staticmethod]
    #[pyo3(signature = (kind = "helix", t_range = (0.0, 6.2831855), samples = 160, color = None, thickness = 0.03))]
    fn named(
        kind: &str,
        t_range: (f32, f32),
        samples: usize,
        color: Option<ColorTuple>,
        thickness: f32,
    ) -> PyResult<Self> {
        if t_range.0 >= t_range.1 {
            return Err(PyValueError::new_err(
                "t_range start must be less than t_range end",
            ));
        }
        if thickness <= 0.0 || !thickness.is_finite() {
            return Err(PyValueError::new_err(
                "thickness must be a positive finite number",
            ));
        }
        let f: fn(f32) -> Vec3 = match kind {
            "helix" => helix_curve,
            "lissajous" | "space_curve" => lissajous_curve,
            other => {
                return Err(PyValueError::new_err(format!(
                    "unknown 3D curve {other:?}; expected helix or lissajous"
                )));
            }
        };
        let mut inner = ParametricCurve3D::new(t_range, f).with_samples(samples);
        inner.color = color_from_tuple(color.unwrap_or(color_tuple(colors::GOLD_C)))?;
        inner.thickness = thickness;
        Ok(Self { inner })
    }
}

#[pyclass(name = "ParametricSurface", module = "murali_engine")]
#[derive(Clone)]
struct PyParametricSurface {
    inner: ParametricSurface,
}

fn saddle_surface(u: f32, v: f32) -> Vec3 {
    vec3(u, 0.25 * (u * u - v * v), v)
}

fn wave_surface(u: f32, v: f32) -> Vec3 {
    let r = (u * u + v * v).sqrt();
    let y = if r <= f32::EPSILON {
        0.35
    } else {
        0.35 * (2.8 * r).sin() / r
    };
    vec3(u, y, v)
}

fn hill_surface_py(u: f32, v: f32) -> Vec3 {
    let ridge = 0.95 * (-(0.38 * (u - 0.55).powi(2) + 0.82 * (v + 0.15).powi(2))).exp();
    let shoulder = 0.48 * (-(1.1 * (u + 0.95).powi(2) + 0.46 * (v - 0.45).powi(2))).exp();
    let ripple = 0.14 * (1.7 * u).sin() * (1.25 * v).cos();
    let basin = 0.10 * (0.55 * u * u + 0.9 * v * v);
    vec3(u, ridge + shoulder + ripple - basin - 0.28, v)
}

#[pymethods]
impl PyParametricSurface {
    #[staticmethod]
    #[pyo3(signature = (kind = "saddle", u_range = (-2.0, 2.0), v_range = (-2.0, 2.0), samples = (32, 32), color = None, render_mode = "solid"))]
    fn named(
        kind: &str,
        u_range: (f32, f32),
        v_range: (f32, f32),
        samples: (usize, usize),
        color: Option<ColorTuple>,
        render_mode: &str,
    ) -> PyResult<Self> {
        if u_range.0 >= u_range.1 || v_range.0 >= v_range.1 {
            return Err(PyValueError::new_err(
                "range starts must be less than range ends",
            ));
        }
        let f: fn(f32, f32) -> Vec3 = match kind {
            "saddle" => saddle_surface,
            "wave" => wave_surface,
            "hill" => hill_surface_py,
            other => {
                return Err(PyValueError::new_err(format!(
                    "unknown surface {other:?}; expected saddle, wave, or hill"
                )));
            }
        };
        Ok(Self {
            inner: ParametricSurface::new(u_range, v_range, f)
                .with_samples(samples.0, samples.1)
                .with_color(color_from_tuple(
                    color.unwrap_or(color_tuple(colors::TEAL_C)),
                )?)
                .with_render_mode(surface_render_mode_from_name(render_mode)?),
        })
    }

    fn with_write_progress(&mut self, progress: f32) -> PyResult<()> {
        if !(0.0..=1.0).contains(&progress) || !progress.is_finite() {
            return Err(PyValueError::new_err(
                "progress must be a finite number between 0 and 1",
            ));
        }
        self.inner = self.inner.clone().with_write_progress(progress);
        Ok(())
    }
}

#[pyclass(name = "Prop3D", module = "murali_engine")]
#[derive(Clone, Debug)]
struct PyProp3D {
    inner: Prop3D,
}

#[pymethods]
impl PyProp3D {
    #[staticmethod]
    fn from_file(path: String) -> PyResult<Self> {
        let inner =
            Prop3D::from_file(path).map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
        Ok(Self { inner })
    }

    fn mesh_count(&self) -> usize {
        self.inner.mesh_count()
    }

    fn bounds_min(&self) -> Vec3Tuple {
        let p = self.inner.bounds_min();
        (p.x, p.y, p.z)
    }

    fn bounds_max(&self) -> Vec3Tuple {
        let p = self.inner.bounds_max();
        (p.x, p.y, p.z)
    }
}

#[pyclass(name = "SceneView", module = "murali_engine")]
struct PySceneView {
    inner: Option<SceneView>,
}

#[pymethods]
impl PySceneView {
    #[new]
    fn new(scene: &mut PyScene) -> PyResult<Self> {
        Ok(Self {
            inner: Some(SceneView::new(scene.take_scene()?)),
        })
    }

    fn size(&mut self, width: f32, height: f32) -> PyResult<()> {
        if width <= 0.0 || height <= 0.0 || !width.is_finite() || !height.is_finite() {
            return Err(PyValueError::new_err(
                "width and height must be positive finite numbers",
            ));
        }
        self.inner = Some(self.take_view()?.size(vec2(width, height)));
        Ok(())
    }

    fn background(&mut self, color: ColorTuple) -> PyResult<()> {
        self.inner = Some(self.take_view()?.background(color_from_tuple(color)?));
        Ok(())
    }

    fn transparent_background(&mut self) -> PyResult<()> {
        self.inner = Some(self.take_view()?.transparent_background());
        Ok(())
    }

    fn corner_radius(&mut self, radius: f32) -> PyResult<()> {
        if radius < 0.0 || !radius.is_finite() {
            return Err(PyValueError::new_err(
                "radius must be a non-negative finite number",
            ));
        }
        self.inner = Some(self.take_view()?.corner_radius(radius));
        Ok(())
    }

    fn border(&mut self, width: f32, color: ColorTuple) -> PyResult<()> {
        if width < 0.0 || !width.is_finite() {
            return Err(PyValueError::new_err(
                "width must be a non-negative finite number",
            ));
        }
        self.inner = Some(self.take_view()?.border(width, color_from_tuple(color)?));
        Ok(())
    }

    #[pyo3(signature = (playback, loop_duration = None))]
    fn playback(&mut self, playback: &str, loop_duration: Option<f32>) -> PyResult<()> {
        self.inner = Some(
            self.take_view()?
                .playback(scene_view_playback_from_name(playback, loop_duration)?),
        );
        Ok(())
    }

    fn start_at(&mut self, parent_time: f32) -> PyResult<()> {
        validate_non_negative_finite("parent_time", parent_time)?;
        self.inner = Some(self.take_view()?.start_at(parent_time));
        Ok(())
    }

    fn resolution(&mut self, width: u32, height: u32) -> PyResult<()> {
        if width == 0 || height == 0 {
            return Err(PyValueError::new_err(
                "width and height must be greater than zero",
            ));
        }
        self.inner = Some(self.take_view()?.resolution(width, height));
        Ok(())
    }
}

impl PySceneView {
    fn take_view(&mut self) -> PyResult<SceneView> {
        self.inner
            .take()
            .ok_or_else(|| PyRuntimeError::new_err("this SceneView has already been consumed"))
    }
}

#[pyclass(name = "Scene", module = "murali_engine")]
struct PyScene {
    inner: Option<Scene>,
}

#[pymethods]
impl PyScene {
    #[new]
    #[pyo3(signature = (frame = None))]
    fn new(frame: Option<&str>) -> PyResult<Self> {
        Ok(Self {
            inner: Some(Scene::new().with_frame(frame_from_name(frame)?)),
        })
    }

    #[pyo3(signature = (tattva, at = None))]
    fn add(
        &mut self,
        tattva: &Bound<'_, PyAny>,
        at: Option<Vec3Tuple>,
    ) -> PyResult<PyTattvaHandle> {
        let position = vec3_from_tuple(at)?;
        let scene = self.scene_mut()?;

        if let Ok(label) = tattva.extract::<PyRef<'_, PyLabel>>() {
            return Ok(PyTattvaHandle {
                id: scene.add_tattva(label.inner.clone(), position),
            });
        }
        if let Ok(circle) = tattva.extract::<PyRef<'_, PyCircle>>() {
            return Ok(PyTattvaHandle {
                id: scene.add_tattva(circle.inner.clone(), position),
            });
        }
        if let Ok(square) = tattva.extract::<PyRef<'_, PySquare>>() {
            return Ok(PyTattvaHandle {
                id: scene.add_tattva(square.inner.clone(), position),
            });
        }
        if let Ok(rectangle) = tattva.extract::<PyRef<'_, PyRectangle>>() {
            return Ok(PyTattvaHandle {
                id: scene.add_tattva(rectangle.inner.clone(), position),
            });
        }
        if let Ok(polygon) = tattva.extract::<PyRef<'_, PyPolygon>>() {
            return Ok(PyTattvaHandle {
                id: scene.add_tattva(polygon.inner.clone(), position),
            });
        }
        if let Ok(line) = tattva.extract::<PyRef<'_, PyLine>>() {
            return Ok(PyTattvaHandle {
                id: scene.add_tattva(line.inner.clone(), position),
            });
        }
        if let Ok(arrow) = tattva.extract::<PyRef<'_, PyArrow>>() {
            return Ok(PyTattvaHandle {
                id: scene.add_tattva(arrow.inner.clone(), position),
            });
        }
        if let Ok(path) = tattva.extract::<PyRef<'_, PyPath>>() {
            return Ok(PyTattvaHandle {
                id: scene.add_tattva(path.inner.clone(), position),
            });
        }
        if let Ok(code_block) = tattva.extract::<PyRef<'_, PyCodeBlock>>() {
            return Ok(PyTattvaHandle {
                id: scene.add_tattva(code_block.inner.clone(), position),
            });
        }
        if let Ok(latex) = tattva.extract::<PyRef<'_, PyLatex>>() {
            return Ok(PyTattvaHandle {
                id: scene.add_tattva(latex.inner.clone(), position),
            });
        }
        if let Ok(typst) = tattva.extract::<PyRef<'_, PyTypst>>() {
            return Ok(PyTattvaHandle {
                id: scene.add_tattva(typst.inner.clone(), position),
            });
        }
        if let Ok(axes) = tattva.extract::<PyRef<'_, PyAxes>>() {
            return Ok(PyTattvaHandle {
                id: scene.add_tattva(axes.inner.clone(), position),
            });
        }
        if let Ok(number_plane) = tattva.extract::<PyRef<'_, PyNumberPlane>>() {
            return Ok(PyTattvaHandle {
                id: scene.add_tattva(number_plane.inner.clone(), position),
            });
        }
        if let Ok(table) = tattva.extract::<PyRef<'_, PyTable>>() {
            return Ok(PyTattvaHandle {
                id: scene.add_tattva(table.inner.clone(), position),
            });
        }
        if let Ok(axes3d) = tattva.extract::<PyRef<'_, PyAxes3D>>() {
            return Ok(PyTattvaHandle {
                id: scene.add_tattva(axes3d.inner.clone(), position),
            });
        }
        if let Ok(curve3d) = tattva.extract::<PyRef<'_, PyParametricCurve3D>>() {
            return Ok(PyTattvaHandle {
                id: scene.add_tattva(curve3d.inner, position),
            });
        }
        if let Ok(surface) = tattva.extract::<PyRef<'_, PyParametricSurface>>() {
            return Ok(PyTattvaHandle {
                id: scene.add_tattva(surface.inner.clone(), position),
            });
        }
        if let Ok(prop) = tattva.extract::<PyRef<'_, PyProp3D>>() {
            return Ok(PyTattvaHandle {
                id: scene.add_tattva(prop.inner.clone(), position),
            });
        }

        Err(PyValueError::new_err(
            "Scene.add expected a supported murali_engine drawable",
        ))
    }

    #[pyo3(signature = (view, at = None))]
    fn add_scene_view(
        &mut self,
        view: &mut PySceneView,
        at: Option<Vec3Tuple>,
    ) -> PyResult<PyTattvaHandle> {
        let position = vec3_from_tuple(at)?;
        let view = view.take_view()?;
        Ok(PyTattvaHandle {
            id: self.scene_mut()?.add_scene_view(view, position),
        })
    }

    fn set_frame(&mut self, frame: &str) -> PyResult<()> {
        let frame = frame_from_name(Some(frame))?;
        self.scene_mut()?.set_frame(frame);
        Ok(())
    }

    fn frame(&self) -> PyResult<&'static str> {
        Ok(frame_name(self.scene()?.frame()))
    }

    fn frame_size(&self) -> PyResult<(f32, f32)> {
        Ok(self.scene()?.frame().logical_size())
    }

    fn to_edge(&mut self, handle: &PyTattvaHandle, direction: &str, margin: f32) -> PyResult<()> {
        if margin < 0.0 || !margin.is_finite() {
            return Err(PyValueError::new_err(
                "margin must be a non-negative finite number",
            ));
        }
        let direction = direction_from_name(direction)?;
        self.scene_mut()?.to_edge(handle.id, direction, margin);
        Ok(())
    }

    fn next_to(
        &mut self,
        moving: &PyTattvaHandle,
        target: &PyTattvaHandle,
        direction: &str,
        padding: f32,
    ) -> PyResult<()> {
        if padding < 0.0 || !padding.is_finite() {
            return Err(PyValueError::new_err(
                "padding must be a non-negative finite number",
            ));
        }
        let direction = direction_from_name(direction)?;
        self.scene_mut()?
            .next_to(moving.id, target.id, direction, padding);
        Ok(())
    }

    fn align_to(
        &mut self,
        moving: &PyTattvaHandle,
        target: &PyTattvaHandle,
        anchor: &str,
    ) -> PyResult<()> {
        let anchor = anchor_from_name(anchor)?;
        self.scene_mut()?.align_to(moving.id, target.id, anchor);
        Ok(())
    }

    fn set_position(&mut self, handle: &PyTattvaHandle, position: Vec3Tuple) -> PyResult<()> {
        let position = vec3_from_tuple(Some(position))?;
        self.scene_mut()?.set_position_3d(handle.id, position);
        Ok(())
    }

    fn set_scale(&mut self, handle: &PyTattvaHandle, scale: Vec3Tuple) -> PyResult<()> {
        let scale = vec3_from_tuple(Some(scale))?;
        self.scene_mut()?.set_scale(handle.id, scale);
        Ok(())
    }

    fn set_rotation_z(&mut self, handle: &PyTattvaHandle, angle_degrees: f32) -> PyResult<()> {
        if !angle_degrees.is_finite() {
            return Err(PyValueError::new_err(
                "angle_degrees must be a finite number",
            ));
        }
        self.scene_mut()?
            .set_rotation(handle.id, Quat::from_rotation_z(angle_degrees.to_radians()));
        Ok(())
    }

    fn set_opacity(&mut self, handle: &PyTattvaHandle, opacity: f32) -> PyResult<()> {
        if !(0.0..=1.0).contains(&opacity) || !opacity.is_finite() {
            return Err(PyValueError::new_err(
                "opacity must be a finite number between 0 and 1",
            ));
        }
        self.scene_mut()?.set_opacity(handle.id, opacity);
        Ok(())
    }

    fn set_layer(&mut self, handle: &PyTattvaHandle, layer: i32) -> PyResult<()> {
        self.scene_mut()?.set_layer(handle.id, layer);
        Ok(())
    }

    fn set_depth_mode(&mut self, handle: &PyTattvaHandle, depth_mode: &str) -> PyResult<()> {
        let depth_mode = depth_mode_from_name(depth_mode)?;
        self.scene_mut()?.set_depth_mode(handle.id, depth_mode);
        Ok(())
    }

    #[pyo3(signature = (position, target, up = (0.0, 1.0, 0.0)))]
    fn set_camera(
        &mut self,
        position: Vec3Tuple,
        target: Vec3Tuple,
        up: Vec3Tuple,
    ) -> PyResult<()> {
        let camera = self.scene_mut()?.camera_mut();
        camera.position = vec3_from_tuple(Some(position))?;
        camera.target = vec3_from_tuple(Some(target))?;
        camera.up = vec3_from_tuple(Some(up))?;
        Ok(())
    }

    #[pyo3(signature = (fov_y_degrees = 45.0, near = 0.1, far = 100.0))]
    fn set_perspective_camera(&mut self, fov_y_degrees: f32, near: f32, far: f32) -> PyResult<()> {
        if fov_y_degrees <= 0.0 || near <= 0.0 || far <= near {
            return Err(PyValueError::new_err(
                "expected fov_y_degrees > 0, near > 0, and far > near",
            ));
        }
        let aspect = self.scene()?.frame().aspect_ratio();
        self.scene_mut()?.camera_mut().projection = Projection::Perspective {
            fov_y_rad: fov_y_degrees.to_radians(),
            aspect,
            near,
            far,
        };
        Ok(())
    }

    fn tattva_count(&self) -> usize {
        self.inner
            .as_ref()
            .map(|scene| scene.tattvas.len())
            .unwrap_or(0)
    }

    fn play(&mut self, timeline: &PyTimeline) -> PyResult<()> {
        let scene = self.scene_mut()?;
        let rust_timeline = timeline.to_rust_timeline()?;
        scene
            .play(rust_timeline)
            .map_err(|error| PyValueError::new_err(error.to_string()))
    }

    fn preview(&mut self) -> PyResult<()> {
        let scene = self.take_scene()?;
        App::new()
            .map_err(|error| PyRuntimeError::new_err(error.to_string()))?
            .with_scene(scene)
            .with_preview()
            .run_app()
            .map_err(|error| PyRuntimeError::new_err(error.to_string()))
    }

    #[pyo3(signature = (path, width = None, fps = None, duration = None))]
    fn save_png(
        &mut self,
        path: String,
        width: Option<u32>,
        fps: Option<u32>,
        duration: Option<f32>,
    ) -> PyResult<()> {
        let mut output_path = PathBuf::from(path);
        if !output_path.is_absolute() {
            output_path = std::env::current_dir()
                .map_err(|error| PyRuntimeError::new_err(error.to_string()))?
                .join(output_path);
        }

        let mut scene = self.take_scene()?;
        scene.capture_screenshots_named([(0.0, Some(output_path))]);

        let mut settings = ExportSettings::from_scene(&scene);
        settings.video_enabled = false;
        settings.preserve_frame_exports = false;
        if let Some(width) = width {
            if width == 0 {
                return Err(PyValueError::new_err("width must be greater than zero"));
            }
            settings.width = width;
        }
        if let Some(fps) = fps {
            if fps == 0 {
                return Err(PyValueError::new_err("fps must be greater than zero"));
            }
            settings.fps = fps;
        }
        if let Some(duration) = duration {
            if duration < 0.0 || !duration.is_finite() {
                return Err(PyValueError::new_err(
                    "duration must be a non-negative finite number",
                ));
            }
            settings.duration_seconds = duration;
        }

        export_scene(scene, &settings).map_err(|error| PyRuntimeError::new_err(error.to_string()))
    }

    fn export(&self, _path: Option<String>) -> PyResult<()> {
        Err(PyNotImplementedError::new_err(
            "Scene.export is not exposed in Python yet",
        ))
    }
}

impl PyTimeline {
    fn to_rust_timeline(&self) -> PyResult<Timeline> {
        let mut timeline = Timeline::new();
        for spec in &self.specs {
            let builder = timeline
                .animate(spec.target_id)
                .at(spec.start_time)
                .for_duration(spec.duration)
                .ease(spec.ease);
            match spec.kind.as_ref() {
                Some(PyAnimationKind::Appear) => builder.appear().spawn(),
                Some(PyAnimationKind::MoveTo(to)) => builder.move_to(*to).spawn(),
                Some(PyAnimationKind::RotateTo(to)) => builder.rotate_to(*to).spawn(),
                Some(PyAnimationKind::ScaleTo(to)) => builder.scale_to(*to).spawn(),
                Some(PyAnimationKind::FadeTo(opacity)) => builder.fade_to(*opacity).spawn(),
                Some(PyAnimationKind::Draw) => builder.draw().spawn(),
                Some(PyAnimationKind::Undraw) => builder.undraw().spawn(),
                Some(PyAnimationKind::TypewriteText) => builder.typewrite_text().spawn(),
                Some(PyAnimationKind::UntypewriteText) => builder.untypewrite_text().spawn(),
                Some(PyAnimationKind::RevealText) => builder.reveal_text().spawn(),
                Some(PyAnimationKind::HideText) => builder.hide_text().spawn(),
                Some(PyAnimationKind::Indicate) => builder.indicate().spawn(),
                Some(PyAnimationKind::WriteTable) => builder.write_table().spawn(),
                Some(PyAnimationKind::UnwriteTable) => builder.unwrite_table().spawn(),
                Some(PyAnimationKind::WriteSurface) => builder.write_surface().spawn(),
                Some(PyAnimationKind::UnwriteSurface) => builder.unwrite_surface().spawn(),
                None => {
                    return Err(PyValueError::new_err(
                        "animation kind is missing; call appear, draw, move_to, rotate_to, scale_to, fade_to, typewrite_text, reveal_text, hide_text, indicate, write_table, or write_surface before spawn",
                    ));
                }
            }
        }
        Ok(timeline)
    }
}

impl PyScene {
    fn scene(&self) -> PyResult<&Scene> {
        self.inner.as_ref().ok_or_else(|| {
            PyRuntimeError::new_err("this Scene has already been consumed by preview or save_png")
        })
    }

    fn scene_mut(&mut self) -> PyResult<&mut Scene> {
        self.inner.as_mut().ok_or_else(|| {
            PyRuntimeError::new_err("this Scene has already been consumed by preview or save_png")
        })
    }

    fn take_scene(&mut self) -> PyResult<Scene> {
        self.inner.take().ok_or_else(|| {
            PyRuntimeError::new_err("this Scene has already been consumed by preview or save_png")
        })
    }
}

#[pymodule]
fn murali_engine(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyScene>()?;
    m.add_class::<PyTattvaHandle>()?;
    m.add_class::<PyTimeline>()?;
    m.add_class::<PyAnimationBuilder>()?;
    m.add_class::<PyLabel>()?;
    m.add_class::<PyCircle>()?;
    m.add_class::<PySquare>()?;
    m.add_class::<PyRectangle>()?;
    m.add_class::<PyPolygon>()?;
    m.add_class::<PyLine>()?;
    m.add_class::<PyArrow>()?;
    m.add_class::<PyPath>()?;
    m.add_class::<PyCodeBlock>()?;
    m.add_class::<PyLatex>()?;
    m.add_class::<PyTypst>()?;
    m.add_class::<PyAxes>()?;
    m.add_class::<PyNumberPlane>()?;
    m.add_class::<PyTable>()?;
    m.add_class::<PyAxes3D>()?;
    m.add_class::<PyParametricCurve3D>()?;
    m.add_class::<PyParametricSurface>()?;
    m.add_class::<PyProp3D>()?;
    m.add_class::<PySceneView>()?;

    m.add("WHITE", color_tuple(colors::WHITE))?;
    m.add("BLACK", color_tuple(colors::BLACK))?;
    m.add("GRAY_A", color_tuple(colors::GRAY_A))?;
    m.add("GRAY_B", color_tuple(colors::GRAY_B))?;
    m.add("GRAY_C", color_tuple(colors::GRAY_C))?;
    m.add("GRAY_D", color_tuple(colors::GRAY_D))?;
    m.add("GRAY_E", color_tuple(colors::GRAY_E))?;
    m.add("GRAY", color_tuple(colors::GRAY))?;
    m.add("GREY", color_tuple(colors::GREY))?;
    m.add("BLUE_A", color_tuple(colors::BLUE_A))?;
    m.add("BLUE_B", color_tuple(colors::BLUE_B))?;
    m.add("BLUE_C", color_tuple(colors::BLUE_C))?;
    m.add("BLUE_D", color_tuple(colors::BLUE_D))?;
    m.add("BLUE_E", color_tuple(colors::BLUE_E))?;
    m.add("BLUE", color_tuple(colors::BLUE))?;
    m.add("TEAL_A", color_tuple(colors::TEAL_A))?;
    m.add("TEAL_B", color_tuple(colors::TEAL_B))?;
    m.add("TEAL_C", color_tuple(colors::TEAL_C))?;
    m.add("TEAL_D", color_tuple(colors::TEAL_D))?;
    m.add("TEAL_E", color_tuple(colors::TEAL_E))?;
    m.add("TEAL", color_tuple(colors::TEAL))?;
    m.add("GREEN_A", color_tuple(colors::GREEN_A))?;
    m.add("GREEN_B", color_tuple(colors::GREEN_B))?;
    m.add("GREEN_C", color_tuple(colors::GREEN_C))?;
    m.add("GREEN_D", color_tuple(colors::GREEN_D))?;
    m.add("GREEN_E", color_tuple(colors::GREEN_E))?;
    m.add("GREEN", color_tuple(colors::GREEN))?;
    m.add("YELLOW_A", color_tuple(colors::YELLOW_A))?;
    m.add("YELLOW_B", color_tuple(colors::YELLOW_B))?;
    m.add("YELLOW_C", color_tuple(colors::YELLOW_C))?;
    m.add("YELLOW_D", color_tuple(colors::YELLOW_D))?;
    m.add("YELLOW_E", color_tuple(colors::YELLOW_E))?;
    m.add("YELLOW", color_tuple(colors::YELLOW))?;
    m.add("GOLD_A", color_tuple(colors::GOLD_A))?;
    m.add("GOLD_B", color_tuple(colors::GOLD_B))?;
    m.add("GOLD_C", color_tuple(colors::GOLD_C))?;
    m.add("GOLD_D", color_tuple(colors::GOLD_D))?;
    m.add("GOLD_E", color_tuple(colors::GOLD_E))?;
    m.add("GOLD", color_tuple(colors::GOLD))?;
    m.add("ORANGE_A", color_tuple(colors::ORANGE_A))?;
    m.add("ORANGE_B", color_tuple(colors::ORANGE_B))?;
    m.add("ORANGE_C", color_tuple(colors::ORANGE_C))?;
    m.add("ORANGE_D", color_tuple(colors::ORANGE_D))?;
    m.add("ORANGE_E", color_tuple(colors::ORANGE_E))?;
    m.add("ORANGE", color_tuple(colors::ORANGE))?;
    m.add("RED_A", color_tuple(colors::RED_A))?;
    m.add("RED_B", color_tuple(colors::RED_B))?;
    m.add("RED_C", color_tuple(colors::RED_C))?;
    m.add("RED_D", color_tuple(colors::RED_D))?;
    m.add("RED_E", color_tuple(colors::RED_E))?;
    m.add("RED", color_tuple(colors::RED))?;
    m.add("MAROON_A", color_tuple(colors::MAROON_A))?;
    m.add("MAROON_B", color_tuple(colors::MAROON_B))?;
    m.add("MAROON_C", color_tuple(colors::MAROON_C))?;
    m.add("MAROON_D", color_tuple(colors::MAROON_D))?;
    m.add("MAROON_E", color_tuple(colors::MAROON_E))?;
    m.add("MAROON", color_tuple(colors::MAROON))?;
    m.add("PURPLE_A", color_tuple(colors::PURPLE_A))?;
    m.add("PURPLE_B", color_tuple(colors::PURPLE_B))?;
    m.add("PURPLE_C", color_tuple(colors::PURPLE_C))?;
    m.add("PURPLE_D", color_tuple(colors::PURPLE_D))?;
    m.add("PURPLE_E", color_tuple(colors::PURPLE_E))?;
    m.add("PURPLE", color_tuple(colors::PURPLE))?;
    m.add("PINK_A", color_tuple(colors::PINK_A))?;
    m.add("PINK_B", color_tuple(colors::PINK_B))?;
    m.add("PINK_C", color_tuple(colors::PINK_C))?;
    m.add("PINK_D", color_tuple(colors::PINK_D))?;
    m.add("PINK_E", color_tuple(colors::PINK_E))?;
    m.add("PINK", color_tuple(colors::PINK))?;
    m.add("PURE_RED", color_tuple(colors::PURE_RED))?;
    m.add("PURE_GREEN", color_tuple(colors::PURE_GREEN))?;
    m.add("PURE_BLUE", color_tuple(colors::PURE_BLUE))?;

    Ok(())
}
