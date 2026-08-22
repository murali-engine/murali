use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use glam::{EulerRot, Quat, Vec3, Vec4};
use murali::frontend::collection::primitives::prop3d::Prop3D;
use murali::frontend::collection::text::label::Label;
use murali::{App, DepthMode};
use murali::{engine::camera::Projection, engine::scene::Scene};

const ASPECT_RATIO: f32 = 16.0 / 9.0;
const FIT_SPAN: f32 = 4.2;
const DEFAULT_MODEL: &str = "demo-apple";

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return Ok(());
    }
    let options = InspectorOptions::from_args(args)?;
    let model_path = resolve_model_path(&options.model)?;
    let prop = Prop3D::from_file(&model_path)?;
    let model_dimensions = prop.dimensions();
    let model_center = prop.center();
    let fitted_base_scale = fitted_scale(model_dimensions, Vec3::ONE, options.fit)?;
    let effective_scale = fitted_base_scale * options.scale;
    let framing_dimensions = model_dimensions * fitted_base_scale.abs();
    let displayed_dimensions = model_dimensions * effective_scale.abs();
    let radius = displayed_dimensions.length() * 0.5;
    let camera_distance = options
        .camera_distance
        .unwrap_or_else(|| framing_distance(framing_dimensions, options.fov, ASPECT_RATIO));
    let viewport_height = 2.0 * camera_distance * (options.fov.to_radians() * 0.5).tan();

    let mut scene = Scene::new();
    scene.camera_mut().projection = Projection::Perspective {
        fov_y_rad: options.fov.to_radians(),
        aspect: ASPECT_RATIO,
        near: (camera_distance - radius * 2.0).max(0.01),
        far: camera_distance + radius * 4.0 + 10.0,
    };
    scene.camera_mut().position = Vec3::new(0.0, radius * 0.12, camera_distance);
    scene.camera_mut().target = Vec3::ZERO;

    let title_y = viewport_height * 0.39;
    let footer_y = -viewport_height * 0.41;
    let title = scene.add_tattva(
        Label::new("3D Model Preview", viewport_height * 0.035)
            .with_color(Vec4::new(0.96, 0.95, 0.90, 1.0)),
        Vec3::new(0.0, title_y, 0.0),
    );
    let file_name = model_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("model");
    let details = scene.add_tattva(
        Label::new(
            format!(
                "{}  |  {} meshes  |  {:.2} x {:.2} x {:.2} units",
                file_name,
                prop.mesh_count(),
                model_dimensions.x,
                model_dimensions.y,
                model_dimensions.z,
            ),
            viewport_height * 0.018,
        )
        .with_color(Vec4::new(0.70, 0.76, 0.84, 1.0)),
        Vec3::new(0.0, title_y - viewport_height * 0.055, 0.0),
    );
    let controls = scene.add_tattva(
        Label::new(
            "Drag: orbit   Wheel: zoom   O: orbit   P: pan",
            viewport_height * 0.017,
        )
        .with_color(Vec4::new(0.62, 0.68, 0.76, 1.0)),
        Vec3::new(0.0, footer_y, 0.0),
    );

    let base_rotation = rotation_at(&options, 0.0);
    let centered_position = options.position - base_rotation * (model_center * effective_scale);
    let prop_id = scene.add_tattva(prop, centered_position);
    scene.set_scale(prop_id, effective_scale);
    scene.set_rotation(prop_id, base_rotation);

    for id in [title, details, controls] {
        scene.set_depth_mode(id, DepthMode::Overlay);
    }

    if options.rotate_speed != 0.0 {
        let center = model_center * effective_scale;
        let position = options.position;
        let rotation_x = options.rotation.x;
        let rotation_y = options.rotation.y;
        let rotation_z = options.rotation.z;
        let speed = options.rotate_speed;
        scene.add_updater(prop_id, move |scene, id, _dt| {
            let rotation = Quat::from_euler(
                EulerRot::XYZ,
                rotation_x.to_radians(),
                (rotation_y + scene.scene_time * speed).to_radians(),
                rotation_z.to_radians(),
            );
            scene.set_rotation(id, rotation);
            scene.set_position_3d(id, position - rotation * center);
        });
    }

    println!("Model: {}", model_path.display());
    println!(
        "Source dimensions: {:.3} x {:.3} x {:.3}; effective scale: {:.3} x {:.3} x {:.3}",
        model_dimensions.x,
        model_dimensions.y,
        model_dimensions.z,
        effective_scale.x,
        effective_scale.y,
        effective_scale.z,
    );

    App::new()?.with_scene(scene).with_preview().run_app()
}

#[derive(Debug, Clone)]
struct InspectorOptions {
    model: String,
    scale: Vec3,
    position: Vec3,
    rotation: Vec3,
    camera_distance: Option<f32>,
    fov: f32,
    fit: bool,
    rotate_speed: f32,
}

impl InspectorOptions {
    fn from_args(args: Vec<String>) -> Result<Self> {
        let mut model = None;
        let mut uniform_scale = 1.0;
        let mut scale_x = None;
        let mut scale_y = None;
        let mut scale_z = None;
        let mut position = Vec3::ZERO;
        let mut rotation = Vec3::ZERO;
        let mut camera_distance = None;
        let mut fov = 42.0;
        let mut fit = true;
        let mut rotate_speed = 24.0;

        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--scale" => set_value(&args, &mut i, "--scale", &mut uniform_scale)?,
                "--scale-x" => set_optional_value(&args, &mut i, "--scale-x", &mut scale_x)?,
                "--scale-y" => set_optional_value(&args, &mut i, "--scale-y", &mut scale_y)?,
                "--scale-z" => set_optional_value(&args, &mut i, "--scale-z", &mut scale_z)?,
                "--x" => set_value(&args, &mut i, "--x", &mut position.x)?,
                "--y" => set_value(&args, &mut i, "--y", &mut position.y)?,
                "--z" => set_value(&args, &mut i, "--z", &mut position.z)?,
                "--rot-x" => set_value(&args, &mut i, "--rot-x", &mut rotation.x)?,
                "--rot-y" => set_value(&args, &mut i, "--rot-y", &mut rotation.y)?,
                "--rot-z" => set_value(&args, &mut i, "--rot-z", &mut rotation.z)?,
                "--camera" => {
                    let mut value = 0.0;
                    set_value(&args, &mut i, "--camera", &mut value)?;
                    camera_distance = Some(value);
                }
                "--fov" => set_value(&args, &mut i, "--fov", &mut fov)?,
                "--rotate-speed" => set_value(&args, &mut i, "--rotate-speed", &mut rotate_speed)?,
                "--no-rotate" => {
                    rotate_speed = 0.0;
                    i += 1;
                }
                "--no-fit" => {
                    fit = false;
                    i += 1;
                }
                "--debug" | "--auto-close" | "--preview" => {
                    i += 1;
                }
                value if value.starts_with("--") => bail!("unknown option: {value}"),
                value => {
                    if model.replace(value.to_string()).is_some() {
                        bail!("only one model path is supported");
                    }
                    i += 1;
                }
            }
        }

        let options = Self {
            model: model.unwrap_or_else(|| DEFAULT_MODEL.to_string()),
            scale: Vec3::new(
                scale_x.unwrap_or(uniform_scale),
                scale_y.unwrap_or(uniform_scale),
                scale_z.unwrap_or(uniform_scale),
            ),
            position,
            rotation,
            camera_distance,
            fov,
            fit,
            rotate_speed,
        };
        options.validate()?;
        Ok(options)
    }

    fn validate(&self) -> Result<()> {
        if !self.scale.is_finite()
            || self.scale.x.abs() <= f32::EPSILON
            || self.scale.y.abs() <= f32::EPSILON
            || self.scale.z.abs() <= f32::EPSILON
        {
            bail!("scale values must be finite and non-zero");
        }
        if !self.position.is_finite()
            || !self.rotation.is_finite()
            || !self.rotate_speed.is_finite()
        {
            bail!("position, rotation, and speed values must be finite");
        }
        if !(10.0..=120.0).contains(&self.fov) {
            bail!("--fov must be between 10 and 120 degrees");
        }
        if self
            .camera_distance
            .is_some_and(|distance| !distance.is_finite() || distance <= 0.0)
        {
            bail!("--camera must be finite and greater than zero");
        }
        Ok(())
    }
}

fn set_value(args: &[String], index: &mut usize, name: &str, target: &mut f32) -> Result<()> {
    *target = parse_value(args, *index, name)?;
    *index += 2;
    Ok(())
}

fn set_optional_value(
    args: &[String],
    index: &mut usize,
    name: &str,
    target: &mut Option<f32>,
) -> Result<()> {
    *target = Some(parse_value(args, *index, name)?);
    *index += 2;
    Ok(())
}

fn parse_value(args: &[String], index: usize, name: &str) -> Result<f32> {
    args.get(index + 1)
        .with_context(|| format!("{name} needs a number"))?
        .parse::<f32>()
        .with_context(|| format!("{name} needs a valid number"))
}

fn fitted_scale(dimensions: Vec3, requested: Vec3, fit: bool) -> Result<Vec3> {
    let longest_side = dimensions.max_element();
    if !longest_side.is_finite() || longest_side <= f32::EPSILON {
        bail!("model has empty or invalid 3D bounds");
    }
    let fit_scale = if fit { FIT_SPAN / longest_side } else { 1.0 };
    Ok(requested * fit_scale)
}

fn framing_distance(dimensions: Vec3, fov_degrees: f32, aspect: f32) -> f32 {
    let half = dimensions * 0.5;
    let tan_vertical = (fov_degrees.to_radians() * 0.5).tan();
    let vertical = half.y / tan_vertical;
    let horizontal = half.x / (tan_vertical * aspect);
    ((vertical.max(horizontal) + half.z) * 1.35).max(1.0)
}

fn rotation_at(options: &InspectorOptions, elapsed: f32) -> Quat {
    Quat::from_euler(
        EulerRot::XYZ,
        options.rotation.x.to_radians(),
        (options.rotation.y + elapsed * options.rotate_speed).to_radians(),
        options.rotation.z.to_radians(),
    )
}

fn resolve_model_path(input: &str) -> Result<PathBuf> {
    let requested = PathBuf::from(input);
    if requested.is_file() {
        return Ok(requested);
    }
    if requested.is_dir() {
        return model_in_directory(&requested)?
            .with_context(|| format!("no .glb or .gltf model found in {}", requested.display()));
    }

    let asset_path = Path::new("assets/props").join(&requested);
    if asset_path.is_file() {
        return Ok(asset_path);
    }
    if asset_path.is_dir() {
        return model_in_directory(&asset_path)?
            .with_context(|| format!("no .glb or .gltf model found in {}", asset_path.display()));
    }

    bail!("could not find model or asset directory: {input}");
}

fn model_in_directory(directory: &Path) -> Result<Option<PathBuf>> {
    for name in ["scene.glb", "scene.gltf"] {
        let candidate = directory.join(name);
        if candidate.is_file() {
            return Ok(Some(candidate));
        }
    }

    let mut models = std::fs::read_dir(directory)?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| {
                        extension.eq_ignore_ascii_case("glb")
                            || extension.eq_ignore_ascii_case("gltf")
                    })
        })
        .collect::<Vec<_>>();
    models.sort();
    match models.len() {
        0 => Ok(None),
        1 => Ok(models.pop()),
        _ => bail!(
            "multiple models found in {}; pass one file explicitly",
            directory.display()
        ),
    }
}

fn print_help() {
    println!(
        "\
Usage:
  cargo run --example model_inspector -- [model-or-directory] [options]

The model is centered and fitted automatically. Scale options adjust the fitted result.
If no model is provided, the bundled demo apple model is used.

Options:
  --scale N          Uniform scale multiplier
  --scale-x N        Override X scale multiplier
  --scale-y N        Override Y scale multiplier
  --scale-z N        Override Z scale multiplier
  --x N              Model X offset
  --y N              Model Y offset
  --z N              Model Z offset
  --rot-x DEG        Initial X rotation in degrees
  --rot-y DEG        Initial Y rotation in degrees
  --rot-z DEG        Initial Z rotation in degrees
  --camera N         Override automatic camera distance
  --fov DEG          Perspective field of view (10-120; default 42)
  --rotate-speed DEG Automatic Y rotation speed per second (default 24)
  --no-rotate        Disable automatic model rotation
  --no-fit           Preserve the model's original scale
  --debug            Print live preview frame timing
  --auto-close       Close the preview automatically after five seconds
  -h, --help         Show this help

Examples:
  cargo run --example model_inspector -- demo-apple
  cargo run --example model_inspector -- assets/props/demo-pyramid.glb --rot-x -20
  cargo run --example model_inspector -- /absolute/path/to/model.glb --scale 0.8
"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_fitted_continuous_preview() {
        let options = InspectorOptions::from_args(vec!["model.glb".into()]).unwrap();

        assert!(options.fit);
        assert_eq!(options.scale, Vec3::ONE);
        assert_eq!(options.rotate_speed, 24.0);
        assert_eq!(options.camera_distance, None);
    }

    #[test]
    fn uses_demo_model_when_no_path_is_provided() {
        let options = InspectorOptions::from_args(Vec::new()).unwrap();

        assert_eq!(options.model, DEFAULT_MODEL);
    }

    #[test]
    fn rejects_invalid_projection_values() {
        let error =
            InspectorOptions::from_args(vec!["model.glb".into(), "--fov".into(), "0".into()])
                .unwrap_err();

        assert!(error.to_string().contains("between 10 and 120"));
    }

    #[test]
    fn fitting_respects_non_uniform_scale_multipliers() {
        let scale = fitted_scale(Vec3::new(2.0, 4.0, 1.0), Vec3::new(3.0, 1.0, 1.0), true).unwrap();

        assert!(scale.abs_diff_eq(Vec3::new(3.15, 1.05, 1.05), 1e-5));
    }

    #[test]
    fn resolves_asset_directories_to_their_model() {
        let path = resolve_model_path("demo-apple").unwrap();

        assert!(path.ends_with("assets/props/demo-apple/demo-apple.gltf"));
    }
}
