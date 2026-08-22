use std::collections::{HashMap, HashSet};

use glam::{Vec2, Vec3, Vec4, vec2};
use serde::{Deserialize, Serialize};

use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Mesh, Project, ProjectionCtx, RenderPrimitive};
use crate::resource::text::layout::measure_label;
use crate::validation::ValidationError;

const SNAPSHOT_COMPONENT: &str = "TensorSnapshot";
const VIEW_COMPONENT: &str = "TensorView";

/// Semantic metadata for one tensor dimension.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TensorAxis {
    pub id: String,
    pub label: String,
    pub element_ids: Vec<String>,
    pub element_labels: Vec<String>,
}

impl TensorAxis {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        element_labels: Vec<impl Into<String>>,
    ) -> Self {
        let id = id.into();
        let element_labels: Vec<String> = element_labels.into_iter().map(Into::into).collect();
        let element_ids = (0..element_labels.len())
            .map(|index| format!("{id}[{index}]"))
            .collect();
        Self {
            id,
            label: label.into(),
            element_ids,
            element_labels,
        }
    }

    /// Creates an axis whose elements keep explicit identity across reordering and resizing.
    pub fn with_elements<I, E, L>(
        id: impl Into<String>,
        label: impl Into<String>,
        elements: I,
    ) -> Self
    where
        I: IntoIterator<Item = (E, L)>,
        E: Into<String>,
        L: Into<String>,
    {
        let (element_ids, element_labels) = elements
            .into_iter()
            .map(|(element_id, element_label)| (element_id.into(), element_label.into()))
            .unzip();
        Self {
            id: id.into(),
            label: label.into(),
            element_ids,
            element_labels,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TensorCoordinate {
    pub axis_id: String,
    pub element_id: String,
}

/// Stable identity for a tensor element across views and animation steps.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TensorElementId {
    pub tensor_id: String,
    pub coordinates: Vec<TensorCoordinate>,
}

/// Validated numerical state from which one or more visual views can be projected.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TensorSnapshot {
    pub id: String,
    pub shape: Vec<usize>,
    pub values: Vec<f32>,
    pub axes: Vec<TensorAxis>,
}

impl TensorSnapshot {
    pub fn try_new(
        id: impl Into<String>,
        shape: Vec<usize>,
        values: Vec<f32>,
        axes: Vec<TensorAxis>,
    ) -> Result<Self, ValidationError> {
        let snapshot = Self {
            id: id.into(),
            shape,
            values,
            axes,
        };
        snapshot.validate()?;
        Ok(snapshot)
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.id.trim().is_empty() {
            return Err(ValidationError::Empty {
                component: SNAPSHOT_COMPONENT,
                field: "id",
            });
        }
        if self.shape.is_empty() {
            return Err(ValidationError::Empty {
                component: SNAPSHOT_COMPONENT,
                field: "shape",
            });
        }
        for &dimension in &self.shape {
            if dimension == 0 {
                return Err(ValidationError::CountTooSmall {
                    component: SNAPSHOT_COMPONENT,
                    field: "shape dimension",
                    minimum: 1,
                    actual: 0,
                });
            }
        }

        let expected_values = self
            .shape
            .iter()
            .try_fold(1_usize, |size, dimension| size.checked_mul(*dimension));
        let Some(expected_values) = expected_values else {
            return Err(ValidationError::ShapeOverflow {
                component: SNAPSHOT_COMPONENT,
                field: "shape",
            });
        };
        if self.values.len() != expected_values {
            return Err(ValidationError::LengthMismatch {
                component: SNAPSHOT_COMPONENT,
                field: "values",
                expected: expected_values,
                actual: self.values.len(),
            });
        }
        if self.axes.len() != self.shape.len() {
            return Err(ValidationError::LengthMismatch {
                component: SNAPSHOT_COMPONENT,
                field: "axes",
                expected: self.shape.len(),
                actual: self.axes.len(),
            });
        }

        let mut axis_ids = HashSet::with_capacity(self.axes.len());
        for (index, axis) in self.axes.iter().enumerate() {
            if axis.id.trim().is_empty() {
                return Err(ValidationError::Empty {
                    component: SNAPSHOT_COMPONENT,
                    field: "axis id",
                });
            }
            if axis.label.trim().is_empty() {
                return Err(ValidationError::Empty {
                    component: SNAPSHOT_COMPONENT,
                    field: "axis label",
                });
            }
            if !axis_ids.insert(axis.id.as_str()) {
                return Err(ValidationError::DuplicateIdentifier {
                    component: SNAPSHOT_COMPONENT,
                    field: "axes",
                    value: axis.id.clone(),
                });
            }
            if axis.element_labels.len() != self.shape[index] {
                return Err(ValidationError::LengthMismatch {
                    component: SNAPSHOT_COMPONENT,
                    field: "axis element labels",
                    expected: self.shape[index],
                    actual: axis.element_labels.len(),
                });
            }
            if axis.element_ids.len() != self.shape[index] {
                return Err(ValidationError::LengthMismatch {
                    component: SNAPSHOT_COMPONENT,
                    field: "axis element ids",
                    expected: self.shape[index],
                    actual: axis.element_ids.len(),
                });
            }
            let mut element_ids = HashSet::with_capacity(axis.element_ids.len());
            for element_id in &axis.element_ids {
                if element_id.trim().is_empty() {
                    return Err(ValidationError::Empty {
                        component: SNAPSHOT_COMPONENT,
                        field: "axis element id",
                    });
                }
                if !element_ids.insert(element_id.as_str()) {
                    return Err(ValidationError::DuplicateIdentifier {
                        component: SNAPSHOT_COMPONENT,
                        field: "axis elements",
                        value: element_id.clone(),
                    });
                }
            }
        }

        for &value in &self.values {
            if !value.is_finite() {
                return Err(ValidationError::NonFinite {
                    component: SNAPSHOT_COMPONENT,
                    field: "values",
                    value,
                });
            }
        }
        Ok(())
    }

    pub fn rank(&self) -> usize {
        self.shape.len()
    }

    pub fn element_id(&self, indices: &[usize]) -> Option<TensorElementId> {
        self.flat_index(indices).map(|_| TensorElementId {
            tensor_id: self.id.clone(),
            coordinates: self
                .axes
                .iter()
                .zip(indices)
                .map(|(axis, &index)| TensorCoordinate {
                    axis_id: axis.id.clone(),
                    element_id: axis.element_ids[index].clone(),
                })
                .collect(),
        })
    }

    pub fn validate_transition_to(&self, target: &Self) -> Result<(), ValidationError> {
        self.validate()?;
        target.validate()?;
        if self.id != target.id {
            return Err(ValidationError::Incompatible {
                component: "TensorTransition",
                field: "tensor id",
                reason: format!("expected '{}', got '{}'", self.id, target.id),
            });
        }
        if self.rank() != target.rank() {
            return Err(ValidationError::RankMismatch {
                component: "TensorTransition",
                field: "snapshot",
                expected: self.rank(),
                actual: target.rank(),
            });
        }
        for (source_axis, target_axis) in self.axes.iter().zip(&target.axes) {
            if source_axis.id != target_axis.id {
                return Err(ValidationError::Incompatible {
                    component: "TensorTransition",
                    field: "axis order",
                    reason: format!(
                        "expected axis '{}', got '{}'",
                        source_axis.id, target_axis.id
                    ),
                });
            }
        }
        Ok(())
    }

    pub(crate) fn interpolated_target(&self, target: &Self, progress: f32) -> Self {
        let progress = progress.clamp(0.0, 1.0);
        let source_values: HashMap<TensorElementId, f32> = self
            .values
            .iter()
            .enumerate()
            .filter_map(|(flat_index, &value)| {
                let mut remainder = flat_index;
                let mut indices = vec![0; self.rank()];
                for axis in (0..self.rank()).rev() {
                    indices[axis] = remainder % self.shape[axis];
                    remainder /= self.shape[axis];
                }
                self.element_id(&indices).map(|id| (id, value))
            })
            .collect();
        let mut interpolated = target.clone();
        for flat_index in 0..target.values.len() {
            let mut remainder = flat_index;
            let mut indices = vec![0; target.rank()];
            for axis in (0..target.rank()).rev() {
                indices[axis] = remainder % target.shape[axis];
                remainder /= target.shape[axis];
            }
            if let Some(source_value) = target
                .element_id(&indices)
                .and_then(|id| source_values.get(&id))
            {
                let target_value = target.values[flat_index];
                interpolated.values[flat_index] =
                    source_value + (target_value - source_value) * progress;
            }
        }
        interpolated
    }

    pub fn value(&self, indices: &[usize]) -> Option<f32> {
        self.flat_index(indices).map(|index| self.values[index])
    }

    fn flat_index(&self, indices: &[usize]) -> Option<usize> {
        if indices.len() != self.shape.len() {
            return None;
        }
        indices
            .iter()
            .zip(&self.shape)
            .try_fold(0_usize, |offset, (&index, &dimension)| {
                (index < dimension).then(|| offset * dimension + index)
            })
    }
}

/// A semantic selection that remains meaningful when a tensor is restyled.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TensorSelector {
    All,
    Element(Vec<usize>),
    AxisIndex { axis_id: String, index: usize },
    AxisElement { axis_id: String, element_id: String },
}

impl TensorSelector {
    pub fn element(indices: impl Into<Vec<usize>>) -> Self {
        Self::Element(indices.into())
    }

    pub fn axis(axis_id: impl Into<String>, index: usize) -> Self {
        Self::AxisIndex {
            axis_id: axis_id.into(),
            index,
        }
    }

    pub fn axis_element(axis_id: impl Into<String>, element_id: impl Into<String>) -> Self {
        Self::AxisElement {
            axis_id: axis_id.into(),
            element_id: element_id.into(),
        }
    }

    fn validate(&self, snapshot: &TensorSnapshot) -> Result<(), ValidationError> {
        match self {
            Self::All => Ok(()),
            Self::Element(indices) => {
                if indices.len() != snapshot.rank() {
                    return Err(ValidationError::RankMismatch {
                        component: VIEW_COMPONENT,
                        field: "selection element",
                        expected: snapshot.rank(),
                        actual: indices.len(),
                    });
                }
                for (&index, &length) in indices.iter().zip(&snapshot.shape) {
                    if index >= length {
                        return Err(ValidationError::IndexOutOfBounds {
                            component: VIEW_COMPONENT,
                            field: "selection element",
                            index,
                            length,
                        });
                    }
                }
                Ok(())
            }
            Self::AxisIndex { axis_id, index } => {
                let Some(axis_position) = snapshot.axes.iter().position(|axis| axis.id == *axis_id)
                else {
                    return Err(ValidationError::UnknownIdentifier {
                        component: VIEW_COMPONENT,
                        field: "selection axis",
                        value: axis_id.clone(),
                    });
                };
                let length = snapshot.shape[axis_position];
                if *index >= length {
                    return Err(ValidationError::IndexOutOfBounds {
                        component: VIEW_COMPONENT,
                        field: "selection axis index",
                        index: *index,
                        length,
                    });
                }
                Ok(())
            }
            Self::AxisElement {
                axis_id,
                element_id,
            } => {
                let Some(axis) = snapshot.axes.iter().find(|axis| axis.id == *axis_id) else {
                    return Err(ValidationError::UnknownIdentifier {
                        component: VIEW_COMPONENT,
                        field: "selection axis",
                        value: axis_id.clone(),
                    });
                };
                if !axis.element_ids.contains(element_id) {
                    return Err(ValidationError::UnknownIdentifier {
                        component: VIEW_COMPONENT,
                        field: "selection axis element",
                        value: element_id.clone(),
                    });
                }
                Ok(())
            }
        }
    }

    fn matches(&self, snapshot: &TensorSnapshot, indices: &[usize]) -> bool {
        match self {
            Self::All => true,
            Self::Element(selected) => selected == indices,
            Self::AxisIndex { axis_id, index } => snapshot
                .axes
                .iter()
                .position(|axis| axis.id == *axis_id)
                .is_some_and(|axis_position| indices[axis_position] == *index),
            Self::AxisElement {
                axis_id,
                element_id,
            } => snapshot
                .axes
                .iter()
                .position(|axis| axis.id == *axis_id)
                .is_some_and(|axis_position| {
                    snapshot.axes[axis_position].element_ids[indices[axis_position]] == *element_id
                }),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TensorCellLayout {
    pub id: TensorElementId,
    pub row: usize,
    pub column: usize,
    pub value: f32,
    pub center: Vec3,
    pub selected: bool,
    pub selection_strength: f32,
    pub opacity: f32,
}

#[derive(Debug, Clone)]
pub(crate) struct TensorTransitionFrame {
    pub source: TensorSnapshot,
    pub target: TensorSnapshot,
    pub progress: f32,
}

#[derive(Debug, Clone)]
pub(crate) struct TensorSelectionFrame {
    pub source: Vec<TensorSelector>,
    pub target: Vec<TensorSelector>,
    pub progress: f32,
}

/// A heatmap-like rank-2 projection of a semantic tensor snapshot.
#[derive(Debug, Clone)]
pub struct TensorView {
    pub snapshot: TensorSnapshot,
    pub cell_size: Vec2,
    pub selections: Vec<TensorSelector>,
    pub negative_color: Vec4,
    pub zero_color: Vec4,
    pub positive_color: Vec4,
    pub grid_color: Vec4,
    pub label_color: Vec4,
    pub value_color: Vec4,
    pub selection_color: Vec4,
    pub grid_thickness: f32,
    pub label_height: f32,
    pub value_height: f32,
    pub show_values: bool,
    pub scale_limit: Option<f32>,
    pub(crate) transition: Option<TensorTransitionFrame>,
    pub(crate) selection_transition: Option<TensorSelectionFrame>,
}

impl TensorView {
    pub fn try_new(snapshot: TensorSnapshot) -> Result<Self, ValidationError> {
        let view = Self {
            snapshot,
            cell_size: vec2(0.72, 0.54),
            selections: Vec::new(),
            negative_color: Vec4::new(0.94, 0.42, 0.48, 1.0),
            zero_color: Vec4::new(0.12, 0.16, 0.22, 1.0),
            positive_color: Vec4::new(0.25, 0.78, 0.74, 1.0),
            grid_color: Vec4::new(0.70, 0.76, 0.82, 1.0),
            label_color: Vec4::new(0.90, 0.93, 0.96, 1.0),
            value_color: Vec4::new(0.97, 0.98, 0.99, 1.0),
            selection_color: Vec4::new(1.0, 0.78, 0.28, 1.0),
            grid_thickness: 0.012,
            label_height: 0.18,
            value_height: 0.16,
            show_values: true,
            scale_limit: None,
            transition: None,
            selection_transition: None,
        };
        view.validate()?;
        Ok(view)
    }

    pub fn with_selection(mut self, selector: TensorSelector) -> Result<Self, ValidationError> {
        selector.validate(&self.snapshot)?;
        self.selections.push(selector);
        Ok(self)
    }

    pub fn validate(&self) -> Result<(), ValidationError> {
        self.snapshot.validate()?;
        if self.snapshot.rank() != 2 {
            return Err(ValidationError::RankMismatch {
                component: VIEW_COMPONENT,
                field: "snapshot",
                expected: 2,
                actual: self.snapshot.rank(),
            });
        }
        for (field, value) in [
            ("cell width", self.cell_size.x),
            ("cell height", self.cell_size.y),
            ("grid thickness", self.grid_thickness),
            ("label height", self.label_height),
            ("value height", self.value_height),
        ] {
            if !value.is_finite() {
                return Err(ValidationError::NonFinite {
                    component: VIEW_COMPONENT,
                    field,
                    value,
                });
            }
            if value <= 0.0 {
                return Err(ValidationError::NonPositive {
                    component: VIEW_COMPONENT,
                    field,
                    value,
                });
            }
        }
        if let Some(limit) = self.scale_limit {
            if !limit.is_finite() {
                return Err(ValidationError::NonFinite {
                    component: VIEW_COMPONENT,
                    field: "scale limit",
                    value: limit,
                });
            }
            if limit <= 0.0 {
                return Err(ValidationError::NonPositive {
                    component: VIEW_COMPONENT,
                    field: "scale limit",
                    value: limit,
                });
            }
        }
        for (field, color) in [
            ("negative color", self.negative_color),
            ("zero color", self.zero_color),
            ("positive color", self.positive_color),
            ("grid color", self.grid_color),
            ("label color", self.label_color),
            ("value color", self.value_color),
            ("selection color", self.selection_color),
        ] {
            for value in color.to_array() {
                if !value.is_finite() {
                    return Err(ValidationError::NonFinite {
                        component: VIEW_COMPONENT,
                        field,
                        value,
                    });
                }
            }
        }
        for selection in &self.selections {
            selection.validate(&self.snapshot)?;
        }
        if let Some(frame) = &self.selection_transition {
            if !frame.progress.is_finite() {
                return Err(ValidationError::NonFinite {
                    component: VIEW_COMPONENT,
                    field: "selection transition progress",
                    value: frame.progress,
                });
            }
            Self::validate_selections(&frame.source, &self.snapshot)?;
            Self::validate_selections(&frame.target, &self.snapshot)?;
        }
        if let Some(frame) = &self.transition {
            frame.source.validate_transition_to(&frame.target)?;
            if frame.source.rank() != 2 {
                return Err(ValidationError::RankMismatch {
                    component: VIEW_COMPONENT,
                    field: "transition snapshot",
                    expected: 2,
                    actual: frame.source.rank(),
                });
            }
            if !frame.progress.is_finite() {
                return Err(ValidationError::NonFinite {
                    component: VIEW_COMPONENT,
                    field: "transition progress",
                    value: frame.progress,
                });
            }
            for selection in &self.selections {
                selection.validate(&frame.target)?;
            }
            if let Some(selection_frame) = &self.selection_transition {
                Self::validate_selections(&selection_frame.source, &frame.target)?;
                Self::validate_selections(&selection_frame.target, &frame.target)?;
            }
        }
        Ok(())
    }

    pub(crate) fn validate_selections(
        selections: &[TensorSelector],
        snapshot: &TensorSnapshot,
    ) -> Result<(), ValidationError> {
        for selection in selections {
            selection.validate(snapshot)?;
        }
        Ok(())
    }

    pub fn layout_snapshot(&self) -> Result<Vec<TensorCellLayout>, ValidationError> {
        self.validate()?;
        if let Some(frame) = &self.transition {
            return Ok(self.transition_layout(frame));
        }
        Ok(self.snapshot_layout(&self.snapshot))
    }

    fn snapshot_layout(&self, snapshot: &TensorSnapshot) -> Vec<TensorCellLayout> {
        let rows = snapshot.shape[0];
        let columns = snapshot.shape[1];
        let width = columns as f32 * self.cell_size.x;
        let height = rows as f32 * self.cell_size.y;
        let left = -width * 0.5;
        let top = height * 0.5;
        let mut cells = Vec::with_capacity(rows * columns);

        for row in 0..rows {
            for column in 0..columns {
                let indices = [row, column];
                let selection_strength = self.selection_strength(snapshot, &indices);
                cells.push(TensorCellLayout {
                    id: TensorElementId {
                        tensor_id: snapshot.id.clone(),
                        coordinates: snapshot
                            .axes
                            .iter()
                            .zip(indices)
                            .map(|(axis, index)| TensorCoordinate {
                                axis_id: axis.id.clone(),
                                element_id: axis.element_ids[index].clone(),
                            })
                            .collect(),
                    },
                    row,
                    column,
                    value: snapshot.values[row * columns + column],
                    center: Vec3::new(
                        left + (column as f32 + 0.5) * self.cell_size.x,
                        top - (row as f32 + 0.5) * self.cell_size.y,
                        0.0,
                    ),
                    selected: selection_strength > f32::EPSILON,
                    selection_strength,
                    opacity: 1.0,
                });
            }
        }
        cells
    }

    fn transition_layout(&self, frame: &TensorTransitionFrame) -> Vec<TensorCellLayout> {
        let progress = frame.progress.clamp(0.0, 1.0);
        let source_cells = self.snapshot_layout(&frame.source);
        let target_cells = self.snapshot_layout(&frame.target);
        let source_by_id: HashMap<TensorElementId, &TensorCellLayout> = source_cells
            .iter()
            .map(|cell| (cell.id.clone(), cell))
            .collect();
        let mut matched = HashSet::new();
        let mut cells = Vec::with_capacity(source_cells.len().max(target_cells.len()));

        for target in target_cells {
            if let Some(source) = source_by_id.get(&target.id) {
                matched.insert(target.id.clone());
                cells.push(TensorCellLayout {
                    id: target.id,
                    row: target.row,
                    column: target.column,
                    value: source.value + (target.value - source.value) * progress,
                    center: source.center.lerp(target.center, progress),
                    selected: target.selected,
                    selection_strength: target.selection_strength,
                    opacity: 1.0,
                });
            } else {
                cells.push(TensorCellLayout {
                    opacity: progress,
                    ..target
                });
            }
        }
        cells.extend(
            source_cells
                .into_iter()
                .filter(|source| !matched.contains(&source.id))
                .map(|source| TensorCellLayout {
                    opacity: 1.0 - progress,
                    ..source
                }),
        );
        cells
    }

    fn selection_strength(&self, snapshot: &TensorSnapshot, indices: &[usize]) -> f32 {
        let matches = |selections: &[TensorSelector]| {
            selections
                .iter()
                .any(|selector| selector.matches(snapshot, indices))
        };
        if let Some(frame) = &self.selection_transition {
            let source = if matches(&frame.source) { 1.0 } else { 0.0 };
            let target = if matches(&frame.target) { 1.0 } else { 0.0 };
            source + (target - source) * frame.progress.clamp(0.0, 1.0)
        } else {
            if matches(&self.selections) { 1.0 } else { 0.0 }
        }
    }

    fn color_for(&self, value: f32) -> Vec4 {
        let limit = self.scale_limit.unwrap_or_else(|| {
            self.active_snapshots()
                .flat_map(|snapshot| snapshot.values.iter())
                .copied()
                .map(f32::abs)
                .fold(0.0, f32::max)
                .max(f32::EPSILON)
        });
        let amount = (value.abs() / limit).clamp(0.0, 1.0);
        if value < 0.0 {
            self.zero_color.lerp(self.negative_color, amount)
        } else {
            self.zero_color.lerp(self.positive_color, amount)
        }
    }

    fn active_snapshots(&self) -> Box<dyn Iterator<Item = &TensorSnapshot> + '_> {
        if let Some(frame) = &self.transition {
            Box::new([&frame.source, &frame.target].into_iter())
        } else {
            Box::new(std::iter::once(&self.snapshot))
        }
    }

    fn emit_line(
        &self,
        ctx: &mut ProjectionCtx,
        start: Vec3,
        end: Vec3,
        color: Vec4,
        thickness: f32,
    ) {
        ctx.emit(RenderPrimitive::Line {
            start,
            end,
            thickness,
            color,
            dash_length: 0.0,
            gap_length: 0.0,
            dash_offset: 0.0,
        });
    }

    fn emit_axis_labels(&self, ctx: &mut ProjectionCtx, snapshot: &TensorSnapshot, opacity: f32) {
        if opacity <= f32::EPSILON {
            return;
        }
        let width = snapshot.shape[1] as f32 * self.cell_size.x;
        let height = snapshot.shape[0] as f32 * self.cell_size.y;
        let left = -width * 0.5;
        let top = height * 0.5;
        let row_axis = &snapshot.axes[0];
        let column_axis = &snapshot.axes[1];
        let mut color = self.label_color;
        color.w *= opacity;

        for (column, label) in column_axis.element_labels.iter().enumerate() {
            let x = left + (column as f32 + 0.5) * self.cell_size.x;
            ctx.emit(RenderPrimitive::Text {
                content: label.clone(),
                height: self.label_height,
                color,
                font_name: None,
                offset: Vec3::new(x, top + self.label_height * 1.1, 0.0),
                rotation: 0.0,
            });
        }
        for (row, label) in row_axis.element_labels.iter().enumerate() {
            let y = top - (row as f32 + 0.5) * self.cell_size.y;
            let label_width = measure_label(label, self.label_height, None).width;
            ctx.emit(RenderPrimitive::Text {
                content: label.clone(),
                height: self.label_height,
                color,
                font_name: None,
                offset: Vec3::new(left - label_width * 0.5 - self.label_height * 0.55, y, 0.0),
                rotation: 0.0,
            });
        }
        ctx.emit(RenderPrimitive::Text {
            content: column_axis.label.clone(),
            height: self.label_height,
            color,
            font_name: None,
            offset: Vec3::new(0.0, top + self.label_height * 2.5, 0.0),
            rotation: 0.0,
        });
        ctx.emit(RenderPrimitive::Text {
            content: row_axis.label.clone(),
            height: self.label_height,
            color,
            font_name: None,
            offset: Vec3::new(
                left - self.row_label_padding_for(snapshot) + self.label_height * 0.35,
                0.0,
                0.0,
            ),
            rotation: std::f32::consts::PI / 2.0,
        });
    }
}

impl Project for TensorView {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let Ok(cells) = self.layout_snapshot() else {
            if let Err(error) = self.validate() {
                ctx.report(error);
            }
            return;
        };
        for cell in &cells {
            let mut fill_color = self.color_for(cell.value);
            fill_color.w *= cell.opacity;
            ctx.emit(RenderPrimitive::Mesh(
                Mesh::rectangle(self.cell_size.x * 0.98, self.cell_size.y * 0.98, fill_color)
                    .as_ref()
                    .translated(cell.center),
            ));
            if self.show_values {
                let mut value_color = self.value_color;
                value_color.w *= cell.opacity;
                ctx.emit(RenderPrimitive::Text {
                    content: format!("{:.2}", cell.value),
                    height: self.value_height,
                    color: value_color,
                    font_name: None,
                    offset: cell.center + Vec3::Z * 0.01,
                    rotation: 0.0,
                });
            }
            if cell.selected {
                let half = self.cell_size * 0.46;
                let z = 0.02;
                let corners = [
                    Vec3::new(cell.center.x - half.x, cell.center.y - half.y, z),
                    Vec3::new(cell.center.x + half.x, cell.center.y - half.y, z),
                    Vec3::new(cell.center.x + half.x, cell.center.y + half.y, z),
                    Vec3::new(cell.center.x - half.x, cell.center.y + half.y, z),
                ];
                let mut selection_color = self.selection_color;
                selection_color.w *= cell.opacity * cell.selection_strength;
                for index in 0..4 {
                    self.emit_line(
                        ctx,
                        corners[index],
                        corners[(index + 1) % 4],
                        selection_color,
                        self.grid_thickness * 2.5,
                    );
                }
            }
            let half = self.cell_size * 0.5;
            let corners = [
                Vec3::new(cell.center.x - half.x, cell.center.y - half.y, 0.01),
                Vec3::new(cell.center.x + half.x, cell.center.y - half.y, 0.01),
                Vec3::new(cell.center.x + half.x, cell.center.y + half.y, 0.01),
                Vec3::new(cell.center.x - half.x, cell.center.y + half.y, 0.01),
            ];
            let mut grid_color = self.grid_color;
            grid_color.w *= cell.opacity;
            for index in 0..4 {
                self.emit_line(
                    ctx,
                    corners[index],
                    corners[(index + 1) % 4],
                    grid_color,
                    self.grid_thickness,
                );
            }
        }

        if let Some(frame) = &self.transition {
            let progress = frame.progress.clamp(0.0, 1.0);
            self.emit_axis_labels(ctx, &frame.source, 1.0 - progress);
            self.emit_axis_labels(ctx, &frame.target, progress);
        } else {
            self.emit_axis_labels(ctx, &self.snapshot, 1.0);
        }
    }
}

impl TensorView {
    fn row_label_padding_for(&self, snapshot: &TensorSnapshot) -> f32 {
        snapshot.axes[0]
            .element_labels
            .iter()
            .map(|label| measure_label(label, self.label_height, None).width)
            .fold(0.0, f32::max)
            + self.label_height * 1.8
    }

    fn content_size_for(&self, snapshot: &TensorSnapshot) -> Vec2 {
        vec2(
            snapshot.shape[1] as f32 * self.cell_size.x
                + self.row_label_padding_for(snapshot) * 2.0,
            snapshot.shape[0] as f32 * self.cell_size.y + self.label_height * 3.2,
        )
    }
}

impl Bounded for TensorView {
    fn local_bounds(&self) -> Bounds {
        if self.validate().is_err() {
            return Bounds::default();
        }
        let size = self
            .active_snapshots()
            .map(|snapshot| self.content_size_for(snapshot))
            .fold(Vec2::ZERO, Vec2::max);
        Bounds::from_center_size(Vec2::ZERO, size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::scene::Scene;
    use crate::engine::timeline::Timeline;
    use crate::frontend::animation::Ease;
    use crate::frontend::collection::primitives::square::Square;

    fn snapshot() -> TensorSnapshot {
        TensorSnapshot::try_new(
            "attention.logits",
            vec![2, 3],
            vec![0.1, -0.2, 0.3, 0.4, 0.5, -0.6],
            vec![
                TensorAxis::new("query", "Queries", vec!["A", "B"]),
                TensorAxis::new("key", "Keys", vec!["A", "B", "C"]),
            ],
        )
        .unwrap()
    }

    fn snapshot_with_values(id: &str, values: Vec<f32>) -> TensorSnapshot {
        TensorSnapshot::try_new(
            id,
            vec![2, 2],
            values,
            vec![
                TensorAxis::with_elements("query", "Queries", [("q0", "A"), ("q1", "B")]),
                TensorAxis::with_elements("key", "Keys", [("k0", "A"), ("k1", "B")]),
            ],
        )
        .unwrap()
    }

    #[test]
    fn snapshot_validates_shape_values_and_axis_labels() {
        let error = TensorSnapshot::try_new(
            "bad",
            vec![2, 2],
            vec![1.0, 2.0],
            vec![
                TensorAxis::new("row", "Rows", vec!["a", "b"]),
                TensorAxis::new("column", "Columns", vec!["x", "y"]),
            ],
        )
        .unwrap_err();
        assert!(matches!(
            error,
            ValidationError::LengthMismatch {
                field: "values",
                ..
            }
        ));

        let mut non_finite = snapshot();
        non_finite.values[0] = f32::NAN;
        assert!(matches!(
            non_finite.validate(),
            Err(ValidationError::NonFinite { .. })
        ));
    }

    #[test]
    fn element_identity_and_row_major_access_are_stable() {
        let snapshot = snapshot();
        assert_eq!(snapshot.value(&[1, 2]), Some(-0.6));
        assert_eq!(
            snapshot.element_id(&[1, 2]),
            Some(TensorElementId {
                tensor_id: "attention.logits".into(),
                coordinates: vec![
                    TensorCoordinate {
                        axis_id: "query".into(),
                        element_id: "query[1]".into(),
                    },
                    TensorCoordinate {
                        axis_id: "key".into(),
                        element_id: "key[2]".into(),
                    },
                ],
            })
        );
        assert_eq!(snapshot.value(&[2, 0]), None);
    }

    #[test]
    fn semantic_axis_selection_is_deterministic() {
        let view = TensorView::try_new(snapshot())
            .unwrap()
            .with_selection(TensorSelector::axis("key", 1))
            .unwrap()
            .with_selection(TensorSelector::element(vec![1, 2]))
            .unwrap();
        let selected: Vec<(usize, usize)> = view
            .layout_snapshot()
            .unwrap()
            .into_iter()
            .filter(|cell| cell.selected)
            .map(|cell| (cell.row, cell.column))
            .collect();
        assert_eq!(selected, vec![(0, 1), (1, 1), (1, 2)]);
    }

    #[test]
    fn direct_invalid_mutation_reports_projection_diagnostic() {
        let mut view = TensorView::try_new(snapshot()).unwrap();
        view.snapshot.shape[1] = 4;
        let mut ctx = ProjectionCtx::new(Default::default());
        view.project(&mut ctx);
        assert!(ctx.primitives.is_empty());
        assert!(matches!(
            ctx.diagnostics[0],
            ValidationError::LengthMismatch { .. }
        ));
        assert_eq!(view.local_bounds(), Bounds::default());
    }

    #[test]
    fn rank_two_is_required_only_by_the_view() {
        let vector = TensorSnapshot::try_new(
            "embedding",
            vec![2],
            vec![0.1, 0.2],
            vec![TensorAxis::new("feature", "Features", vec!["x", "y"])],
        )
        .unwrap();
        assert!(matches!(
            TensorView::try_new(vector),
            Err(ValidationError::RankMismatch {
                expected: 2,
                actual: 1,
                ..
            })
        ));
    }

    #[test]
    fn transition_layout_matches_reordered_elements_and_fades_new_cells() {
        let source = snapshot_with_values("attention", vec![0.0, 2.0, 4.0, 6.0]);
        let target = TensorSnapshot::try_new(
            "attention",
            vec![2, 3],
            vec![12.0, 10.0, 14.0, 18.0, 16.0, 20.0],
            vec![
                TensorAxis::with_elements("query", "Queries", [("q0", "A"), ("q1", "B")]),
                TensorAxis::with_elements("key", "Keys", [("k1", "B"), ("k0", "A"), ("k2", "C")]),
            ],
        )
        .unwrap();
        let mut view = TensorView::try_new(source.clone()).unwrap();
        let source_center = view
            .layout_snapshot()
            .unwrap()
            .into_iter()
            .find(|cell| cell.id == source.element_id(&[0, 1]).unwrap())
            .unwrap()
            .center;
        view.transition = Some(TensorTransitionFrame {
            source,
            target,
            progress: 0.5,
        });

        let cells = view.layout_snapshot().unwrap();
        let moved = cells
            .iter()
            .find(|cell| cell.id.coordinates[1].element_id == "k1" && cell.row == 0)
            .unwrap();
        assert_eq!(moved.value, 7.0);
        assert!(moved.center.x < source_center.x);
        assert_eq!(moved.opacity, 1.0);
        let added = cells
            .iter()
            .find(|cell| cell.id.coordinates[1].element_id == "k2" && cell.row == 0)
            .unwrap();
        assert_eq!(added.opacity, 0.5);
    }

    #[test]
    fn transition_layout_fades_removed_cells() {
        let source = snapshot_with_values("attention", vec![0.0, 2.0, 4.0, 6.0]);
        let target = TensorSnapshot::try_new(
            "attention",
            vec![1, 2],
            vec![10.0, 12.0],
            vec![
                TensorAxis::with_elements("query", "Queries", [("q0", "A")]),
                TensorAxis::with_elements("key", "Keys", [("k0", "A"), ("k1", "B")]),
            ],
        )
        .unwrap();
        let mut view = TensorView::try_new(source.clone()).unwrap();
        view.transition = Some(TensorTransitionFrame {
            source,
            target,
            progress: 0.25,
        });

        let cells = view.layout_snapshot().unwrap();
        let removed: Vec<&TensorCellLayout> = cells
            .iter()
            .filter(|cell| cell.id.coordinates[0].element_id == "q1")
            .collect();
        assert_eq!(removed.len(), 2);
        assert!(removed.iter().all(|cell| cell.opacity == 0.75));
    }

    #[test]
    fn tensor_transition_is_deterministic_across_repeated_seeks() {
        let source = snapshot_with_values("attention", vec![0.0, 2.0, 4.0, 6.0]);
        let target = snapshot_with_values("attention", vec![10.0, 12.0, 14.0, 16.0]);
        let mut scene = Scene::new();
        let id = scene.add_tattva(TensorView::try_new(source.clone()).unwrap(), Vec3::ZERO);
        let mut timeline = Timeline::new();
        timeline
            .animate(id)
            .at(1.0)
            .for_duration(2.0)
            .ease(Ease::Linear)
            .tensor_to(target.clone())
            .spawn();
        scene.play(timeline).unwrap();

        for time in [2.0, 0.0, 2.0, 3.0] {
            scene.seek_to(time).unwrap();
            let tensor = scene.get_tattva_typed::<TensorView>(id).unwrap();
            if time == 0.0 {
                assert_eq!(tensor.state.snapshot.values, source.values);
            } else if time == 3.0 {
                assert_eq!(tensor.state.snapshot.values, target.values);
            } else {
                assert_eq!(tensor.state.snapshot.values, vec![5.0, 7.0, 9.0, 11.0]);
            }
        }
    }

    #[test]
    fn chained_tensor_transitions_reconstruct_their_starting_snapshots() {
        let first = snapshot_with_values("attention", vec![0.0; 4]);
        let second = snapshot_with_values("attention", vec![10.0; 4]);
        let third = snapshot_with_values("attention", vec![20.0; 4]);
        let mut scene = Scene::new();
        let id = scene.add_tattva(TensorView::try_new(first).unwrap(), Vec3::ZERO);
        let mut timeline = Timeline::new();
        timeline
            .animate(id)
            .at(0.0)
            .for_duration(1.0)
            .ease(Ease::Linear)
            .tensor_to(second)
            .spawn();
        timeline
            .animate(id)
            .at(1.0)
            .for_duration(1.0)
            .ease(Ease::Linear)
            .tensor_to(third)
            .spawn();
        scene.play(timeline).unwrap();

        for time in [1.5, 0.5, 1.5] {
            scene.seek_to(time).unwrap();
            let values = &scene
                .get_tattva_typed::<TensorView>(id)
                .unwrap()
                .state
                .snapshot
                .values;
            assert!(
                values
                    .iter()
                    .all(|value| (*value - time * 10.0).abs() < 1e-6)
            );
        }
    }

    #[test]
    fn later_overlapping_tensor_transition_owns_the_rendered_state() {
        let first = snapshot_with_values("attention", vec![0.0; 4]);
        let second = snapshot_with_values("attention", vec![10.0; 4]);
        let third = snapshot_with_values("attention", vec![20.0; 4]);
        let mut scene = Scene::new();
        let id = scene.add_tattva(TensorView::try_new(first).unwrap(), Vec3::ZERO);
        let mut timeline = Timeline::new();
        timeline
            .animate(id)
            .at(0.0)
            .for_duration(2.0)
            .ease(Ease::Linear)
            .tensor_to(second)
            .spawn();
        timeline
            .animate(id)
            .at(1.0)
            .for_duration(2.0)
            .ease(Ease::Linear)
            .tensor_to(third)
            .spawn();
        scene.play(timeline).unwrap();

        for time in [2.0, 0.5, 2.0, 3.0] {
            scene.seek_to(time).unwrap();
            let value = scene
                .get_tattva_typed::<TensorView>(id)
                .unwrap()
                .state
                .snapshot
                .values[0];
            let expected = match time {
                0.5 => 2.5,
                2.0 => 12.5,
                _ => 20.0,
            };
            assert!((value - expected).abs() < 1e-6);
        }
    }

    #[test]
    fn incompatible_tensor_transition_is_rejected_when_scene_is_installed() {
        let source = snapshot_with_values("source", vec![0.0; 4]);
        let target = snapshot_with_values("different", vec![1.0; 4]);
        let mut scene = Scene::new();
        let id = scene.add_tattva(TensorView::try_new(source).unwrap(), Vec3::ZERO);
        let mut timeline = Timeline::new();
        timeline.animate(id).tensor_to(target).spawn();

        assert!(matches!(
            scene.play(timeline),
            Err(ValidationError::Incompatible {
                component: "TensorTransition",
                field: "tensor id",
                ..
            })
        ));
    }

    #[test]
    fn tensor_transition_rejects_a_non_tensor_target() {
        let target = snapshot_with_values("attention", vec![1.0; 4]);
        let mut scene = Scene::new();
        let id = scene.add_tattva(Square::new(1.0, Vec4::ONE), Vec3::ZERO);
        let mut timeline = Timeline::new();
        timeline.animate(id).tensor_to(target).spawn();

        assert!(matches!(
            scene.play(timeline),
            Err(ValidationError::TargetTypeMismatch {
                component: "TensorTransition",
                expected: "TensorView",
                ..
            })
        ));
    }

    #[test]
    fn semantic_selection_transition_crossfades_and_seeks() {
        let view = TensorView::try_new(snapshot_with_values("attention", vec![0.0; 4]))
            .unwrap()
            .with_selection(TensorSelector::axis_element("query", "q0"))
            .unwrap();
        let mut scene = Scene::new();
        let id = scene.add_tattva(view, Vec3::ZERO);
        let mut timeline = Timeline::new();
        timeline
            .animate(id)
            .at(0.0)
            .for_duration(1.0)
            .ease(Ease::Linear)
            .tensor_select(vec![TensorSelector::axis_element("query", "q1")])
            .spawn();
        scene.play(timeline).unwrap();

        for time in [0.5, 0.0, 0.5, 1.0] {
            scene.seek_to(time).unwrap();
            let cells = scene
                .get_tattva_typed::<TensorView>(id)
                .unwrap()
                .state
                .layout_snapshot()
                .unwrap();
            let q0 = cells
                .iter()
                .find(|cell| cell.row == 0 && cell.column == 0)
                .unwrap()
                .selection_strength;
            let q1 = cells
                .iter()
                .find(|cell| cell.row == 1 && cell.column == 0)
                .unwrap()
                .selection_strength;
            if time == 0.0 {
                assert_eq!((q0, q1), (1.0, 0.0));
            } else if time == 1.0 {
                assert_eq!((q0, q1), (0.0, 1.0));
            } else {
                assert_eq!((q0, q1), (0.5, 0.5));
            }
        }
    }
}
