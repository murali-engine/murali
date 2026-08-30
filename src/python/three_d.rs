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

    fn with_step(mut slf: PyRefMut<'_, Self>, step: f32) -> PyResult<PyRefMut<'_, Self>> {
        if step <= 0.0 || !step.is_finite() {
            return Err(PyValueError::new_err(
                "step must be a positive finite number",
            ));
        }
        slf.inner = slf.inner.clone().with_step(step);
        Ok(slf)
    }

    fn with_axis_thickness(mut slf: PyRefMut<'_, Self>, thickness: f32) -> PyResult<PyRefMut<'_, Self>> {
        if thickness <= 0.0 || !thickness.is_finite() {
            return Err(PyValueError::new_err(
                "thickness must be a positive finite number",
            ));
        }
        slf.inner = slf.inner.clone().with_axis_thickness(thickness);
        Ok(slf)
    }

    fn with_tick_size(mut slf: PyRefMut<'_, Self>, tick_size: f32) -> PyResult<PyRefMut<'_, Self>> {
        if tick_size < 0.0 || !tick_size.is_finite() {
            return Err(PyValueError::new_err(
                "tick_size must be a non-negative finite number",
            ));
        }
        slf.inner = slf.inner.clone().with_tick_size(tick_size);
        Ok(slf)
    }

    fn without_ticks(mut slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().without_ticks();
        slf
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
    #[pyo3(signature = (kind = "helix", t_range = None, samples = 160, color = None, thickness = 0.03))]
    fn named(
        kind: &str,
        t_range: Option<(f32, f32)>,
        samples: usize,
        color: Option<ColorTuple>,
        thickness: f32,
    ) -> PyResult<Self> {
        let t_range = t_range.unwrap_or((0.0, std::f32::consts::TAU));
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

    #[staticmethod]
    #[pyo3(signature = (source, target = None, mix = 0.0, samples = (72, 144), half_size = (5.9, 3.5), color = None))]
    fn from_map_projection(
        source: &str,
        target: Option<&str>,
        mix: f32,
        samples: (usize, usize),
        half_size: (f32, f32),
        color: Option<ColorTuple>,
    ) -> PyResult<Self> {
        let source = map_projection_kind_from_name(source)?;
        let target = match target {
            Some(name) => map_projection_kind_from_name(name)?,
            None => source,
        };
        if samples.0 < 2 || samples.1 < 2 {
            return Err(PyValueError::new_err(
                "samples must be at least 2 in each direction",
            ));
        }
        if half_size.0 <= 0.0 || half_size.1 <= 0.0 || !half_size.0.is_finite() || !half_size.1.is_finite()
        {
            return Err(PyValueError::new_err(
                "half_size values must be positive finite numbers",
            ));
        }
        if !mix.is_finite() {
            return Err(PyValueError::new_err("mix must be a finite number"));
        }
        Ok(Self {
            inner: ParametricSurface::new(
                crate::math::map_projection::u_range(),
                crate::math::map_projection::v_range(),
                crate::math::map_projection::surface_function(
                    source,
                    target,
                    mix,
                    half_size.0,
                    half_size.1,
                ),
            )
            .with_samples(samples.0, samples.1)
            .with_color(color_from_tuple(
                color.unwrap_or(color_tuple(colors::WHITE)),
            )?),
        })
    }

    #[staticmethod]
    #[pyo3(signature = (function, u_range = None, v_range = None, samples = (32, 32), color = None, render_mode = "solid"))]
    fn from_function(
        function: Bound<'_, PyAny>,
        u_range: Option<(f32, f32)>,
        v_range: Option<(f32, f32)>,
        samples: (usize, usize),
        color: Option<ColorTuple>,
        render_mode: &str,
    ) -> PyResult<Self> {
        let u_range = u_range.unwrap_or((0.0, std::f32::consts::PI));
        let v_range = v_range.unwrap_or((0.0, std::f32::consts::TAU));
        Ok(Self {
            inner: surface_from_python_function(&function, u_range, v_range, samples)?
                .with_color(color_from_tuple(
                    color.unwrap_or(color_tuple(colors::TEAL_C)),
                )?)
                .with_render_mode(surface_render_mode_from_name(render_mode)?),
        })
    }

    fn with_write_progress(mut slf: PyRefMut<'_, Self>, progress: f32) -> PyResult<PyRefMut<'_, Self>> {
        if !(0.0..=1.0).contains(&progress) || !progress.is_finite() {
            return Err(PyValueError::new_err(
                "progress must be a finite number between 0 and 1",
            ));
        }
        slf.inner = slf.inner.clone().with_write_progress(progress);
        Ok(slf)
    }

    fn with_color(mut slf: PyRefMut<'_, Self>, color: ColorTuple) -> PyResult<PyRefMut<'_, Self>> {
        slf.inner = slf.inner.clone().with_color(color_from_tuple(color)?);
        Ok(slf)
    }

    fn with_texture<'py>(mut slf: PyRefMut<'py, Self>, name: &str) -> PyResult<PyRefMut<'py, Self>> {
        let mut inner = slf.inner.clone();
        inner.texture = Some(TextureImage::builtin_shared(builtin_texture_from_name(name)?));
        slf.inner = inner;
        Ok(slf)
    }

    fn with_texture_flip_x(mut slf: PyRefMut<'_, Self>, flip: bool) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().with_texture_flip_x(flip);
        slf
    }

    fn with_texture_flip_y(mut slf: PyRefMut<'_, Self>, flip: bool) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().with_texture_flip_y(flip);
        slf
    }
}

fn map_projection_kind_from_name(name: &str) -> PyResult<crate::math::map_projection::MapProjectionKind> {
    crate::math::map_projection::MapProjectionKind::from_name(name).ok_or_else(|| {
        PyValueError::new_err(format!(
            "unknown map projection {name:?}; expected equirectangular, sinusoidal, mollweide, hammer, or mercator"
        ))
    })
}

fn surface_from_python_function(
    function: &Bound<'_, PyAny>,
    u_range: (f32, f32),
    v_range: (f32, f32),
    samples: (usize, usize),
) -> PyResult<ParametricSurface> {
    if !function.is_callable() {
        return Err(PyValueError::new_err("function must be callable"));
    }
    if u_range.0 >= u_range.1 || v_range.0 >= v_range.1 {
        return Err(PyValueError::new_err(
            "range starts must be less than range ends",
        ));
    }
    let (u_samples, v_samples) = samples;
    if u_samples < 2 || v_samples < 2 {
        return Err(PyValueError::new_err(
            "samples must be at least 2 in each direction",
        ));
    }
    let u_step = (u_range.1 - u_range.0) / (u_samples - 1) as f32;
    let v_step = (v_range.1 - v_range.0) / (v_samples - 1) as f32;
    let mut points = Vec::with_capacity(u_samples * v_samples);
    for i in 0..u_samples {
        for j in 0..v_samples {
            let u = u_range.0 + i as f32 * u_step;
            let v = v_range.0 + j as f32 * v_step;
            let value = function.call1((u, v))?;
            let tuple: Vec3Tuple = value.extract().map_err(|_| {
                PyValueError::new_err("surface function must return an (x, y, z) tuple")
            })?;
            points.push(vec3_from_tuple(Some(tuple))?);
        }
    }
    let points = Arc::new(points);
    Ok(
        ParametricSurface::new(u_range, v_range, move |u, v| {
            let i = ((u - u_range.0) / u_step).round() as usize;
            let j = ((v - v_range.0) / v_step).round() as usize;
            let i = i.min(u_samples.saturating_sub(1));
            let j = j.min(v_samples.saturating_sub(1));
            points[i * v_samples + j]
        })
        .with_samples(u_samples, v_samples),
    )
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

    #[staticmethod]
    fn from_glb(path: String) -> PyResult<Self> {
        let inner =
            Prop3D::from_glb(path).map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
        Ok(Self { inner })
    }

    #[staticmethod]
    fn from_gltf(path: String) -> PyResult<Self> {
        let inner =
            Prop3D::from_gltf(path).map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
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

    fn center(&self) -> Vec3Tuple {
        let p = self.inner.center();
        (p.x, p.y, p.z)
    }

    fn dimensions(&self) -> Vec3Tuple {
        let p = self.inner.dimensions();
        (p.x, p.y, p.z)
    }
}
