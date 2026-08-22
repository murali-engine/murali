use glam::{Vec2, Vec4, vec2, vec3};

use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};
use crate::resource::text::layout::measure_label;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Experimental API: this type is part of the evolving linear algebra visual toolkit.
pub enum CoordinateReadoutMode {
    Tuple,
    RowVector,
    ColumnVector,
    FeatureList,
}

#[derive(Debug, Clone)]
/// Experimental API: this type is part of the evolving linear algebra visual toolkit.
pub struct CoordinateReadout {
    pub values: Vec<f32>,
    pub labels: Vec<String>,
    pub mode: CoordinateReadoutMode,
    pub text_height: f32,
    pub color: Vec4,
    pub highlight_color: Vec4,
    pub highlighted_indices: Vec<usize>,
}

impl CoordinateReadout {
    pub fn new(values: Vec<f32>) -> Self {
        Self {
            labels: Vec::new(),
            values,
            mode: CoordinateReadoutMode::Tuple,
            text_height: 0.24,
            color: Vec4::ONE,
            highlight_color: Vec4::new(0.95, 0.82, 0.34, 1.0),
            highlighted_indices: Vec::new(),
        }
    }

    pub fn from_vec2(vector: Vec2) -> Self {
        Self::new(vec![vector.x, vector.y])
    }

    pub fn with_labels(mut self, labels: Vec<impl Into<String>>) -> Self {
        self.labels = labels.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_mode(mut self, mode: CoordinateReadoutMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_highlights(mut self, indices: Vec<usize>) -> Self {
        self.highlighted_indices = indices;
        self
    }

    fn format_value(value: f32) -> String {
        if (value - value.round()).abs() < 0.005 {
            format!("{value:.0}")
        } else {
            format!("{value:.2}")
        }
    }

    fn lines(&self) -> Vec<String> {
        match self.mode {
            CoordinateReadoutMode::Tuple => {
                let values = self
                    .values
                    .iter()
                    .map(|v| Self::format_value(*v))
                    .collect::<Vec<_>>()
                    .join(", ");
                vec![format!("({values})")]
            }
            CoordinateReadoutMode::RowVector => {
                let values = self
                    .values
                    .iter()
                    .map(|v| Self::format_value(*v))
                    .collect::<Vec<_>>()
                    .join("  ");
                vec![format!("[ {values} ]")]
            }
            CoordinateReadoutMode::ColumnVector => self
                .values
                .iter()
                .map(|v| format!("[ {} ]", Self::format_value(*v)))
                .collect(),
            CoordinateReadoutMode::FeatureList => self
                .values
                .iter()
                .enumerate()
                .map(|(idx, value)| {
                    let label = self
                        .labels
                        .get(idx)
                        .cloned()
                        .unwrap_or_else(|| format!("x{idx}"));
                    format!("{label}: {}", Self::format_value(*value))
                })
                .collect(),
        }
    }
}

impl Project for CoordinateReadout {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let lines = self.lines();
        let line_gap = self.text_height * 1.35;
        let top = (lines.len().saturating_sub(1) as f32) * line_gap * 0.5;
        for (idx, line) in lines.iter().enumerate() {
            let color = if self.highlighted_indices.contains(&idx) {
                self.highlight_color
            } else {
                self.color
            };
            ctx.emit(RenderPrimitive::Text {
                content: line.clone(),
                height: self.text_height,
                color,
                font_name: None,
                offset: vec3(0.0, top - idx as f32 * line_gap, 0.0),
                rotation: 0.0,
            });
        }
    }
}

impl Bounded for CoordinateReadout {
    fn local_bounds(&self) -> Bounds {
        let lines = self.lines();
        let width = lines
            .iter()
            .map(|line| measure_label(line, self.text_height, None).width)
            .fold(0.0, f32::max);
        let height = lines.len().max(1) as f32 * self.text_height * 1.35;
        Bounds::from_center_size(Vec2::ZERO, vec2(width, height))
    }
}
