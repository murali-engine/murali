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

    fn with_color(mut slf: PyRefMut<'_, Self>, color: ColorTuple) -> PyResult<PyRefMut<'_, Self>> {
        slf.inner.color = color_from_tuple(color)?;
        Ok(slf)
    }

    fn with_font(mut slf: PyRefMut<'_, Self>, font_name: String) -> PyRefMut<'_, Self> {
        slf.inner.font_name = Some(font_name);
        slf
    }

    fn __repr__(&self) -> String {
        format!(
            "Label(text={:?}, height={})",
            self.inner.text, self.inner.world_height
        )
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

    fn with_color(mut slf: PyRefMut<'_, Self>, color: ColorTuple) -> PyResult<PyRefMut<'_, Self>> {
        slf.inner = slf.inner.clone().with_color(color_from_tuple(color)?);
        Ok(slf)
    }

    fn with_theme<'py>(mut slf: PyRefMut<'py, Self>, theme: &str) -> PyResult<PyRefMut<'py, Self>> {
        slf.inner = slf.inner
            .clone()
            .with_theme(code_block_theme_from_name(theme)?);
        Ok(slf)
    }

    fn with_surface<'py>(mut slf: PyRefMut<'py, Self>, surface: &str) -> PyResult<PyRefMut<'py, Self>> {
        slf.inner = slf.inner
            .clone()
            .with_surface(code_block_surface_from_name(surface)?);
        Ok(slf)
    }

    fn with_title(mut slf: PyRefMut<'_, Self>, title: String) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().with_title(title);
        slf
    }

    fn with_controls(mut slf: PyRefMut<'_, Self>, show: bool) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().with_controls(show);
        slf
    }

    fn with_line_numbers(mut slf: PyRefMut<'_, Self>, show: bool) -> PyRefMut<'_, Self> {
        slf.inner = slf.inner.clone().with_line_numbers(show);
        slf
    }

    fn with_content_box_size(mut slf: PyRefMut<'_, Self>, width: f32, height: f32) -> PyResult<PyRefMut<'_, Self>> {
        if width <= 0.0 || height <= 0.0 || !width.is_finite() || !height.is_finite() {
            return Err(PyValueError::new_err(
                "width and height must be positive finite numbers",
            ));
        }
        slf.inner = slf.inner.clone().with_content_box_size(width, height);
        Ok(slf)
    }

    fn with_content_offset(mut slf: PyRefMut<'_, Self>, x: f32, y: f32) -> PyResult<PyRefMut<'_, Self>> {
        if !x.is_finite() || !y.is_finite() {
            return Err(PyValueError::new_err(
                "content offset values must be finite numbers",
            ));
        }
        slf.inner = slf.inner.clone().with_content_offset(x, y);
        Ok(slf)
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

    fn with_color(mut slf: PyRefMut<'_, Self>, color: ColorTuple) -> PyResult<PyRefMut<'_, Self>> {
        slf.inner = slf.inner.clone().with_color(color_from_tuple(color)?);
        Ok(slf)
    }

    #[staticmethod]
    #[pyo3(signature = (source, height = 1.0, color = None))]
    fn vector_paths(
        source: String,
        height: f32,
        color: Option<ColorTuple>,
    ) -> PyResult<Vec<PyVectorPath>> {
        validate_vector_source(&source, height)?;
        let symbols = latex_vector_paths(
            &source,
            height,
            color_from_tuple(color.unwrap_or(color_tuple(colors::WHITE)))?,
        )
        .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
        Ok(py_vector_paths(symbols))
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

    fn with_color(mut slf: PyRefMut<'_, Self>, color: ColorTuple) -> PyResult<PyRefMut<'_, Self>> {
        slf.inner = slf.inner.clone().with_color(color_from_tuple(color)?);
        Ok(slf)
    }

    #[staticmethod]
    #[pyo3(signature = (source, height = 1.0, sample_count = 256))]
    fn outline_points(
        source: String,
        height: f32,
        sample_count: usize,
    ) -> PyResult<Vec<Vec2Tuple>> {
        if source.trim().is_empty() {
            return Err(PyValueError::new_err("source must not be empty"));
        }
        if height <= 0.0 || !height.is_finite() {
            return Err(PyValueError::new_err(
                "height must be a positive finite number",
            ));
        }
        if sample_count < 3 {
            return Err(PyValueError::new_err("sample_count must be at least 3"));
        }
        let points = typst_outline_points(&source, height, sample_count)
            .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
        Ok(points.into_iter().map(|point| (point.x, point.y)).collect())
    }

    #[staticmethod]
    #[pyo3(signature = (source, height = 1.0, color = None))]
    fn vector_paths(
        source: String,
        height: f32,
        color: Option<ColorTuple>,
    ) -> PyResult<Vec<PyVectorPath>> {
        validate_vector_source(&source, height)?;
        let symbols = typst_vector_paths(
            &source,
            height,
            color_from_tuple(color.unwrap_or(color_tuple(colors::WHITE)))?,
        )
        .map_err(|error| PyRuntimeError::new_err(error.to_string()))?;
        Ok(py_vector_paths(symbols))
    }
}

#[pyclass(name = "VectorPath", module = "murali_engine")]
#[derive(Clone, Debug)]
struct PyVectorPath {
    path: crate::frontend::collection::primitives::path::Path,
    #[pyo3(get)]
    key: String,
    #[pyo3(get)]
    center: Vec2Tuple,
}

#[pymethods]
impl PyVectorPath {
    #[getter]
    fn path(&self) -> PyPath {
        PyPath {
            inner: self.path.clone(),
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "VectorPath(key={:?}, center=({:.3}, {:.3}))",
            self.key, self.center.0, self.center.1
        )
    }
}

fn validate_vector_source(source: &str, height: f32) -> PyResult<()> {
    if source.trim().is_empty() {
        return Err(PyValueError::new_err("source must not be empty"));
    }
    if height <= 0.0 || !height.is_finite() {
        return Err(PyValueError::new_err(
            "height must be a positive finite number",
        ));
    }
    Ok(())
}

fn py_vector_paths(
    symbols: Vec<crate::resource::typst_resource::vector::VectorSymbol>,
) -> Vec<PyVectorPath> {
    symbols
        .into_iter()
        .map(|symbol| PyVectorPath {
            path: symbol.path,
            key: symbol.key,
            center: (symbol.center.x, symbol.center.y),
        })
        .collect()
}

#[pyclass(name = "Letter3D", module = "murali_engine")]
#[derive(Clone)]
struct PyLetter3D {
    inner: Letter3D,
}

#[pymethods]
impl PyLetter3D {
    #[new]
    #[pyo3(signature = (character, height = 2.4, depth = 0.95, font_path = None))]
    fn new(
        character: String,
        height: f32,
        depth: f32,
        font_path: Option<String>,
    ) -> PyResult<Self> {
        let character = capital_letter(&character)?;
        if height <= 0.0 || depth <= 0.0 || !height.is_finite() || !depth.is_finite() {
            return Err(PyValueError::new_err(
                "height and depth must be positive finite numbers",
            ));
        }
        let inner = match font_path.as_deref() {
            Some(path) => Letter3D::from_font_path(character, height, depth, path),
            None => Letter3D::new(character, height, depth),
        }
        .map_err(|error| PyValueError::new_err(error.to_string()))?;
        Ok(Self { inner })
    }

    fn with_face_colors(
        mut slf: PyRefMut<'_, Self>,
        front: ColorTuple,
        back: ColorTuple,
        side: ColorTuple,
    ) -> PyResult<PyRefMut<'_, Self>> {
        slf.inner = slf.inner.clone().with_face_colors(
            color_from_tuple(front)?,
            color_from_tuple(back)?,
            color_from_tuple(side)?,
        );
        Ok(slf)
    }

    fn with_texture<'py>(mut slf: PyRefMut<'py, Self>, name: &str) -> PyResult<PyRefMut<'py, Self>> {
        slf.inner = slf.inner
            .clone()
            .with_texture(TextureImage::builtin(builtin_texture_from_name(name)?));
        Ok(slf)
    }

    fn width(&self) -> f32 {
        self.inner.width()
    }
}

#[pyclass(name = "LetterParticles3D", module = "murali_engine")]
#[derive(Clone)]
struct PyLetterParticles3D {
    inner: LetterParticles3D,
}

#[pymethods]
impl PyLetterParticles3D {
    #[new]
    #[pyo3(signature = (character, height = 2.4, depth = 0.95, count = 700, font_path = None))]
    fn new(
        character: String,
        height: f32,
        depth: f32,
        count: usize,
        font_path: Option<String>,
    ) -> PyResult<Self> {
        let character = capital_letter(&character)?;
        if height <= 0.0 || depth <= 0.0 || !height.is_finite() || !depth.is_finite() {
            return Err(PyValueError::new_err(
                "height and depth must be positive finite numbers",
            ));
        }
        if count == 0 {
            return Err(PyValueError::new_err("count must be greater than zero"));
        }
        let inner = match font_path.as_deref() {
            Some(path) => LetterParticles3D::from_font_path(character, height, depth, count, path),
            None => LetterParticles3D::new(character, height, depth, count),
        }
        .map_err(|error| PyValueError::new_err(error.to_string()))?;
        Ok(Self { inner })
    }

    fn with_motion(mut slf: PyRefMut<'_, Self>, distance: f32, rise: f32, curl: f32) -> PyResult<PyRefMut<'_, Self>> {
        for (name, value) in [("distance", distance), ("rise", rise), ("curl", curl)] {
            if !value.is_finite() {
                return Err(PyValueError::new_err(format!(
                    "{name} must be a finite number"
                )));
            }
        }
        slf.inner = slf.inner.clone().with_motion(distance, rise, curl);
        Ok(slf)
    }

    fn with_particle_size(mut slf: PyRefMut<'_, Self>, size: f32) -> PyResult<PyRefMut<'_, Self>> {
        if size <= 0.0 || !size.is_finite() {
            return Err(PyValueError::new_err(
                "size must be a positive finite number",
            ));
        }
        slf.inner = slf.inner.clone().with_particle_size(size);
        Ok(slf)
    }

    fn with_color(mut slf: PyRefMut<'_, Self>, color: ColorTuple) -> PyResult<PyRefMut<'_, Self>> {
        slf.inner = slf.inner.clone().with_color(color_from_tuple(color)?);
        Ok(slf)
    }

    fn with_palette(mut slf: PyRefMut<'_, Self>, palette: Vec<ColorTuple>) -> PyResult<PyRefMut<'_, Self>> {
        if palette.is_empty() {
            return Err(PyValueError::new_err("palette must not be empty"));
        }
        let palette = palette
            .into_iter()
            .map(color_from_tuple)
            .collect::<PyResult<Vec<_>>>()?;
        slf.inner = slf.inner.clone().with_palette(palette);
        Ok(slf)
    }

    fn with_seed(mut slf: PyRefMut<'_, Self>, seed: f32) -> PyResult<PyRefMut<'_, Self>> {
        if !seed.is_finite() {
            return Err(PyValueError::new_err("seed must be a finite number"));
        }
        slf.inner = slf.inner.clone().with_seed(seed);
        Ok(slf)
    }

    fn particle_count(&self) -> usize {
        self.inner.particle_count()
    }
}
