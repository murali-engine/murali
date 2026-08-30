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

    fn with_stroke(mut slf: PyRefMut<'_, Self>, thickness: f32, color: ColorTuple) -> PyResult<PyRefMut<'_, Self>> {
        if thickness <= 0.0 || !thickness.is_finite() {
            return Err(PyValueError::new_err(
                "stroke thickness must be a positive finite number",
            ));
        }
        slf.inner = slf.inner
            .clone()
            .with_stroke(thickness, color_from_tuple(color)?);
        Ok(slf)
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

    fn with_stroke(mut slf: PyRefMut<'_, Self>, thickness: f32, color: ColorTuple) -> PyResult<PyRefMut<'_, Self>> {
        if thickness <= 0.0 || !thickness.is_finite() {
            return Err(PyValueError::new_err(
                "stroke thickness must be a positive finite number",
            ));
        }
        slf.inner = slf.inner
            .clone()
            .with_stroke(thickness, color_from_tuple(color)?);
        Ok(slf)
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

    fn with_stroke(mut slf: PyRefMut<'_, Self>, thickness: f32, color: ColorTuple) -> PyResult<PyRefMut<'_, Self>> {
        if thickness <= 0.0 || !thickness.is_finite() {
            return Err(PyValueError::new_err(
                "stroke thickness must be a positive finite number",
            ));
        }
        slf.inner = slf.inner
            .clone()
            .with_stroke(thickness, color_from_tuple(color)?);
        Ok(slf)
    }
}

#[pyclass(name = "RoundedRectangle", module = "murali_engine")]
#[derive(Clone, Debug)]
struct PyRoundedRectangle {
    inner: RoundedRectangle,
}

#[pymethods]
impl PyRoundedRectangle {
    #[new]
    #[pyo3(signature = (width = 1.0, height = 1.0, radius = 0.18, color = None))]
    fn new(width: f32, height: f32, radius: f32, color: Option<ColorTuple>) -> PyResult<Self> {
        if width <= 0.0 || height <= 0.0 || !width.is_finite() || !height.is_finite() {
            return Err(PyValueError::new_err(
                "width and height must be positive finite numbers",
            ));
        }
        if radius < 0.0 || !radius.is_finite() {
            return Err(PyValueError::new_err(
                "radius must be a non-negative finite number",
            ));
        }
        Ok(Self {
            inner: RoundedRectangle::new(
                width,
                height,
                radius,
                color_from_tuple(color.unwrap_or(color_tuple(colors::WHITE)))?,
            ),
        })
    }

    fn with_stroke(mut slf: PyRefMut<'_, Self>, thickness: f32, color: ColorTuple) -> PyResult<PyRefMut<'_, Self>> {
        if thickness <= 0.0 || !thickness.is_finite() {
            return Err(PyValueError::new_err(
                "stroke thickness must be a positive finite number",
            ));
        }
        slf.inner = slf.inner
            .clone()
            .with_stroke(thickness, color_from_tuple(color)?);
        Ok(slf)
    }

    fn with_corner_segments(mut slf: PyRefMut<'_, Self>, corner_segments: usize) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().with_corner_segments(corner_segments);
        slf
    }
}

#[pyclass(name = "ChatBubble", module = "murali_engine")]
#[derive(Clone, Debug)]
struct PyChatBubble {
    inner: ChatBubble,
}

#[pymethods]
impl PyChatBubble {
    #[new]
    #[pyo3(signature = (width = 5.8, height = 0.82, radius = 0.18, color = None))]
    fn new(width: f32, height: f32, radius: f32, color: Option<ColorTuple>) -> PyResult<Self> {
        if width <= 0.0 || height <= 0.0 || !width.is_finite() || !height.is_finite() {
            return Err(PyValueError::new_err(
                "width and height must be positive finite numbers",
            ));
        }
        if radius < 0.0 || !radius.is_finite() {
            return Err(PyValueError::new_err(
                "radius must be a non-negative finite number",
            ));
        }
        Ok(Self {
            inner: ChatBubble::new(
                width,
                height,
                radius,
                color_from_tuple(color.unwrap_or(color_tuple(colors::WHITE)))?,
            ),
        })
    }

    fn with_stroke(mut slf: PyRefMut<'_, Self>, thickness: f32, color: ColorTuple) -> PyResult<PyRefMut<'_, Self>> {
        if thickness <= 0.0 || !thickness.is_finite() {
            return Err(PyValueError::new_err(
                "stroke thickness must be a positive finite number",
            ));
        }
        slf.inner = slf.inner
            .clone()
            .with_stroke(thickness, color_from_tuple(color)?);
        Ok(slf)
    }

    fn with_tip<'py>(mut slf: PyRefMut<'py, Self>, side: &str, width: f32, height: f32) -> PyResult<PyRefMut<'py, Self>> {
        if width < 0.0 || height < 0.0 || !width.is_finite() || !height.is_finite() {
            return Err(PyValueError::new_err(
                "tip width and height must be non-negative finite numbers",
            ));
        }
        slf.inner =
            slf.inner
                .clone()
                .with_tip(chat_bubble_tip_side_from_name(side)?, width, height);
        Ok(slf)
    }

    fn with_tip_inset(mut slf: PyRefMut<'_, Self>, inset: f32) -> PyResult<PyRefMut<'_, Self>> {
        if inset < 0.0 || !inset.is_finite() {
            return Err(PyValueError::new_err(
                "tip inset must be a non-negative finite number",
            ));
        }
        slf.inner = slf.inner.clone().with_tip_inset(inset);
        Ok(slf)
    }

    fn with_corner_segments(mut slf: PyRefMut<'_, Self>, corner_segments: usize) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().with_corner_segments(corner_segments);
        slf
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

    fn with_stroke(mut slf: PyRefMut<'_, Self>, thickness: f32, color: ColorTuple) -> PyResult<PyRefMut<'_, Self>> {
        if thickness <= 0.0 || !thickness.is_finite() {
            return Err(PyValueError::new_err(
                "stroke thickness must be a positive finite number",
            ));
        }
        slf.inner = slf.inner
            .clone()
            .with_stroke(thickness, color_from_tuple(color)?);
        Ok(slf)
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

    fn with_dash(mut slf: PyRefMut<'_, Self>, dash: f32, gap: f32) -> PyResult<PyRefMut<'_, Self>> {
        if dash < 0.0 || gap < 0.0 || !dash.is_finite() || !gap.is_finite() {
            return Err(PyValueError::new_err(
                "dash and gap must be non-negative finite numbers",
            ));
        }
        slf.inner = slf.inner.clone().with_dash(dash, gap);
        Ok(slf)
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
        let inner = std::mem::replace(&mut self.inner, Path::new());
        self.inner = inner.move_to(vec2_from_tuple(point)?);
        Ok(())
    }

    fn line_to(&mut self, point: Vec2Tuple) -> PyResult<()> {
        let inner = std::mem::replace(&mut self.inner, Path::new());
        self.inner = inner.line_to(vec2_from_tuple(point)?);
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

    fn with_color(mut slf: PyRefMut<'_, Self>, color: ColorTuple) -> PyResult<PyRefMut<'_, Self>> {
        slf.inner = slf.inner.clone().with_color(color_from_tuple(color)?);
        Ok(slf)
    }

    fn with_thickness(mut slf: PyRefMut<'_, Self>, thickness: f32) -> PyResult<PyRefMut<'_, Self>> {
        if thickness <= 0.0 || !thickness.is_finite() {
            return Err(PyValueError::new_err(
                "thickness must be a positive finite number",
            ));
        }
        slf.inner = slf.inner.clone().with_thickness(thickness);
        Ok(slf)
    }

    fn with_dash(mut slf: PyRefMut<'_, Self>, dash: f32, gap: f32) -> PyResult<PyRefMut<'_, Self>> {
        if dash < 0.0 || gap < 0.0 || !dash.is_finite() || !gap.is_finite() {
            return Err(PyValueError::new_err(
                "dash and gap must be non-negative finite numbers",
            ));
        }
        slf.inner = slf.inner.clone().with_dash(dash, gap);
        Ok(slf)
    }

    #[staticmethod]
    #[pyo3(signature = (points, color = None, thickness = 0.02))]
    fn from_points(
        points: Vec<Vec2Tuple>,
        color: Option<ColorTuple>,
        thickness: f32,
    ) -> PyResult<Self> {
        if points.len() < 2 {
            return Err(PyValueError::new_err(
                "Path.from_points requires at least two points",
            ));
        }
        if thickness <= 0.0 || !thickness.is_finite() {
            return Err(PyValueError::new_err(
                "thickness must be a positive finite number",
            ));
        }
        let mut path = Path::new()
            .with_color(color_from_tuple(
                color.unwrap_or(color_tuple(colors::WHITE)),
            )?)
            .with_thickness(thickness)
            .move_to(vec2_from_tuple(points[0])?);
        for point in points.into_iter().skip(1) {
            path = path.line_to(vec2_from_tuple(point)?);
        }
        Ok(Self { inner: path })
    }

    #[staticmethod]
    #[pyo3(signature = (polylines, color = None, thickness = 0.02))]
    fn from_polylines(
        polylines: Vec<Vec<Vec2Tuple>>,
        color: Option<ColorTuple>,
        thickness: f32,
    ) -> PyResult<Self> {
        if thickness <= 0.0 || !thickness.is_finite() {
            return Err(PyValueError::new_err(
                "thickness must be a positive finite number",
            ));
        }
        let mut path = Path::new()
            .with_color(color_from_tuple(
                color.unwrap_or(color_tuple(colors::WHITE)),
            )?)
            .with_thickness(thickness);
        for polyline in polylines {
            if polyline.len() < 2 {
                return Err(PyValueError::new_err(
                    "each polyline in Path.from_polylines requires at least two points",
                ));
            }
            path = path.move_to(vec2_from_tuple(polyline[0])?);
            for point in polyline.into_iter().skip(1) {
                path = path.line_to(vec2_from_tuple(point)?);
            }
        }
        Ok(Self { inner: path })
    }

    #[staticmethod]
    #[pyo3(signature = (source, target = None, mix = 0.0, half_size = (5.9, 3.5), color = None, thickness = 0.02))]
    fn from_map_graticule(
        source: &str,
        target: Option<&str>,
        mix: f32,
        half_size: (f32, f32),
        color: Option<ColorTuple>,
        thickness: f32,
    ) -> PyResult<Self> {
        if thickness <= 0.0 || !thickness.is_finite() {
            return Err(PyValueError::new_err(
                "thickness must be a positive finite number",
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
        let source = map_projection_kind_from_name(source)?;
        let target = match target {
            Some(name) => map_projection_kind_from_name(name)?,
            None => source,
        };
        Ok(Self {
            inner: crate::math::map_projection::graticule_path(
                source,
                target,
                mix,
                half_size.0,
                half_size.1,
            )
            .with_color(color_from_tuple(color.unwrap_or((
                0.78, 0.92, 1.0, 0.22,
            )))?)
            .with_thickness(thickness),
        })
    }
}

#[pyclass(name = "TracedPath", module = "murali_engine")]
struct PyTracedPath {
    inner: Option<TracedPath>,
}

impl PyTracedPath {
    fn inner_mut(&mut self) -> PyResult<&mut TracedPath> {
        self.inner.as_mut().ok_or_else(|| {
            PyRuntimeError::new_err("this TracedPath has already been added to a scene")
        })
    }

    fn take_inner(&mut self) -> PyResult<TracedPath> {
        self.inner.take().ok_or_else(|| {
            PyRuntimeError::new_err("this TracedPath has already been added to a scene")
        })
    }
}

#[pymethods]
impl PyTracedPath {
    #[new]
    #[pyo3(signature = (tracked, color = None, thickness = 0.04))]
    fn new(tracked: &PyTattvaHandle, color: Option<ColorTuple>, thickness: f32) -> PyResult<Self> {
        if thickness <= 0.0 || !thickness.is_finite() {
            return Err(PyValueError::new_err(
                "thickness must be a positive finite number",
            ));
        }
        Ok(Self {
            inner: Some(TracedPath::new(
                tracked.id,
                |position, _rotation| position,
                color_from_tuple(color.unwrap_or(color_tuple(colors::WHITE)))?,
                thickness,
            )),
        })
    }

    fn with_min_distance(mut slf: PyRefMut<'_, Self>, min_distance: f32) -> PyResult<PyRefMut<'_, Self>> {
        if min_distance < 0.0 || !min_distance.is_finite() {
            return Err(PyValueError::new_err(
                "min_distance must be a non-negative finite number",
            ));
        }
        slf.inner_mut()?.min_distance = min_distance;
        Ok(slf)
    }

    fn with_max_points(mut slf: PyRefMut<'_, Self>, max_points: usize) -> PyResult<PyRefMut<'_, Self>> {
        if max_points == 0 {
            return Err(PyValueError::new_err(
                "max_points must be greater than zero",
            ));
        }
        slf.inner_mut()?.max_points = max_points;
        Ok(slf)
    }
}

#[pyclass(name = "ParticleBelt", module = "murali_engine")]
#[derive(Clone, Debug)]
struct PyParticleBelt {
    inner: ParticleBelt,
}

#[pymethods]
impl PyParticleBelt {
    #[new]
    fn new(radius: f32) -> PyResult<Self> {
        if radius <= 0.0 || !radius.is_finite() {
            return Err(PyValueError::new_err(
                "radius must be a positive finite number",
            ));
        }
        Ok(Self {
            inner: ParticleBelt::new(radius),
        })
    }

    fn with_band_width(mut slf: PyRefMut<'_, Self>, band_width: f32) -> PyResult<PyRefMut<'_, Self>> {
        if band_width < 0.0 || !band_width.is_finite() {
            return Err(PyValueError::new_err(
                "band_width must be a non-negative finite number",
            ));
        }
        slf.inner = slf.inner.clone().with_band_width(band_width);
        Ok(slf)
    }

    fn with_particle_count(mut slf: PyRefMut<'_, Self>, particle_count: usize) -> PyResult<PyRefMut<'_, Self>> {
        if particle_count == 0 {
            return Err(PyValueError::new_err(
                "particle_count must be greater than zero",
            ));
        }
        slf.inner = slf.inner.clone().with_particle_count(particle_count);
        Ok(slf)
    }

    fn with_particle_size_range(mut slf: PyRefMut<'_, Self>, min_radius: f32, max_radius: f32) -> PyResult<PyRefMut<'_, Self>> {
        if min_radius <= 0.0 || !min_radius.is_finite() {
            return Err(PyValueError::new_err(
                "min_radius must be a positive finite number",
            ));
        }
        if max_radius < min_radius || !max_radius.is_finite() {
            return Err(PyValueError::new_err(
                "max_radius must be a finite number at least min_radius",
            ));
        }
        slf.inner = slf.inner
            .clone()
            .with_particle_size_range(min_radius, max_radius);
        Ok(slf)
    }

    fn with_palette(mut slf: PyRefMut<'_, Self>, palette: Vec<ColorTuple>) -> PyResult<PyRefMut<'_, Self>> {
        if palette.is_empty() {
            return Err(PyValueError::new_err("palette must contain at least one color"));
        }
        let colors = palette
            .into_iter()
            .map(color_from_tuple)
            .collect::<PyResult<Vec<_>>>()?;
        slf.inner = slf.inner.clone().with_palette(colors);
        Ok(slf)
    }

    fn with_phase(mut slf: PyRefMut<'_, Self>, phase: f32) -> PyResult<PyRefMut<'_, Self>> {
        if !phase.is_finite() {
            return Err(PyValueError::new_err("phase must be a finite number"));
        }
        slf.inner = slf.inner.clone().with_phase(phase);
        Ok(slf)
    }

    fn with_orbit_speed(mut slf: PyRefMut<'_, Self>, orbit_speed: f32) -> PyResult<PyRefMut<'_, Self>> {
        if !orbit_speed.is_finite() {
            return Err(PyValueError::new_err("orbit_speed must be a finite number"));
        }
        slf.inner = slf.inner.clone().with_orbit_speed(orbit_speed);
        Ok(slf)
    }

    fn with_clockwise_ratio(mut slf: PyRefMut<'_, Self>, clockwise_ratio: f32) -> PyResult<PyRefMut<'_, Self>> {
        if !clockwise_ratio.is_finite() {
            return Err(PyValueError::new_err(
                "clockwise_ratio must be a finite number",
            ));
        }
        slf.inner = slf.inner.clone().with_clockwise_ratio(clockwise_ratio);
        Ok(slf)
    }

    fn all_clockwise(&mut self) {
        self.inner = self.inner.clone().all_clockwise();
    }

    fn all_anticlockwise(&mut self) {
        self.inner = self.inner.clone().all_anticlockwise();
    }

    fn with_band_breathing(mut slf: PyRefMut<'_, Self>, amplitude: f32, rate: f32) -> PyResult<PyRefMut<'_, Self>> {
        if amplitude < 0.0 || !amplitude.is_finite() {
            return Err(PyValueError::new_err(
                "amplitude must be a non-negative finite number",
            ));
        }
        if !rate.is_finite() {
            return Err(PyValueError::new_err("rate must be a finite number"));
        }
        slf.inner = slf.inner.clone().with_band_breathing(amplitude, rate);
        Ok(slf)
    }

    fn with_radial_jitter(mut slf: PyRefMut<'_, Self>, amplitude: f32, rate: f32) -> PyResult<PyRefMut<'_, Self>> {
        if amplitude < 0.0 || !amplitude.is_finite() {
            return Err(PyValueError::new_err(
                "amplitude must be a non-negative finite number",
            ));
        }
        if !rate.is_finite() {
            return Err(PyValueError::new_err("rate must be a finite number"));
        }
        slf.inner = slf.inner.clone().with_radial_jitter(amplitude, rate);
        Ok(slf)
    }

    fn with_seed(mut slf: PyRefMut<'_, Self>, seed: f32) -> PyResult<PyRefMut<'_, Self>> {
        if !seed.is_finite() {
            return Err(PyValueError::new_err("seed must be a finite number"));
        }
        slf.inner = slf.inner.clone().with_seed(seed);
        Ok(slf)
    }

    fn with_angular_spread(mut slf: PyRefMut<'_, Self>, angular_spread: f32) -> PyResult<PyRefMut<'_, Self>> {
        if angular_spread < 0.0 || !angular_spread.is_finite() {
            return Err(PyValueError::new_err(
                "angular_spread must be a non-negative finite number",
            ));
        }
        slf.inner = slf.inner.clone().with_angular_spread(angular_spread);
        Ok(slf)
    }

    fn particle_count(&self) -> usize {
        self.inner.particle_count
    }

    fn radius(&self) -> f32 {
        self.inner.radius
    }
}
