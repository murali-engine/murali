#[pyclass(name = "ContextBlock", module = "murali_engine")]
#[derive(Clone, Debug)]
struct PyContextBlock {
    inner: ContextBlock,
}

#[pymethods]
impl PyContextBlock {
    #[new]
    #[pyo3(signature = (id, label, role, token_count, preview = None))]
    fn new(
        id: String,
        label: String,
        role: &str,
        token_count: usize,
        preview: Option<String>,
    ) -> PyResult<Self> {
        if token_count == 0 {
            return Err(PyValueError::new_err("token_count must be greater than zero"));
        }
        let mut inner = ContextBlock::new(
            id,
            label,
            context_block_role_from_name(role)?,
            token_count,
        );
        if let Some(preview) = preview {
            inner = inner.with_preview(preview);
        }
        Ok(Self { inner })
    }

    fn with_preview(mut slf: PyRefMut<'_, Self>, preview: String) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().with_preview(preview);
        slf
    }

    fn truncated_to<'py>(mut slf: PyRefMut<'py, Self>, retained_tokens: usize, truncation: &str) -> PyResult<PyRefMut<'py, Self>> {
        slf.inner = slf.inner
            .clone()
            .truncated_to(retained_tokens, context_truncation_from_name(truncation)?);
        Ok(slf)
    }

    fn omitted_tokens(&self) -> usize {
        self.inner.omitted_tokens()
    }
}

#[pyclass(name = "ContextWindow", module = "murali_engine")]
#[derive(Clone, Debug)]
struct PyContextWindow {
    inner: ContextWindow,
}

#[pymethods]
impl PyContextWindow {
    #[new]
    #[pyo3(signature = (blocks, token_budget, title = None))]
    fn new(
        blocks: Vec<PyRef<'_, PyContextBlock>>,
        token_budget: usize,
        title: Option<String>,
    ) -> PyResult<Self> {
        let blocks = blocks
            .into_iter()
            .map(|block| block.inner.clone())
            .collect::<Vec<_>>();
        let mut inner = ContextWindow::try_new(blocks, token_budget)
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        if let Some(title) = title {
            inner = inner.with_title(title);
        }
        Ok(Self { inner })
    }

    fn with_title(mut slf: PyRefMut<'_, Self>, title: String) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().with_title(title);
        slf
    }

    fn used_tokens(&self) -> usize {
        self.inner.used_tokens()
    }

    fn available_tokens(&self) -> usize {
        self.inner.available_tokens()
    }
}

#[pyclass(name = "SignalFlow", module = "murali_engine")]
#[derive(Clone, Debug)]
struct PySignalFlow {
    inner: SignalFlow,
}

fn vec3_tuple(point: Vec3) -> Vec3Tuple {
    (point.x, point.y, point.z)
}

#[pymethods]
impl PySignalFlow {
    #[new]
    fn new(path_points: Vec<Vec3Tuple>) -> PyResult<Self> {
        let points = path_points
            .into_iter()
            .map(|point| vec3_from_tuple(Some(point)))
            .collect::<PyResult<Vec<_>>>()?;
        if points.len() < 2 {
            return Err(PyValueError::new_err(
                "SignalFlow requires at least two path points",
            ));
        }
        Ok(Self {
            inner: SignalFlow::new(points),
        })
    }

    #[staticmethod]
    fn from_paths(paths: Vec<Vec<Vec3Tuple>>) -> PyResult<Self> {
        let paths = paths
            .into_iter()
            .map(|path| {
                let points = path
                    .into_iter()
                    .map(|point| vec3_from_tuple(Some(point)))
                    .collect::<PyResult<Vec<_>>>()?;
                if points.len() < 2 {
                    return Err(PyValueError::new_err(
                        "SignalFlow paths must each contain at least two points",
                    ));
                }
                Ok(points)
            })
            .collect::<PyResult<Vec<_>>>()?;
        if paths.is_empty() {
            return Err(PyValueError::new_err(
                "SignalFlow.from_paths requires at least one path",
            ));
        }
        Ok(Self {
            inner: SignalFlow::from_paths(paths),
        })
    }

    fn with_progress(mut slf: PyRefMut<'_, Self>, progress: f32) -> PyResult<PyRefMut<'_, Self>> {
        if !(0.0..=1.0).contains(&progress) || !progress.is_finite() {
            return Err(PyValueError::new_err(
                "progress must be a finite number between 0 and 1",
            ));
        }
        slf.inner = slf.inner.clone().with_progress(progress);
        Ok(slf)
    }

    fn with_edge_color(mut slf: PyRefMut<'_, Self>, color: ColorTuple) -> PyResult<PyRefMut<'_, Self>> {
        slf.inner = slf.inner.clone().with_edge_color(color_from_tuple(color)?);
        Ok(slf)
    }

    fn with_pulse_color(mut slf: PyRefMut<'_, Self>, color: ColorTuple) -> PyResult<PyRefMut<'_, Self>> {
        slf.inner = slf.inner.clone().with_pulse_color(color_from_tuple(color)?);
        Ok(slf)
    }

    fn with_node_color(mut slf: PyRefMut<'_, Self>, color: ColorTuple) -> PyResult<PyRefMut<'_, Self>> {
        slf.inner.node_color = color_from_tuple(color)?;
        Ok(slf)
    }

    fn with_edge_thickness(mut slf: PyRefMut<'_, Self>, thickness: f32) -> PyResult<PyRefMut<'_, Self>> {
        if thickness <= 0.0 || !thickness.is_finite() {
            return Err(PyValueError::new_err(
                "thickness must be a positive finite number",
            ));
        }
        slf.inner.edge_thickness = thickness;
        Ok(slf)
    }

    fn with_pulse_radius(mut slf: PyRefMut<'_, Self>, radius: f32) -> PyResult<PyRefMut<'_, Self>> {
        if radius < 0.0 || !radius.is_finite() {
            return Err(PyValueError::new_err(
                "radius must be a non-negative finite number",
            ));
        }
        slf.inner.pulse_radius = radius;
        Ok(slf)
    }

    fn with_node_radius(mut slf: PyRefMut<'_, Self>, radius: f32) -> PyResult<PyRefMut<'_, Self>> {
        if radius < 0.0 || !radius.is_finite() {
            return Err(PyValueError::new_err(
                "radius must be a non-negative finite number",
            ));
        }
        slf.inner.node_radius = radius;
        Ok(slf)
    }

    fn with_highlight_nodes(mut slf: PyRefMut<'_, Self>, highlight: bool) -> PyRefMut<'_, Self> {
        slf.inner.highlight_nodes = highlight;
        slf
    }

    fn current_position(&self) -> Option<Vec3Tuple> {
        self.inner.current_position().map(vec3_tuple)
    }

    fn current_positions(&self) -> Vec<Vec3Tuple> {
        self.inner
            .current_positions()
            .into_iter()
            .map(vec3_tuple)
            .collect()
    }
}
