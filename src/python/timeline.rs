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

    fn rotate_xyz<'py>(
        mut slf: PyRefMut<'py, Self>,
        x_degrees: f32,
        y_degrees: f32,
        z_degrees: f32,
    ) -> PyResult<PyRefMut<'py, Self>> {
        slf.spec.kind = Some(PyAnimationKind::RotateTo(quat_from_xyz_degrees(
            x_degrees, y_degrees, z_degrees,
        )?));
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

    fn equation_continuity_from<'py>(
        mut slf: PyRefMut<'py, Self>,
        source: &PyTattvaHandle,
    ) -> PyResult<PyRefMut<'py, Self>> {
        slf.spec.kind = Some(PyAnimationKind::EquationContinuityFrom(source.id));
        Ok(slf)
    }

    fn matrix_step_row<'py>(
        mut slf: PyRefMut<'py, Self>,
        row: usize,
        highlight: ColorTuple,
        dim_opacity: f32,
    ) -> PyResult<PyRefMut<'py, Self>> {
        validate_dim_opacity(dim_opacity)?;
        slf.spec.kind = Some(PyAnimationKind::MatrixStepRow {
            row,
            highlight: color_from_tuple(highlight)?,
            dim_opacity,
        });
        Ok(slf)
    }

    fn matrix_step_column<'py>(
        mut slf: PyRefMut<'py, Self>,
        col: usize,
        highlight: ColorTuple,
        dim_opacity: f32,
    ) -> PyResult<PyRefMut<'py, Self>> {
        validate_dim_opacity(dim_opacity)?;
        slf.spec.kind = Some(PyAnimationKind::MatrixStepColumn {
            col,
            highlight: color_from_tuple(highlight)?,
            dim_opacity,
        });
        Ok(slf)
    }

    fn matrix_step_cells<'py>(
        mut slf: PyRefMut<'py, Self>,
        cells: Vec<(usize, usize)>,
        highlight: ColorTuple,
        dim_opacity: f32,
    ) -> PyResult<PyRefMut<'py, Self>> {
        if cells.is_empty() {
            return Err(PyValueError::new_err(
                "matrix_step_cells requires at least one cell",
            ));
        }
        validate_dim_opacity(dim_opacity)?;
        slf.spec.kind = Some(PyAnimationKind::MatrixStepCells {
            cells,
            highlight: color_from_tuple(highlight)?,
            dim_opacity,
        });
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

    fn morph_from<'py>(
        mut slf: PyRefMut<'py, Self>,
        source: &PyTattvaHandle,
    ) -> PyResult<PyRefMut<'py, Self>> {
        slf.spec.kind = Some(PyAnimationKind::MorphFrom(source.id));
        Ok(slf)
    }

    fn letter_particle_scatter_to<'py>(
        mut slf: PyRefMut<'py, Self>,
        to: f32,
    ) -> PyResult<PyRefMut<'py, Self>> {
        if !(0.0..=1.0).contains(&to) || !to.is_finite() {
            return Err(PyValueError::new_err(
                "scatter amount must be a finite number between 0 and 1",
            ));
        }
        slf.spec.kind = Some(PyAnimationKind::LetterParticleScatterTo(to));
        Ok(slf)
    }

    fn belt_evolve<'py>(mut slf: PyRefMut<'py, Self>) -> PyResult<PyRefMut<'py, Self>> {
        slf.spec.kind = Some(PyAnimationKind::BeltEvolve { speed: None });
        Ok(slf)
    }

    fn belt_evolve_with_speed<'py>(
        mut slf: PyRefMut<'py, Self>,
        speed: f32,
    ) -> PyResult<PyRefMut<'py, Self>> {
        if !speed.is_finite() {
            return Err(PyValueError::new_err("speed must be a finite number"));
        }
        slf.spec.kind = Some(PyAnimationKind::BeltEvolve { speed: Some(speed) });
        Ok(slf)
    }

    fn spawn(&self, py: Python<'_>) -> PyResult<()> {
        if self.spec.kind.is_none() {
            return Err(PyValueError::new_err(
                "animation kind is missing; call appear, draw, move_to, rotate_to, rotate_xyz, scale_to, fade_to, typewrite_text, reveal_text, hide_text, indicate, equation_continuity_from, matrix_step_row, matrix_step_column, matrix_step_cells, write_table, write_surface, letter_particle_scatter_to, belt_evolve, belt_evolve_with_speed, or morph_from before spawn",
            ));
        }
        let mut timeline = self.timeline.borrow_mut(py);
        timeline.specs.push(self.spec.clone());
        Ok(())
    }
}

fn validate_dim_opacity(dim_opacity: f32) -> PyResult<()> {
    if !(0.0..=1.0).contains(&dim_opacity) || !dim_opacity.is_finite() {
        return Err(PyValueError::new_err(
            "dim_opacity must be a finite number between 0 and 1",
        ));
    }
    Ok(())
}

#[pyclass(name = "Timeline", module = "murali_engine")]
#[derive(Debug, Default)]
struct PyTimeline {
    specs: Vec<PyAnimationSpec>,
    signal_specs: Vec<PySignalPlaybackSpec>,
    camera_specs: Vec<PyCameraAnimationSpec>,
    during_specs: Vec<PyDuringCallbackSpec>,
    at_specs: Vec<PyAtCallbackSpec>,
    hold_until: f32,
}

#[pymethods]
impl PyTimeline {
    #[new]
    fn new() -> Self {
        Self {
            specs: Vec::new(),
            signal_specs: Vec::new(),
            camera_specs: Vec::new(),
            during_specs: Vec::new(),
            at_specs: Vec::new(),
            hold_until: 0.0,
        }
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

    #[pyo3(signature = (target, start_time, duration, ease = "linear", mode = "once", repeats = 1))]
    fn play_signal(
        &mut self,
        target: &PyTattvaHandle,
        start_time: f32,
        duration: f32,
        ease: &str,
        mode: &str,
        repeats: usize,
    ) -> PyResult<()> {
        validate_non_negative_finite("start_time", start_time)?;
        if duration <= 0.0 || !duration.is_finite() {
            return Err(PyValueError::new_err(
                "duration must be a positive finite number",
            ));
        }
        let mode = match mode {
            "once" => PySignalPlaybackMode::Once,
            "round_trip" | "roundtrip" => PySignalPlaybackMode::RoundTrip,
            "loop" | "looped" => PySignalPlaybackMode::Loop,
            other => {
                return Err(PyValueError::new_err(format!(
                    "unknown signal playback mode {other:?}; expected once, round_trip, or loop"
                )));
            }
        };
        if matches!(mode, PySignalPlaybackMode::Loop) && repeats == 0 {
            return Err(PyValueError::new_err(
                "repeats must be greater than zero for loop signal playback",
            ));
        }
        self.signal_specs.push(PySignalPlaybackSpec {
            target_id: target.id,
            start_time,
            duration,
            ease: ease_from_name(ease)?,
            mode,
            repeats,
        });
        Ok(())
    }

    #[pyo3(signature = (start_time, duration, position, target, ease = "linear"))]
    fn animate_camera_frame(
        &mut self,
        start_time: f32,
        duration: f32,
        position: Vec3Tuple,
        target: Vec3Tuple,
        ease: &str,
    ) -> PyResult<()> {
        validate_non_negative_finite("start_time", start_time)?;
        if duration <= 0.0 || !duration.is_finite() {
            return Err(PyValueError::new_err(
                "duration must be a positive finite number",
            ));
        }
        self.camera_specs.push(PyCameraAnimationSpec {
            start_time,
            duration,
            ease: ease_from_name(ease)?,
            kind: PyCameraAnimationKind::FrameTo {
                position: vec3_from_tuple(Some(position))?,
                target: vec3_from_tuple(Some(target))?,
            },
        });
        Ok(())
    }

    #[pyo3(signature = (start_time, duration, zoom, ease = "linear"))]
    fn zoom_camera(
        &mut self,
        start_time: f32,
        duration: f32,
        zoom: f32,
        ease: &str,
    ) -> PyResult<()> {
        validate_non_negative_finite("start_time", start_time)?;
        if duration <= 0.0 || !duration.is_finite() {
            return Err(PyValueError::new_err(
                "duration must be a positive finite number",
            ));
        }
        if zoom <= 0.0 || !zoom.is_finite() {
            return Err(PyValueError::new_err(
                "zoom must be a positive finite number",
            ));
        }
        self.camera_specs.push(PyCameraAnimationSpec {
            start_time,
            duration,
            ease: ease_from_name(ease)?,
            kind: PyCameraAnimationKind::ZoomTo { zoom },
        });
        Ok(())
    }

    #[pyo3(signature = (start_time, duration, callback, ease = "linear"))]
    fn call_during(
        &mut self,
        start_time: f32,
        duration: f32,
        callback: Bound<'_, PyAny>,
        ease: &str,
    ) -> PyResult<()> {
        validate_non_negative_finite("start_time", start_time)?;
        if duration <= 0.0 || !duration.is_finite() {
            return Err(PyValueError::new_err(
                "duration must be a positive finite number",
            ));
        }
        if !callback.is_callable() {
            return Err(PyValueError::new_err("callback must be callable"));
        }
        self.during_specs.push(PyDuringCallbackSpec {
            start_time,
            duration,
            ease: ease_from_name(ease)?,
            callback: callback.unbind(),
        });
        Ok(())
    }

    fn wait_until(&mut self, timestamp: f32) -> PyResult<()> {
        validate_non_negative_finite("timestamp", timestamp)?;
        self.hold_until = self.hold_until.max(timestamp);
        Ok(())
    }

    fn call_at(&mut self, time: f32, callback: Bound<'_, PyAny>) -> PyResult<()> {
        validate_non_negative_finite("time", time)?;
        if !callback.is_callable() {
            return Err(PyValueError::new_err("callback must be callable"));
        }
        self.at_specs.push(PyAtCallbackSpec {
            time,
            callback: callback.unbind(),
        });
        Ok(())
    }

    fn len(&self) -> usize {
        self.specs.len()
            + self.signal_specs.len()
            + self.camera_specs.len()
            + self.during_specs.len()
            + self.at_specs.len()
    }

    fn is_empty(&self) -> bool {
        self.specs.is_empty()
            && self.signal_specs.is_empty()
            && self.camera_specs.is_empty()
            && self.during_specs.is_empty()
            && self.at_specs.is_empty()
    }
}

#[pyclass(name = "SceneTick", module = "murali_engine", unsendable)]
struct PySceneTick {
    scene: *mut Scene,
}

impl PySceneTick {
    fn scene_mut(&mut self) -> PyResult<&mut Scene> {
        if self.scene.is_null() {
            return Err(PyRuntimeError::new_err(
                "this SceneTick is only valid during a timeline or updater callback",
            ));
        }
        Ok(unsafe { &mut *self.scene })
    }
}

#[pymethods]
impl PySceneTick {
    fn set_position(&mut self, handle: &PyTattvaHandle, position: Vec3Tuple) -> PyResult<()> {
        let position = vec3_from_tuple(Some(position))?;
        self.scene_mut()?.set_position_3d(handle.id, position);
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

    fn hide(&mut self, handle: &PyTattvaHandle) -> PyResult<()> {
        self.scene_mut()?.hide(handle.id);
        Ok(())
    }

    fn show(&mut self, handle: &PyTattvaHandle) -> PyResult<()> {
        self.scene_mut()?.show(handle.id);
        Ok(())
    }

    fn stop_trace(&mut self, handle: &PyTattvaHandle) -> PyResult<()> {
        stop_traced_path(self.scene_mut()?, handle.id)
    }
}
