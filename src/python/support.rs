use crate::colors;
use crate::engine::app::App;
use crate::engine::camera::Projection;
use crate::engine::export::{ExportSettings, export_scene};
use crate::engine::frame::Frame;
use crate::engine::scene::Scene;
use crate::engine::scene_view::{SceneView, SceneViewPlayback};
use crate::engine::timeline::{SignalPlayback, Timeline};
use crate::frontend::animation::Ease;
use crate::frontend::collection::ai::{
    ContextBlock, ContextBlockRole, ContextTruncation, ContextWindow, SignalFlow,
};
use crate::frontend::collection::composite::axes::Axes;
use crate::frontend::collection::composite::axes3d::Axes3D;
use crate::frontend::collection::composite::number_plane::NumberPlane;
use crate::frontend::collection::maths::basic_math::NumberLine;
use crate::frontend::collection::maths::calculus::parametric_curve3d::ParametricCurve3D;
use crate::frontend::collection::maths::calculus::parametric_surface::{
    ParametricSurface, SurfaceRenderMode,
};
use crate::frontend::collection::maths::notation::equation::{EquationLayout, EquationPart};
use crate::frontend::collection::maths::notation::matrix::Matrix;
use crate::frontend::collection::maths::optimization::OptimizationPath2D;
use crate::frontend::collection::primitives::arrow::Arrow;
use crate::frontend::collection::primitives::chat_bubble::{ChatBubble, ChatBubbleTipSide};
use crate::frontend::collection::primitives::circle::Circle;
use crate::frontend::collection::primitives::line::Line;
use crate::frontend::collection::primitives::particle_belt::ParticleBelt;
use crate::frontend::collection::primitives::path::Path;
use crate::frontend::collection::primitives::polygon::Polygon;
use crate::frontend::collection::primitives::prop3d::Prop3D;
use crate::frontend::collection::primitives::rectangle::Rectangle;
use crate::frontend::collection::primitives::rounded_rectangle::RoundedRectangle;
use crate::frontend::collection::primitives::square::Square;
use crate::frontend::collection::table::table::Table;
use crate::frontend::collection::text::code_block::{CodeBlock, CodeBlockSurface, CodeBlockTheme};
use crate::frontend::collection::text::label::Label;
use crate::frontend::collection::text::latex::Latex;
use crate::frontend::collection::text::letter3d::{Letter3D, LetterParticles3D};
use crate::frontend::collection::text::typst::Typst;
use crate::frontend::collection::utility::TracedPath;
use crate::frontend::DirtyFlags;
use crate::frontend::layout::{Anchor, Direction};
use crate::frontend::props::DepthMode;
use crate::resource::texture::{BuiltinTexture, TextureImage};
use crate::resource::latex_resource::latex_vector_paths;
use crate::resource::typst_resource::{typst_outline_points, typst_vector_paths};
use crate::utils::project::find_murali_project_root;
use glam::{EulerRot, Quat, Vec2, Vec3, Vec4, vec2, vec3, vec4};
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyAny;
use std::path::PathBuf;
use std::sync::Arc;

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
    EquationContinuityFrom(usize),
    MatrixStepCells {
        cells: Vec<(usize, usize)>,
        highlight: Vec4,
        dim_opacity: f32,
    },
    MatrixStepRow {
        row: usize,
        highlight: Vec4,
        dim_opacity: f32,
    },
    MatrixStepColumn {
        col: usize,
        highlight: Vec4,
        dim_opacity: f32,
    },
    WriteTable,
    UnwriteTable,
    WriteSurface,
    UnwriteSurface,
    LetterParticleScatterTo(f32),
    MorphFrom(usize),
    BeltEvolve { speed: Option<f32> },
}

#[derive(Clone, Debug)]
struct PyAnimationSpec {
    target_id: usize,
    start_time: f32,
    duration: f32,
    ease: Ease,
    kind: Option<PyAnimationKind>,
}

#[derive(Clone, Copy, Debug)]
enum PySignalPlaybackMode {
    Once,
    RoundTrip,
    Loop,
}

#[derive(Clone, Debug)]
struct PySignalPlaybackSpec {
    target_id: usize,
    start_time: f32,
    duration: f32,
    ease: Ease,
    mode: PySignalPlaybackMode,
    repeats: usize,
}

#[derive(Clone, Debug)]
enum PyCameraAnimationKind {
    FrameTo { position: Vec3, target: Vec3 },
    ZoomTo { zoom: f32 },
}

#[derive(Clone, Debug)]
struct PyCameraAnimationSpec {
    start_time: f32,
    duration: f32,
    ease: Ease,
    kind: PyCameraAnimationKind,
}

#[derive(Debug)]
struct PyDuringCallbackSpec {
    start_time: f32,
    duration: f32,
    ease: Ease,
    callback: Py<PyAny>,
}

#[derive(Debug)]
struct PyAtCallbackSpec {
    time: f32,
    callback: Py<PyAny>,
}

fn replace_path(scene: &mut Scene, id: usize, path: Path) -> PyResult<()> {
    let Some(tattva) = scene.get_tattva_typed_mut::<Path>(id) else {
        return Err(PyValueError::new_err("update_path expected a Path handle"));
    };
    tattva.state = path;
    Ok(())
}

fn replace_rectangle(scene: &mut Scene, id: usize, rectangle: Rectangle) -> PyResult<()> {
    let Some(tattva) = scene.get_tattva_typed_mut::<Rectangle>(id) else {
        return Err(PyValueError::new_err(
            "update_rectangle expected a Rectangle handle",
        ));
    };
    tattva.state = rectangle;
    Ok(())
}

fn replace_parametric_surface(
    scene: &mut Scene,
    id: usize,
    surface: ParametricSurface,
) -> PyResult<()> {
    let Some(tattva) = scene.get_tattva_typed_mut::<ParametricSurface>(id) else {
        return Err(PyValueError::new_err(
            "update_parametric_surface expected a ParametricSurface handle",
        ));
    };
    tattva.state = surface;
    tattva.mark_dirty(DirtyFlags::GEOMETRY | DirtyFlags::BOUNDS | DirtyFlags::STYLE);
    Ok(())
}

fn set_label_text(scene: &mut Scene, id: usize, text: String) -> PyResult<()> {
    let Some(tattva) = scene.get_tattva_typed_mut::<Label>(id) else {
        return Err(PyValueError::new_err(
            "set_label_text expected a Label handle",
        ));
    };
    tattva.state.text = text;
    Ok(())
}

fn set_label_color(scene: &mut Scene, id: usize, color: Vec4) -> PyResult<()> {
    let Some(tattva) = scene.get_tattva_typed_mut::<Label>(id) else {
        return Err(PyValueError::new_err(
            "set_label_color expected a Label handle",
        ));
    };
    tattva.state.color = color;
    Ok(())
}

fn stop_traced_path(scene: &mut Scene, id: usize) -> PyResult<()> {
    let Some(tattva) = scene.get_tattva_typed_mut::<TracedPath>(id) else {
        return Err(PyValueError::new_err(
            "stop_trace expected a TracedPath handle",
        ));
    };
    tattva.state.stop_recording();
    Ok(())
}

fn invoke_scene_callback(callback: &Py<PyAny>, scene: &mut Scene, time: Option<f32>) {
    Python::with_gil(|py| {
        let tick = match Bound::new(
            py,
            PySceneTick {
                scene: scene as *mut Scene,
            },
        ) {
            Ok(tick) => tick,
            Err(error) => {
                error.print(py);
                return;
            }
        };
        let result = match time {
            Some(time) => callback.call1(py, (&tick, time)),
            None => callback.call1(py, (&tick,)),
        };
        tick.borrow_mut().scene = std::ptr::null_mut();
        if let Err(error) = result {
            error.print(py);
        }
    });
}

fn python_project_start(py: Python<'_>) -> PyResult<PathBuf> {
    let cwd = std::env::current_dir()
        .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
    let Ok(main) = py.import("__main__") else {
        return Ok(cwd);
    };
    let Ok(file) = main.getattr("__file__") else {
        return Ok(cwd);
    };
    let Ok(file) = file.extract::<String>() else {
        return Ok(cwd);
    };
    if file.is_empty() || (file.starts_with('<') && file.ends_with('>')) {
        return Ok(cwd);
    }

    let script = PathBuf::from(file);
    let script = if script.is_absolute() {
        script
    } else {
        cwd.join(script)
    };

    // Test runners and launchers can make __main__.__file__ point into the
    // virtual environment. Prefer it only when it actually belongs to a
    // Murali project; otherwise interactive/test usage remains cwd-based.
    if find_murali_project_root(&script).is_some() {
        Ok(script)
    } else {
        Ok(cwd)
    }
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

fn quat_from_xyz_degrees(x_degrees: f32, y_degrees: f32, z_degrees: f32) -> PyResult<Quat> {
    for (name, value) in [
        ("x_degrees", x_degrees),
        ("y_degrees", y_degrees),
        ("z_degrees", z_degrees),
    ] {
        if !value.is_finite() {
            return Err(PyValueError::new_err(format!(
                "{name} must be a finite number"
            )));
        }
    }
    Ok(Quat::from_euler(
        EulerRot::XYZ,
        x_degrees.to_radians(),
        y_degrees.to_radians(),
        z_degrees.to_radians(),
    ))
}

fn capital_letter(value: &str) -> PyResult<char> {
    let mut chars = value.chars();
    let Some(character) = chars.next() else {
        return Err(PyValueError::new_err("expected a single capital letter"));
    };
    if chars.next().is_some() || !character.is_ascii_uppercase() {
        return Err(PyValueError::new_err(
            "expected a single ASCII capital letter A-Z",
        ));
    }
    Ok(character)
}

fn builtin_texture_from_name(name: &str) -> PyResult<BuiltinTexture> {
    match name {
        "white_marble" | "white-marble" => Ok(BuiltinTexture::WhiteMarble),
        "black_marble" | "black-marble" => Ok(BuiltinTexture::BlackMarble),
        "earth" | "earth_map" | "earth-map" => Ok(BuiltinTexture::EarthMap),
        other => Err(PyValueError::new_err(format!(
            "unknown builtin texture {other:?}; expected white_marble, black_marble, or earth"
        ))),
    }
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

fn chat_bubble_tip_side_from_name(name: &str) -> PyResult<ChatBubbleTipSide> {
    match name {
        "left" => Ok(ChatBubbleTipSide::Left),
        "right" => Ok(ChatBubbleTipSide::Right),
        other => Err(PyValueError::new_err(format!(
            "unknown chat bubble tip side {other:?}; expected left or right"
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

fn context_block_role_from_name(name: &str) -> PyResult<ContextBlockRole> {
    match name {
        "system" => Ok(ContextBlockRole::System),
        "user" => Ok(ContextBlockRole::User),
        "assistant" => Ok(ContextBlockRole::Assistant),
        "tool" => Ok(ContextBlockRole::Tool),
        "retrieved" => Ok(ContextBlockRole::Retrieved),
        other => Err(PyValueError::new_err(format!(
            "unknown context block role {other:?}; expected system, user, assistant, tool, or retrieved"
        ))),
    }
}

fn context_truncation_from_name(name: &str) -> PyResult<ContextTruncation> {
    match name {
        "from_start" | "start" => Ok(ContextTruncation::FromStart),
        "from_end" | "end" => Ok(ContextTruncation::FromEnd),
        other => Err(PyValueError::new_err(format!(
            "unknown context truncation {other:?}; expected from_start or from_end"
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
