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

    fn apply_theme<'py>(
        mut slf: PyRefMut<'py, Self>,
        theme: &Bound<'_, PyAny>,
    ) -> PyResult<PyRefMut<'py, Self>> {
        let background = theme.getattr("background").map_err(|_| {
            PyValueError::new_err("theme must expose a background RGBA tuple")
        })?;
        let background: ColorTuple = background.extract().map_err(|_| {
            PyValueError::new_err("theme.background must be an RGBA tuple")
        })?;
        slf.inner = Some(slf.take_view()?.background(color_from_tuple(background)?));
        Ok(slf)
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
    #[pyo3(signature = (frame = None, background = None))]
    fn new(frame: Option<&str>, background: Option<ColorTuple>) -> PyResult<Self> {
        let mut scene = Scene::new().with_frame(frame_from_name(frame)?);
        if let Some(background) = background {
            scene.set_background(color_from_tuple(background)?);
        }
        Ok(Self { inner: Some(scene) })
    }

    fn set_background(&mut self, color: ColorTuple) -> PyResult<()> {
        self.scene_mut()?.set_background(color_from_tuple(color)?);
        Ok(())
    }

    fn apply_theme<'py>(
        mut slf: PyRefMut<'py, Self>,
        theme: &Bound<'_, PyAny>,
    ) -> PyResult<PyRefMut<'py, Self>> {
        let background = theme.getattr("background").map_err(|_| {
            PyValueError::new_err("theme must expose a background RGBA tuple")
        })?;
        let background: ColorTuple = background.extract().map_err(|_| {
            PyValueError::new_err("theme.background must be an RGBA tuple")
        })?;
        slf.scene_mut()?.set_background(color_from_tuple(background)?);
        Ok(slf)
    }

    fn clear_background(&mut self) -> PyResult<()> {
        self.scene_mut()?.clear_background();
        Ok(())
    }

    fn background(&self) -> PyResult<Option<ColorTuple>> {
        Ok(self.scene()?.background().map(color_tuple))
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
        if let Ok(letter) = tattva.extract::<PyRef<'_, PyLetter3D>>() {
            return Ok(PyTattvaHandle {
                id: scene.add_tattva(letter.inner.clone(), position),
            });
        }
        if let Ok(particles) = tattva.extract::<PyRef<'_, PyLetterParticles3D>>() {
            return Ok(PyTattvaHandle {
                id: scene.add_tattva(particles.inner.clone(), position),
            });
        }
        if let Ok(circle) = tattva.extract::<PyRef<'_, PyCircle>>() {
            return Ok(PyTattvaHandle {
                id: scene.add_tattva(circle.inner.clone(), position),
            });
        }
        if let Ok(belt) = tattva.extract::<PyRef<'_, PyParticleBelt>>() {
            return Ok(PyTattvaHandle {
                id: scene.add_tattva(belt.inner.clone(), position),
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
        if let Ok(rounded_rectangle) = tattva.extract::<PyRef<'_, PyRoundedRectangle>>() {
            return Ok(PyTattvaHandle {
                id: scene.add_tattva(rounded_rectangle.inner.clone(), position),
            });
        }
        if let Ok(chat_bubble) = tattva.extract::<PyRef<'_, PyChatBubble>>() {
            return Ok(PyTattvaHandle {
                id: scene.add_tattva(chat_bubble.inner.clone(), position),
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
        if let Ok(mut traced_path) = tattva.extract::<PyRefMut<'_, PyTracedPath>>() {
            return Ok(PyTattvaHandle {
                id: scene.add_tattva(traced_path.take_inner()?, position),
            });
        }
        if let Ok(code_block) = tattva.extract::<PyRef<'_, PyCodeBlock>>() {
            return Ok(PyTattvaHandle {
                id: scene.add_tattva(code_block.inner.clone(), position),
            });
        }
        if let Ok(context_window) = tattva.extract::<PyRef<'_, PyContextWindow>>() {
            return Ok(PyTattvaHandle {
                id: scene.add_tattva(context_window.inner.clone(), position),
            });
        }
        if let Ok(signal_flow) = tattva.extract::<PyRef<'_, PySignalFlow>>() {
            return Ok(PyTattvaHandle {
                id: scene.add_tattva(signal_flow.inner.clone(), position),
            });
        }
        if let Ok(equation) = tattva.extract::<PyRef<'_, PyEquationLayout>>() {
            return Ok(PyTattvaHandle {
                id: scene.add_tattva(equation.inner.clone(), position),
            });
        }
        if let Ok(matrix) = tattva.extract::<PyRef<'_, PyMatrix>>() {
            return Ok(PyTattvaHandle {
                id: scene.add_tattva(matrix.inner.clone(), position),
            });
        }
        if let Ok(number_line) = tattva.extract::<PyRef<'_, PyNumberLine>>() {
            return Ok(PyTattvaHandle {
                id: scene.add_tattva(number_line.inner.clone(), position),
            });
        }
        if let Ok(optimization_path) = tattva.extract::<PyRef<'_, PyOptimizationPath2D>>() {
            return Ok(PyTattvaHandle {
                id: scene.add_tattva(optimization_path.inner.clone(), position),
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

    fn position(&self, handle: &PyTattvaHandle) -> PyResult<Vec3Tuple> {
        let position = self.scene()?.position(handle.id).ok_or_else(|| {
            PyValueError::new_err("position expected a valid tattva handle")
        })?;
        Ok((position.x, position.y, position.z))
    }

    fn world_bounds(&self, handle: &PyTattvaHandle) -> PyResult<(f32, f32, f32, f32)> {
        let bounds = self.scene()?.world_bounds(handle.id).ok_or_else(|| {
            PyValueError::new_err("world_bounds expected a valid tattva handle")
        })?;
        Ok((bounds.min.x, bounds.min.y, bounds.max.x, bounds.max.y))
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

    fn set_rotation(
        &mut self,
        handle: &PyTattvaHandle,
        x_degrees: f32,
        y_degrees: f32,
        z_degrees: f32,
    ) -> PyResult<()> {
        self.scene_mut()?.set_rotation(
            handle.id,
            quat_from_xyz_degrees(x_degrees, y_degrees, z_degrees)?,
        );
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

    fn update_path(&mut self, handle: &PyTattvaHandle, path: PyRef<'_, PyPath>) -> PyResult<()> {
        replace_path(self.scene_mut()?, handle.id, path.inner.clone())
    }

    fn update_rectangle(
        &mut self,
        handle: &PyTattvaHandle,
        rectangle: PyRef<'_, PyRectangle>,
    ) -> PyResult<()> {
        replace_rectangle(self.scene_mut()?, handle.id, rectangle.inner.clone())
    }

    fn update_parametric_surface(
        &mut self,
        handle: &PyTattvaHandle,
        surface: PyRef<'_, PyParametricSurface>,
    ) -> PyResult<()> {
        replace_parametric_surface(self.scene_mut()?, handle.id, surface.inner.clone())
    }

    fn set_label_text(&mut self, handle: &PyTattvaHandle, text: String) -> PyResult<()> {
        set_label_text(self.scene_mut()?, handle.id, text)
    }

    fn set_label_color(&mut self, handle: &PyTattvaHandle, color: ColorTuple) -> PyResult<()> {
        set_label_color(self.scene_mut()?, handle.id, color_from_tuple(color)?)
    }

    fn stop_trace(&mut self, handle: &PyTattvaHandle) -> PyResult<()> {
        stop_traced_path(self.scene_mut()?, handle.id)
    }

    fn hide(&mut self, handle: &PyTattvaHandle) -> PyResult<()> {
        self.scene_mut()?.hide(handle.id);
        Ok(())
    }

    fn show(&mut self, handle: &PyTattvaHandle) -> PyResult<()> {
        self.scene_mut()?.show(handle.id);
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

    fn add_updater(&mut self, callback: Bound<'_, PyAny>) -> PyResult<usize> {
        if !callback.is_callable() {
            return Err(PyValueError::new_err("callback must be callable"));
        }
        let callback = callback.unbind();
        Ok(self.scene_mut()?.add_updater(0, move |scene, _, _dt| {
            let time = scene.scene_time;
            invoke_scene_callback(&callback, scene, Some(time));
        }))
    }

    fn set_view_width(&mut self, width: f32) -> PyResult<()> {
        if width <= 0.0 || !width.is_finite() {
            return Err(PyValueError::new_err(
                "width must be a positive finite number",
            ));
        }
        self.scene_mut()?.camera_mut().set_view_width(width);
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

    #[pyo3(signature = (auto_close = false, hold = 3.0))]
    fn preview(&mut self, py: Python<'_>, auto_close: bool, hold: f32) -> PyResult<()> {
        if hold < 0.0 || !hold.is_finite() {
            return Err(PyValueError::new_err(
                "hold must be a non-negative finite number of seconds",
            ));
        }
        let scene = self.take_scene()?;
        let mut app = App::new_from(python_project_start(py)?)
            .map_err(|error| PyRuntimeError::new_err(error.to_string()))?
            .with_scene(scene)
            .with_preview();
        if auto_close {
            app = app.with_auto_close(hold);
        }
        app.run_app()
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

    #[pyo3(signature = (path, width = None, fps = None, duration = None, preserve_frames = false))]
    fn export_video(
        &mut self,
        path: String,
        width: Option<u32>,
        fps: Option<u32>,
        duration: Option<f32>,
        preserve_frames: bool,
    ) -> PyResult<String> {
        let requested_path = resolve_output_path(path)?;
        let requested_is_mp4 = requested_path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.eq_ignore_ascii_case("mp4"))
            .unwrap_or(false);

        let artifact_dir = if requested_is_mp4 {
            let stem = requested_path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .filter(|stem| !stem.is_empty())
                .ok_or_else(|| PyValueError::new_err("video path must include a file name"))?;
            requested_path
                .parent()
                .map(|parent| parent.join(stem))
                .unwrap_or_else(|| PathBuf::from(stem))
        } else {
            requested_path.clone()
        };

        let scene = self.take_scene()?;
        let mut settings = ExportSettings::from_scene(&scene);
        settings.video_enabled = true;
        settings.preserve_frame_exports = preserve_frames;
        settings.artifact_dir = artifact_dir;
        apply_export_options(&mut settings, width, fps, duration)?;

        let video_path = settings.video_path();
        export_scene(scene, &settings)
            .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;

        let final_path = if requested_is_mp4 {
            if let Some(parent) = requested_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
            }
            if video_path != requested_path {
                std::fs::copy(&video_path, &requested_path)
                    .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
            }
            requested_path
        } else {
            video_path
        };

        Ok(final_path.to_string_lossy().to_string())
    }

    #[pyo3(signature = (path, width = None, fps = None, duration = None, preserve_frames = false))]
    fn export(
        &mut self,
        path: String,
        width: Option<u32>,
        fps: Option<u32>,
        duration: Option<f32>,
        preserve_frames: bool,
    ) -> PyResult<String> {
        self.export_video(path, width, fps, duration, preserve_frames)
    }
}

fn resolve_output_path(path: String) -> PyResult<PathBuf> {
    let mut output_path = PathBuf::from(path);
    if !output_path.is_absolute() {
        output_path = std::env::current_dir()
            .map_err(|error| PyRuntimeError::new_err(error.to_string()))?
            .join(output_path);
    }
    Ok(output_path)
}

fn apply_export_options(
    settings: &mut ExportSettings,
    width: Option<u32>,
    fps: Option<u32>,
    duration: Option<f32>,
) -> PyResult<()> {
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
    Ok(())
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
                Some(PyAnimationKind::EquationContinuityFrom(source_id)) => {
                    builder.equation_continuity_from(*source_id).spawn()
                }
                Some(PyAnimationKind::MatrixStepCells {
                    cells,
                    highlight,
                    dim_opacity,
                }) => builder
                    .matrix_step_cells(cells.clone(), *highlight, *dim_opacity)
                    .spawn(),
                Some(PyAnimationKind::MatrixStepRow {
                    row,
                    highlight,
                    dim_opacity,
                }) => builder
                    .matrix_step_row(*row, *highlight, *dim_opacity)
                    .spawn(),
                Some(PyAnimationKind::MatrixStepColumn {
                    col,
                    highlight,
                    dim_opacity,
                }) => builder
                    .matrix_step_column(*col, *highlight, *dim_opacity)
                    .spawn(),
                Some(PyAnimationKind::WriteTable) => builder.write_table().spawn(),
                Some(PyAnimationKind::UnwriteTable) => builder.unwrite_table().spawn(),
                Some(PyAnimationKind::WriteSurface) => builder.write_surface().spawn(),
                Some(PyAnimationKind::UnwriteSurface) => builder.unwrite_surface().spawn(),
                Some(PyAnimationKind::LetterParticleScatterTo(to)) => {
                    builder.letter_particle_scatter_to(*to).spawn()
                }
                Some(PyAnimationKind::MorphFrom(source_id)) => {
                    builder.morph_from(*source_id).spawn()
                }
                Some(PyAnimationKind::BeltEvolve { speed }) => match speed {
                    Some(speed) => builder.belt_evolve_with_speed(*speed).spawn(),
                    None => builder.belt_evolve().spawn(),
                },
                None => {
                    return Err(PyValueError::new_err(
                        "animation kind is missing; call appear, draw, move_to, rotate_to, scale_to, fade_to, typewrite_text, reveal_text, hide_text, indicate, equation_continuity_from, matrix_step_row, matrix_step_column, matrix_step_cells, write_table, write_surface, belt_evolve, or morph_from before spawn",
                    ));
                }
            }
        }
        for spec in &self.signal_specs {
            let playback = match spec.mode {
                PySignalPlaybackMode::Once => {
                    SignalPlayback::once(spec.start_time, spec.duration, spec.ease)
                }
                PySignalPlaybackMode::RoundTrip => {
                    SignalPlayback::round_trip(spec.start_time, spec.duration, spec.ease)
                }
                PySignalPlaybackMode::Loop => {
                    SignalPlayback::looped(spec.start_time, spec.duration, spec.repeats, spec.ease)
                }
            };
            timeline.play_signal(spec.target_id, playback);
        }
        for spec in &self.camera_specs {
            let builder = timeline
                .animate_camera()
                .at(spec.start_time)
                .for_duration(spec.duration)
                .ease(spec.ease);
            match spec.kind {
                PyCameraAnimationKind::FrameTo { position, target } => {
                    builder.frame_to(position, target).spawn();
                }
                PyCameraAnimationKind::ZoomTo { zoom } => {
                    builder.zoom_to(zoom).spawn();
                }
            }
        }
        for spec in &self.during_specs {
            let callback = Python::with_gil(|py| spec.callback.clone_ref(py));
            let ease = spec.ease;
            timeline.call_during(spec.start_time, spec.duration, move |scene, t| {
                invoke_scene_callback(&callback, scene, Some(ease.eval(t)));
            });
        }
        for spec in &self.at_specs {
            let callback = Python::with_gil(|py| spec.callback.clone_ref(py));
            timeline.call_at(spec.time, move |scene| {
                invoke_scene_callback(&callback, scene, None);
            });
        }
        if self.hold_until > 0.0 {
            timeline.wait_until(self.hold_until);
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
