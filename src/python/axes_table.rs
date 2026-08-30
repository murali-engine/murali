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

    fn with_step(mut slf: PyRefMut<'_, Self>, step: f32) -> PyResult<PyRefMut<'_, Self>> {
        if step <= 0.0 || !step.is_finite() {
            return Err(PyValueError::new_err(
                "step must be a positive finite number",
            ));
        }
        slf.inner = slf.inner.clone().with_step(step);
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

    fn with_step(mut slf: PyRefMut<'_, Self>, step: f32) -> PyResult<PyRefMut<'_, Self>> {
        if step <= 0.0 || !step.is_finite() {
            return Err(PyValueError::new_err(
                "step must be a positive finite number",
            ));
        }
        slf.inner = slf.inner.clone().with_step(step);
        Ok(slf)
    }

    fn with_grid_color(mut slf: PyRefMut<'_, Self>, color: ColorTuple) -> PyResult<PyRefMut<'_, Self>> {
        slf.inner.grid_color = color_from_tuple(color)?;
        Ok(slf)
    }

    fn with_axis_color(mut slf: PyRefMut<'_, Self>, color: ColorTuple) -> PyResult<PyRefMut<'_, Self>> {
        slf.inner.axis_color = color_from_tuple(color)?;
        Ok(slf)
    }

    fn with_grid_thickness(mut slf: PyRefMut<'_, Self>, thickness: f32) -> PyResult<PyRefMut<'_, Self>> {
        if thickness <= 0.0 || !thickness.is_finite() {
            return Err(PyValueError::new_err(
                "grid thickness must be a positive finite number",
            ));
        }
        slf.inner.grid_thickness = thickness;
        Ok(slf)
    }

    fn with_axis_thickness(mut slf: PyRefMut<'_, Self>, thickness: f32) -> PyResult<PyRefMut<'_, Self>> {
        if thickness <= 0.0 || !thickness.is_finite() {
            return Err(PyValueError::new_err(
                "axis thickness must be a positive finite number",
            ));
        }
        slf.inner.axis_thickness = thickness;
        Ok(slf)
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

    fn with_row_labels(mut slf: PyRefMut<'_, Self>, labels: Vec<String>) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().with_row_labels(labels);
        slf
    }

    fn with_col_labels(mut slf: PyRefMut<'_, Self>, labels: Vec<String>) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().with_col_labels(labels);
        slf
    }

    fn with_title(mut slf: PyRefMut<'_, Self>, title: String) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().with_title(title);
        slf
    }

    fn with_line_color(mut slf: PyRefMut<'_, Self>, color: ColorTuple) -> PyResult<PyRefMut<'_, Self>> {
        slf.inner = slf.inner.clone().with_line_color(color_from_tuple(color)?);
        Ok(slf)
    }

    fn with_text_color(mut slf: PyRefMut<'_, Self>, color: ColorTuple) -> PyResult<PyRefMut<'_, Self>> {
        slf.inner = slf.inner.clone().with_text_color(color_from_tuple(color)?);
        Ok(slf)
    }

    fn with_text_height(mut slf: PyRefMut<'_, Self>, height: f32) -> PyResult<PyRefMut<'_, Self>> {
        if height <= 0.0 || !height.is_finite() {
            return Err(PyValueError::new_err(
                "height must be a positive finite number",
            ));
        }
        slf.inner = slf.inner.clone().with_text_height(height);
        Ok(slf)
    }

    fn with_background_color(mut slf: PyRefMut<'_, Self>, color: ColorTuple) -> PyResult<PyRefMut<'_, Self>> {
        slf.inner = slf.inner
            .clone()
            .with_background_color(color_from_tuple(color)?);
        Ok(slf)
    }

    fn with_labels_inside(mut slf: PyRefMut<'_, Self>, inside: bool) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().with_labels_inside(inside);
        slf
    }

    fn num_rows(&self) -> usize {
        self.inner.num_rows()
    }

    fn num_cols(&self) -> usize {
        self.inner.num_cols()
    }
}
