#[pyclass(name = "EquationPart", module = "murali_engine")]
#[derive(Clone, Debug)]
struct PyEquationPart {
    inner: EquationPart,
}

#[pymethods]
impl PyEquationPart {
    #[new]
    #[pyo3(signature = (text, key = None, color = None))]
    fn new(text: String, key: Option<String>, color: Option<ColorTuple>) -> PyResult<Self> {
        let mut inner = EquationPart::new(text);
        if let Some(key) = key {
            inner = inner.with_key(key);
        }
        if let Some(color) = color {
            inner = inner.with_color(color_from_tuple(color)?);
        }
        Ok(Self { inner })
    }

    fn with_key(mut slf: PyRefMut<'_, Self>, key: String) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().with_key(key);
        slf
    }

    fn with_color(mut slf: PyRefMut<'_, Self>, color: ColorTuple) -> PyResult<PyRefMut<'_, Self>> {
        slf.inner = slf.inner.clone().with_color(color_from_tuple(color)?);
        Ok(slf)
    }

    fn with_opacity(mut slf: PyRefMut<'_, Self>, opacity: f32) -> PyResult<PyRefMut<'_, Self>> {
        if !(0.0..=1.0).contains(&opacity) || !opacity.is_finite() {
            return Err(PyValueError::new_err(
                "opacity must be a finite number between 0 and 1",
            ));
        }
        slf.inner = slf.inner.clone().with_opacity(opacity);
        Ok(slf)
    }

    fn with_scale(mut slf: PyRefMut<'_, Self>, scale: f32) -> PyResult<PyRefMut<'_, Self>> {
        if scale <= 0.0 || !scale.is_finite() {
            return Err(PyValueError::new_err(
                "scale must be a positive finite number",
            ));
        }
        slf.inner = slf.inner.clone().with_scale(scale);
        Ok(slf)
    }

    fn with_offset(mut slf: PyRefMut<'_, Self>, offset: Vec3Tuple) -> PyResult<PyRefMut<'_, Self>> {
        slf.inner = slf.inner
            .clone()
            .with_offset(vec3_from_tuple(Some(offset))?);
        Ok(slf)
    }
}

#[pyclass(name = "EquationLayout", module = "murali_engine")]
#[derive(Clone, Debug)]
struct PyEquationLayout {
    inner: EquationLayout,
}

#[pymethods]
impl PyEquationLayout {
    #[new]
    fn new(parts: Vec<PyRef<'_, PyEquationPart>>, height: f32) -> PyResult<Self> {
        if parts.is_empty() {
            return Err(PyValueError::new_err(
                "EquationLayout requires at least one part",
            ));
        }
        if height <= 0.0 || !height.is_finite() {
            return Err(PyValueError::new_err(
                "height must be a positive finite number",
            ));
        }
        Ok(Self {
            inner: EquationLayout::new(
                parts
                    .into_iter()
                    .map(|part| part.inner.clone())
                    .collect::<Vec<_>>(),
                height,
            ),
        })
    }

    fn with_gap(mut slf: PyRefMut<'_, Self>, gap: f32) -> PyResult<PyRefMut<'_, Self>> {
        if gap < 0.0 || !gap.is_finite() {
            return Err(PyValueError::new_err(
                "gap must be a non-negative finite number",
            ));
        }
        slf.inner.gap = gap;
        Ok(slf)
    }
}

#[pyclass(name = "Matrix", module = "murali_engine")]
#[derive(Clone, Debug)]
struct PyMatrix {
    inner: Matrix,
}

#[pymethods]
impl PyMatrix {
    #[new]
    fn new(entries: Vec<Vec<String>>, cell_height: f32) -> PyResult<Self> {
        if entries.is_empty() || entries.iter().any(Vec::is_empty) {
            return Err(PyValueError::new_err(
                "Matrix requires at least one row and one column",
            ));
        }
        let cols = entries[0].len();
        if entries.iter().any(|row| row.len() != cols) {
            return Err(PyValueError::new_err(
                "Matrix rows must all have the same length",
            ));
        }
        if cell_height <= 0.0 || !cell_height.is_finite() {
            return Err(PyValueError::new_err(
                "cell_height must be a positive finite number",
            ));
        }
        Ok(Self {
            inner: Matrix::new(entries, cell_height),
        })
    }

    fn with_color(mut slf: PyRefMut<'_, Self>, color: ColorTuple) -> PyResult<PyRefMut<'_, Self>> {
        slf.inner.color = color_from_tuple(color)?;
        Ok(slf)
    }

    fn with_bracket_color(mut slf: PyRefMut<'_, Self>, color: ColorTuple) -> PyResult<PyRefMut<'_, Self>> {
        slf.inner.bracket_color = color_from_tuple(color)?;
        Ok(slf)
    }

    fn with_bracket_thickness(mut slf: PyRefMut<'_, Self>, thickness: f32) -> PyResult<PyRefMut<'_, Self>> {
        if thickness <= 0.0 || !thickness.is_finite() {
            return Err(PyValueError::new_err(
                "thickness must be a positive finite number",
            ));
        }
        slf.inner.bracket_thickness = thickness;
        Ok(slf)
    }

    fn set_cell_color(&mut self, row: usize, col: usize, color: ColorTuple) -> PyResult<()> {
        let color = color_from_tuple(color)?;
        let cell = self.inner.cell_mut(row, col).ok_or_else(|| {
            PyValueError::new_err(format!("matrix cell ({row}, {col}) is out of range"))
        })?;
        cell.color = color;
        Ok(())
    }

    fn set_cell_highlight(&mut self, row: usize, col: usize, color: ColorTuple) -> PyResult<()> {
        let color = color_from_tuple(color)?;
        let cell = self.inner.cell_mut(row, col).ok_or_else(|| {
            PyValueError::new_err(format!("matrix cell ({row}, {col}) is out of range"))
        })?;
        *cell = cell.clone().with_highlight(color);
        Ok(())
    }
}

#[pyclass(name = "NumberLine", module = "murali_engine")]
#[derive(Clone, Debug)]
struct PyNumberLine {
    inner: NumberLine,
}

#[pymethods]
impl PyNumberLine {
    #[new]
    fn new(range: (f32, f32)) -> PyResult<Self> {
        if range.0 >= range.1 || !range.0.is_finite() || !range.1.is_finite() {
            return Err(PyValueError::new_err(
                "range start must be less than range end and both values must be finite",
            ));
        }
        Ok(Self {
            inner: NumberLine::new(range),
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

    fn with_color(mut slf: PyRefMut<'_, Self>, color: ColorTuple) -> PyResult<PyRefMut<'_, Self>> {
        slf.inner = slf.inner.clone().with_color(color_from_tuple(color)?);
        Ok(slf)
    }

    fn with_origin_color(mut slf: PyRefMut<'_, Self>, color: ColorTuple) -> PyResult<PyRefMut<'_, Self>> {
        slf.inner = slf.inner
            .clone()
            .with_origin_color(color_from_tuple(color)?);
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

    fn without_origin_emphasis(mut slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().without_origin_emphasis();
        slf
    }
}

#[pyclass(name = "OptimizationPath2D", module = "murali_engine")]
#[derive(Clone, Debug)]
struct PyOptimizationPath2D {
    inner: OptimizationPath2D,
}

#[pymethods]
impl PyOptimizationPath2D {
    #[new]
    fn new(points: Vec<Vec2Tuple>) -> PyResult<Self> {
        if points.len() < 2 {
            return Err(PyValueError::new_err(
                "OptimizationPath2D requires at least two points",
            ));
        }
        Ok(Self {
            inner: OptimizationPath2D::new(
                points
                    .into_iter()
                    .map(vec2_from_tuple)
                    .collect::<PyResult<Vec<_>>>()?,
            ),
        })
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

    fn with_point_size(mut slf: PyRefMut<'_, Self>, point_size: f32) -> PyResult<PyRefMut<'_, Self>> {
        if point_size < 0.0 || !point_size.is_finite() {
            return Err(PyValueError::new_err(
                "point_size must be a non-negative finite number",
            ));
        }
        slf.inner = slf.inner.clone().with_point_size(point_size);
        Ok(slf)
    }

    fn without_points(mut slf: PyRefMut<'_, Self>) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().without_points();
        slf
    }

    fn steps(&self) -> usize {
        self.inner.steps()
    }
}
