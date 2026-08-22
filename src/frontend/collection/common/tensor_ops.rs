use std::collections::{HashMap, HashSet};

use super::tensor::{TensorAxis, TensorElementId, TensorSnapshot};
use crate::validation::ValidationError;

const COMPONENT: &str = "TensorOperation";

/// A built-in binary operation applied element by element after semantic-axis alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TensorElementwiseOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Minimum,
    Maximum,
}

/// Non-affine normalization applied along one named tensor axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TensorNormalization {
    LayerNorm,
    RmsNorm,
}

/// One deterministic choice from a categorical tensor slice.
#[derive(Debug, Clone, PartialEq)]
pub struct TensorSample {
    pub element_id: TensorElementId,
    pub probability: f32,
}

/// Semantic restrictions used to derive a lower-rank tensor without flattening its axes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TensorSlice {
    selections: Vec<TensorAxisSlice>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TensorAxisSlice {
    At {
        axis_id: String,
        element_id: String,
    },
    Elements {
        axis_id: String,
        element_ids: Vec<String>,
    },
}

impl TensorSlice {
    pub fn new() -> Self {
        Self::default()
    }

    /// Fixes an axis to one semantic element and removes that axis from the result.
    pub fn at(mut self, axis_id: impl Into<String>, element_id: impl Into<String>) -> Self {
        self.selections.push(TensorAxisSlice::At {
            axis_id: axis_id.into(),
            element_id: element_id.into(),
        });
        self
    }

    /// Retains selected semantic elements in the supplied order and keeps the axis in the result.
    pub fn elements<I, S>(mut self, axis_id: impl Into<String>, element_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.selections.push(TensorAxisSlice::Elements {
            axis_id: axis_id.into(),
            element_ids: element_ids.into_iter().map(Into::into).collect(),
        });
        self
    }
}

impl TensorElementwiseOp {
    fn apply(self, lhs: f32, rhs: f32) -> f32 {
        match self {
            Self::Add => lhs + rhs,
            Self::Subtract => lhs - rhs,
            Self::Multiply => lhs * rhs,
            Self::Divide => lhs / rhs,
            Self::Minimum => lhs.min(rhs),
            Self::Maximum => lhs.max(rhs),
        }
    }
}

impl TensorSnapshot {
    /// Produces a semantic slice while retaining the identity and order of every remaining axis.
    ///
    /// Fixed axes are removed. Subset axes remain and may explicitly reorder their elements.
    /// Axes omitted from the slice are retained unchanged.
    pub fn try_slice(
        &self,
        output_id: impl Into<String>,
        slice: &TensorSlice,
    ) -> Result<Self, ValidationError> {
        self.validate()?;
        let selections = slice.validate(self)?;
        let mut output_axes = Vec::new();
        let mut source_positions = Vec::with_capacity(self.rank());

        for (source_axis_index, axis) in self.axes.iter().enumerate() {
            match selections.get(axis.id.as_str()) {
                Some(ResolvedAxisSlice::At(index)) => {
                    source_positions.push(SourceAxisPositions::Fixed(*index));
                }
                Some(ResolvedAxisSlice::Elements(indices)) => {
                    let mut output_axis = axis.clone();
                    output_axis.element_ids = indices
                        .iter()
                        .map(|&index| axis.element_ids[index].clone())
                        .collect();
                    output_axis.element_labels = indices
                        .iter()
                        .map(|&index| axis.element_labels[index].clone())
                        .collect();
                    let output_axis_index = output_axes.len();
                    output_axes.push(output_axis);
                    source_positions.push(SourceAxisPositions::Retained {
                        output_axis_index,
                        source_indices: indices.clone(),
                    });
                }
                None => {
                    let output_axis_index = output_axes.len();
                    output_axes.push(axis.clone());
                    source_positions.push(SourceAxisPositions::Retained {
                        output_axis_index,
                        source_indices: (0..self.shape[source_axis_index]).collect(),
                    });
                }
            }
        }

        if output_axes.is_empty() {
            return Err(ValidationError::Empty {
                component: COMPONENT,
                field: "slice retained axes",
            });
        }

        let output_shape: Vec<usize> = output_axes
            .iter()
            .map(|axis| axis.element_ids.len())
            .collect();
        let output_size = checked_shape_size(&output_shape, "slice output shape")?;
        let mut values = Vec::with_capacity(output_size);
        for output_flat_index in 0..output_size {
            let output_indices = unravel_index(output_flat_index, &output_shape);
            let source_indices: Vec<usize> = source_positions
                .iter()
                .map(|positions| match positions {
                    SourceAxisPositions::Fixed(index) => *index,
                    SourceAxisPositions::Retained {
                        output_axis_index,
                        source_indices,
                    } => source_indices[output_indices[*output_axis_index]],
                })
                .collect();
            values.push(self.values[flat_index(&source_indices, &self.shape)]);
        }
        Self::try_new(output_id, output_shape, values, output_axes)
    }

    /// Projects a higher-rank tensor onto two explicitly named semantic axes.
    ///
    /// The slice must fix every other axis. Row and column axes are returned in the requested
    /// order, regardless of their storage order in the source snapshot.
    pub fn try_project_2d(
        &self,
        output_id: impl Into<String>,
        row_axis_id: &str,
        column_axis_id: &str,
        slice: &TensorSlice,
    ) -> Result<Self, ValidationError> {
        self.validate()?;
        if row_axis_id == column_axis_id {
            return Err(ValidationError::DuplicateIdentifier {
                component: COMPONENT,
                field: "2D projection axes",
                value: row_axis_id.to_string(),
            });
        }
        let output_id = output_id.into();
        let projected = self.try_slice(output_id.clone(), slice)?;
        if projected.rank() != 2 {
            return Err(ValidationError::RankMismatch {
                component: COMPONENT,
                field: "2D projection retained axes",
                expected: 2,
                actual: projected.rank(),
            });
        }
        let row = projected.axis_index(row_axis_id, "2D projection row axis")?;
        let column = projected.axis_index(column_axis_id, "2D projection column axis")?;
        if row == 0 && column == 1 {
            return Ok(projected);
        }
        if row == 1 && column == 0 {
            return projected.try_transpose_2d(output_id);
        }
        Err(ValidationError::Incompatible {
            component: COMPONENT,
            field: "2D projection axes",
            reason: "row and column must name both retained axes".to_string(),
        })
    }

    /// Samples categorical slices using caller-supplied unit-interval values.
    ///
    /// Supplying the random variates makes sampling reproducible and independent of renderer frame
    /// rate. Values along the sampled axis must be non-negative and have a positive sum.
    pub fn try_sample_categorical(
        &self,
        axis_id: &str,
        unit_samples: &[f32],
    ) -> Result<Vec<TensorSample>, ValidationError> {
        self.validate()?;
        let axis = self.axis_index(axis_id, "categorical axis")?;
        let slice_count = self.values.len() / self.shape[axis];
        if unit_samples.len() != slice_count {
            return Err(ValidationError::LengthMismatch {
                component: COMPONENT,
                field: "categorical samples",
                expected: slice_count,
                actual: unit_samples.len(),
            });
        }
        for &sample in unit_samples {
            if !sample.is_finite() {
                return Err(ValidationError::NonFinite {
                    component: COMPONENT,
                    field: "categorical sample",
                    value: sample,
                });
            }
            if !(0.0..1.0).contains(&sample) {
                return Err(ValidationError::OutOfRange {
                    component: COMPONENT,
                    field: "categorical sample",
                    minimum: 0.0,
                    maximum: 1.0,
                    value: sample,
                });
            }
        }

        let mut slice_index = 0;
        self.select_along_axis(axis, |values| {
            let sample = unit_samples[slice_index];
            slice_index += 1;
            let sum = values.iter().sum::<f32>();
            let mut cumulative = 0.0;
            for (index, &value) in values.iter().enumerate() {
                cumulative += value / sum;
                if sample < cumulative || index + 1 == values.len() {
                    return index;
                }
            }
            values.len() - 1
        })
    }

    /// Reshapes row-major values onto explicit semantic axes.
    ///
    /// Supplying the axes makes the identity introduced by a reshape explicit instead of
    /// manufacturing anonymous dimensions. The new axes must describe exactly the same number of
    /// values as the source snapshot.
    pub fn try_reshape(
        &self,
        output_id: impl Into<String>,
        axes: Vec<TensorAxis>,
    ) -> Result<Self, ValidationError> {
        self.validate()?;
        let shape: Vec<usize> = axes.iter().map(|axis| axis.element_ids.len()).collect();
        let output_size = checked_shape_size(&shape, "reshape output shape")?;
        if output_size != self.values.len() {
            return Err(ValidationError::LengthMismatch {
                component: COMPONENT,
                field: "reshape output values",
                expected: self.values.len(),
                actual: output_size,
            });
        }
        Self::try_new(output_id, shape, self.values.clone(), axes)
    }

    /// Splits a snapshot along a named axis while retaining each element's semantic identity.
    pub fn try_split<S: AsRef<str>>(
        &self,
        axis_id: &str,
        lengths: &[usize],
        output_ids: &[S],
    ) -> Result<Vec<Self>, ValidationError> {
        self.validate()?;
        if lengths.is_empty() {
            return Err(ValidationError::Empty {
                component: COMPONENT,
                field: "split lengths",
            });
        }
        if output_ids.len() != lengths.len() {
            return Err(ValidationError::LengthMismatch {
                component: COMPONENT,
                field: "split output ids",
                expected: lengths.len(),
                actual: output_ids.len(),
            });
        }
        let axis_index = self.axis_index(axis_id, "split axis")?;
        let mut total = 0_usize;
        for &length in lengths {
            if length == 0 {
                return Err(ValidationError::CountTooSmall {
                    component: COMPONENT,
                    field: "split length",
                    minimum: 1,
                    actual: 0,
                });
            }
            total = total
                .checked_add(length)
                .ok_or(ValidationError::ShapeOverflow {
                    component: COMPONENT,
                    field: "split lengths",
                })?;
        }
        if total != self.shape[axis_index] {
            return Err(ValidationError::LengthMismatch {
                component: COMPONENT,
                field: "split lengths",
                expected: self.shape[axis_index],
                actual: total,
            });
        }

        let mut offset = 0;
        let mut outputs = Vec::with_capacity(lengths.len());
        for (&length, output_id) in lengths.iter().zip(output_ids) {
            let mut axes = self.axes.clone();
            axes[axis_index].element_ids =
                self.axes[axis_index].element_ids[offset..(offset + length)].to_vec();
            axes[axis_index].element_labels =
                self.axes[axis_index].element_labels[offset..(offset + length)].to_vec();
            let mut shape = self.shape.clone();
            shape[axis_index] = length;
            let output_size = checked_shape_size(&shape, "split output shape")?;
            let mut values = Vec::with_capacity(output_size);
            for output_flat_index in 0..output_size {
                let mut indices = unravel_index(output_flat_index, &shape);
                indices[axis_index] += offset;
                values.push(self.values[flat_index(&indices, &self.shape)]);
            }
            outputs.push(Self::try_new(output_id.as_ref(), shape, values, axes)?);
            offset += length;
        }
        Ok(outputs)
    }

    /// Merges compatible snapshots along a named axis.
    ///
    /// All non-merged axes must have identical metadata. Elements on the merged axis retain their
    /// IDs and must remain unique in the resulting snapshot.
    pub fn try_merge(
        snapshots: &[Self],
        axis_id: &str,
        output_id: impl Into<String>,
    ) -> Result<Self, ValidationError> {
        let Some(first) = snapshots.first() else {
            return Err(ValidationError::Empty {
                component: COMPONENT,
                field: "merge inputs",
            });
        };
        first.validate()?;
        let axis_index = first.axis_index(axis_id, "merge axis")?;
        let mut merged_axis = first.axes[axis_index].clone();
        let mut merged_length = 0_usize;
        for (input_index, snapshot) in snapshots.iter().enumerate() {
            snapshot.validate()?;
            if snapshot.rank() != first.rank() {
                return Err(ValidationError::RankMismatch {
                    component: COMPONENT,
                    field: "merge input",
                    expected: first.rank(),
                    actual: snapshot.rank(),
                });
            }
            for axis in 0..first.rank() {
                if snapshot.axes[axis].id != first.axes[axis].id {
                    return Err(ValidationError::Incompatible {
                        component: COMPONENT,
                        field: "merge axis order",
                        reason: format!(
                            "input {input_index} has axis '{}' where '{}' was expected",
                            snapshot.axes[axis].id, first.axes[axis].id
                        ),
                    });
                }
                if axis != axis_index && snapshot.axes[axis] != first.axes[axis] {
                    return Err(ValidationError::Incompatible {
                        component: COMPONENT,
                        field: "merge non-merged axes",
                        reason: format!(
                            "input {input_index} has incompatible axis '{}'",
                            snapshot.axes[axis].id
                        ),
                    });
                }
            }
            merged_length = merged_length
                .checked_add(snapshot.shape[axis_index])
                .ok_or(ValidationError::ShapeOverflow {
                    component: COMPONENT,
                    field: "merge output shape",
                })?;
            if input_index > 0 {
                merged_axis
                    .element_ids
                    .extend(snapshot.axes[axis_index].element_ids.iter().cloned());
                merged_axis
                    .element_labels
                    .extend(snapshot.axes[axis_index].element_labels.iter().cloned());
            }
        }

        let mut shape = first.shape.clone();
        shape[axis_index] = merged_length;
        let output_size = checked_shape_size(&shape, "merge output shape")?;
        let mut values = Vec::with_capacity(output_size);
        for output_flat_index in 0..output_size {
            let output_indices = unravel_index(output_flat_index, &shape);
            let mut merged_axis_index = output_indices[axis_index];
            let mut source = first;
            for snapshot in snapshots {
                if merged_axis_index < snapshot.shape[axis_index] {
                    source = snapshot;
                    break;
                }
                merged_axis_index -= snapshot.shape[axis_index];
            }
            let mut source_indices = output_indices;
            source_indices[axis_index] = merged_axis_index;
            values.push(source.values[flat_index(&source_indices, &source.shape)]);
        }
        let mut axes = first.axes.clone();
        axes[axis_index] = merged_axis;
        Self::try_new(output_id, shape, values, axes)
    }

    /// Applies a binary operation after aligning the right operand by semantic axis and element ID.
    ///
    /// The left snapshot defines the output shape. Missing right-hand axes and singleton axes are
    /// broadcast; populated axes may be stored in a different order but must contain the same
    /// semantic element IDs.
    pub fn try_elementwise(
        &self,
        rhs: &Self,
        output_id: impl Into<String>,
        operation: TensorElementwiseOp,
    ) -> Result<Self, ValidationError> {
        self.validate()?;
        rhs.validate()?;
        let lhs_axis_positions: HashMap<&str, usize> = self
            .axes
            .iter()
            .enumerate()
            .map(|(index, axis)| (axis.id.as_str(), index))
            .collect();
        let mut rhs_to_lhs = Vec::with_capacity(rhs.rank());
        let mut rhs_element_positions = Vec::with_capacity(rhs.rank());
        for (rhs_axis_index, rhs_axis) in rhs.axes.iter().enumerate() {
            let Some(&lhs_axis_index) = lhs_axis_positions.get(rhs_axis.id.as_str()) else {
                return Err(ValidationError::UnknownIdentifier {
                    component: COMPONENT,
                    field: "elementwise right axis",
                    value: rhs_axis.id.clone(),
                });
            };
            rhs_to_lhs.push(lhs_axis_index);
            if rhs.shape[rhs_axis_index] == 1 {
                rhs_element_positions.push(HashMap::new());
                continue;
            }
            if rhs.shape[rhs_axis_index] != self.shape[lhs_axis_index] {
                return Err(ValidationError::LengthMismatch {
                    component: COMPONENT,
                    field: "elementwise axis",
                    expected: self.shape[lhs_axis_index],
                    actual: rhs.shape[rhs_axis_index],
                });
            }
            let positions: HashMap<&str, usize> = rhs_axis
                .element_ids
                .iter()
                .enumerate()
                .map(|(index, id)| (id.as_str(), index))
                .collect();
            for lhs_element_id in &self.axes[lhs_axis_index].element_ids {
                if !positions.contains_key(lhs_element_id.as_str()) {
                    return Err(ValidationError::Incompatible {
                        component: COMPONENT,
                        field: "elementwise axis elements",
                        reason: format!(
                            "right axis '{}' is missing element '{}'",
                            rhs_axis.id, lhs_element_id
                        ),
                    });
                }
            }
            rhs_element_positions.push(positions);
        }

        let mut values = Vec::with_capacity(self.values.len());
        for (lhs_flat_index, &lhs_value) in self.values.iter().enumerate() {
            let lhs_indices = unravel_index(lhs_flat_index, &self.shape);
            let rhs_indices: Vec<usize> = rhs
                .axes
                .iter()
                .enumerate()
                .map(|(rhs_axis_index, _)| {
                    if rhs.shape[rhs_axis_index] == 1 {
                        0
                    } else {
                        let lhs_axis_index = rhs_to_lhs[rhs_axis_index];
                        let lhs_element_id =
                            &self.axes[lhs_axis_index].element_ids[lhs_indices[lhs_axis_index]];
                        rhs_element_positions[rhs_axis_index][lhs_element_id.as_str()]
                    }
                })
                .collect();
            let rhs_value = rhs.values[flat_index(&rhs_indices, &rhs.shape)];
            let value = operation.apply(lhs_value, rhs_value);
            if !value.is_finite() {
                return Err(ValidationError::NonFinite {
                    component: COMPONENT,
                    field: "elementwise result",
                    value,
                });
            }
            values.push(value);
        }
        Self::try_new(output_id, self.shape.clone(), values, self.axes.clone())
    }

    /// Transposes a rank-2 snapshot while preserving semantic axis metadata.
    pub fn try_transpose_2d(&self, output_id: impl Into<String>) -> Result<Self, ValidationError> {
        self.validate()?;
        self.require_rank_2("transpose input")?;
        let rows = self.shape[0];
        let columns = self.shape[1];
        let mut values = vec![0.0; self.values.len()];
        for row in 0..rows {
            for column in 0..columns {
                values[column * rows + row] = self.values[row * columns + column];
            }
        }
        Self::try_new(
            output_id,
            vec![columns, rows],
            values,
            vec![self.axes[1].clone(), self.axes[0].clone()],
        )
    }

    /// Multiplies two rank-2 snapshots, aligning the contracted axis by element identity.
    pub fn try_matmul(
        &self,
        rhs: &Self,
        output_id: impl Into<String>,
    ) -> Result<Self, ValidationError> {
        self.validate()?;
        rhs.validate()?;
        self.require_rank_2("left matrix")?;
        rhs.require_rank_2("right matrix")?;

        let lhs_contract = &self.axes[1];
        let rhs_contract = &rhs.axes[0];
        if lhs_contract.id != rhs_contract.id {
            return Err(ValidationError::Incompatible {
                component: COMPONENT,
                field: "matmul contraction axis",
                reason: format!(
                    "left axis '{}' does not match right axis '{}'",
                    lhs_contract.id, rhs_contract.id
                ),
            });
        }
        if lhs_contract.element_ids.len() != rhs_contract.element_ids.len() {
            return Err(ValidationError::LengthMismatch {
                component: COMPONENT,
                field: "matmul contraction axis",
                expected: lhs_contract.element_ids.len(),
                actual: rhs_contract.element_ids.len(),
            });
        }

        let rhs_positions: HashMap<&str, usize> = rhs_contract
            .element_ids
            .iter()
            .enumerate()
            .map(|(index, id)| (id.as_str(), index))
            .collect();
        let mut aligned_rhs_rows = Vec::with_capacity(lhs_contract.element_ids.len());
        for element_id in &lhs_contract.element_ids {
            let Some(&rhs_row) = rhs_positions.get(element_id.as_str()) else {
                return Err(ValidationError::Incompatible {
                    component: COMPONENT,
                    field: "matmul contraction elements",
                    reason: format!("right matrix is missing element '{element_id}'"),
                });
            };
            aligned_rhs_rows.push(rhs_row);
        }

        let rows = self.shape[0];
        let contracted = self.shape[1];
        let columns = rhs.shape[1];
        let Some(output_size) = rows.checked_mul(columns) else {
            return Err(ValidationError::ShapeOverflow {
                component: COMPONENT,
                field: "matmul output shape",
            });
        };
        let mut values = vec![0.0; output_size];
        for row in 0..rows {
            for column in 0..columns {
                let mut sum = 0.0;
                for (lhs_column, &rhs_row) in aligned_rhs_rows.iter().enumerate() {
                    sum += self.values[row * contracted + lhs_column]
                        * rhs.values[rhs_row * columns + column];
                }
                values[row * columns + column] = sum;
            }
        }

        Self::try_new(
            output_id,
            vec![rows, columns],
            values,
            vec![self.axes[0].clone(), rhs.axes[1].clone()],
        )
    }

    /// Divides every value by a positive finite scalar while preserving tensor identity.
    pub fn try_scaled(&self, divisor: f32) -> Result<Self, ValidationError> {
        self.validate()?;
        if !divisor.is_finite() {
            return Err(ValidationError::NonFinite {
                component: COMPONENT,
                field: "scale divisor",
                value: divisor,
            });
        }
        if divisor <= 0.0 {
            return Err(ValidationError::NonPositive {
                component: COMPONENT,
                field: "scale divisor",
                value: divisor,
            });
        }
        let mut scaled = self.clone();
        for value in &mut scaled.values {
            *value /= divisor;
        }
        scaled.validate()?;
        Ok(scaled)
    }

    /// Normalizes each slice along a named axis while preserving semantic axes and element IDs.
    ///
    /// This operation is intentionally non-affine. Learned scale and bias tensors can be applied
    /// afterward with [`TensorSnapshot::try_elementwise`].
    pub fn try_normalized(
        &self,
        output_id: impl Into<String>,
        axis_id: &str,
        normalization: TensorNormalization,
        epsilon: f32,
    ) -> Result<Self, ValidationError> {
        self.validate()?;
        let axis = self.axis_index(axis_id, "normalization axis")?;
        if !epsilon.is_finite() {
            return Err(ValidationError::NonFinite {
                component: COMPONENT,
                field: "normalization epsilon",
                value: epsilon,
            });
        }
        if epsilon <= 0.0 {
            return Err(ValidationError::NonPositive {
                component: COMPONENT,
                field: "normalization epsilon",
                value: epsilon,
            });
        }

        let axis_length = self.shape[axis];
        let inner = self.shape[(axis + 1)..].iter().product::<usize>();
        let outer = self.values.len() / (axis_length * inner);
        let mut values = self.values.clone();
        for outer_index in 0..outer {
            for inner_index in 0..inner {
                let index = |axis_index: usize| {
                    outer_index * axis_length * inner + axis_index * inner + inner_index
                };
                let mean = (0..axis_length)
                    .map(|axis_index| self.values[index(axis_index)])
                    .sum::<f32>()
                    / axis_length as f32;
                let square_mean = (0..axis_length)
                    .map(|axis_index| {
                        let value = self.values[index(axis_index)];
                        match normalization {
                            TensorNormalization::LayerNorm => (value - mean).powi(2),
                            TensorNormalization::RmsNorm => value.powi(2),
                        }
                    })
                    .sum::<f32>()
                    / axis_length as f32;
                let divisor = (square_mean + epsilon).sqrt();
                for axis_index in 0..axis_length {
                    let input = self.values[index(axis_index)];
                    values[index(axis_index)] = match normalization {
                        TensorNormalization::LayerNorm => (input - mean) / divisor,
                        TensorNormalization::RmsNorm => input / divisor,
                    };
                }
            }
        }
        Self::try_new(output_id, self.shape.clone(), values, self.axes.clone())
    }

    /// Replaces entries above the rank-2 causal diagonal with a finite mask value.
    pub fn try_causal_masked(&self, masked_value: f32) -> Result<Self, ValidationError> {
        self.validate()?;
        self.require_rank_2("causal mask input")?;
        if !masked_value.is_finite() {
            return Err(ValidationError::NonFinite {
                component: COMPONENT,
                field: "masked value",
                value: masked_value,
            });
        }
        let rows = self.shape[0];
        let columns = self.shape[1];
        let mut masked = self.clone();
        for row in 0..rows {
            for column in (row + 1)..columns {
                masked.values[row * columns + column] = masked_value;
            }
        }
        Ok(masked)
    }

    /// Applies a numerically stable softmax along a named semantic axis.
    pub fn try_softmax(&self, axis_id: &str) -> Result<Self, ValidationError> {
        self.validate()?;
        let Some(axis) = self.axes.iter().position(|axis| axis.id == axis_id) else {
            return Err(ValidationError::UnknownIdentifier {
                component: COMPONENT,
                field: "softmax axis",
                value: axis_id.to_string(),
            });
        };
        let axis_length = self.shape[axis];
        let inner = self.shape[(axis + 1)..].iter().product::<usize>();
        let outer = self.values.len() / (axis_length * inner);
        let mut output = self.clone();

        for outer_index in 0..outer {
            for inner_index in 0..inner {
                let index = |axis_index: usize| {
                    outer_index * axis_length * inner + axis_index * inner + inner_index
                };
                let max_value = (0..axis_length)
                    .map(|axis_index| self.values[index(axis_index)])
                    .fold(f32::NEG_INFINITY, f32::max);
                let sum = (0..axis_length)
                    .map(|axis_index| (self.values[index(axis_index)] - max_value).exp())
                    .sum::<f32>();
                for axis_index in 0..axis_length {
                    output.values[index(axis_index)] =
                        (self.values[index(axis_index)] - max_value).exp() / sum;
                }
            }
        }
        output.validate()?;
        Ok(output)
    }

    fn require_rank_2(&self, field: &'static str) -> Result<(), ValidationError> {
        if self.rank() != 2 {
            return Err(ValidationError::RankMismatch {
                component: COMPONENT,
                field,
                expected: 2,
                actual: self.rank(),
            });
        }
        Ok(())
    }

    fn axis_index(&self, axis_id: &str, field: &'static str) -> Result<usize, ValidationError> {
        self.axes
            .iter()
            .position(|axis| axis.id == axis_id)
            .ok_or_else(|| ValidationError::UnknownIdentifier {
                component: COMPONENT,
                field,
                value: axis_id.to_string(),
            })
    }

    fn select_along_axis<F>(
        &self,
        axis: usize,
        mut select: F,
    ) -> Result<Vec<TensorSample>, ValidationError>
    where
        F: FnMut(&[f32]) -> usize,
    {
        let axis_length = self.shape[axis];
        let inner = self.shape[(axis + 1)..].iter().product::<usize>();
        let outer = self.values.len() / (axis_length * inner);
        let mut samples = Vec::with_capacity(outer * inner);
        let mut slice_values = vec![0.0; axis_length];
        for outer_index in 0..outer {
            for inner_index in 0..inner {
                for (axis_index, value) in slice_values.iter_mut().enumerate() {
                    let flat = outer_index * axis_length * inner + axis_index * inner + inner_index;
                    *value = self.values[flat];
                }
                if slice_values.iter().any(|value| *value < 0.0) {
                    return Err(ValidationError::Incompatible {
                        component: COMPONENT,
                        field: "categorical values",
                        reason: "values must be non-negative".to_string(),
                    });
                }
                let sum = slice_values.iter().sum::<f32>();
                if sum <= 0.0 {
                    return Err(ValidationError::NonPositive {
                        component: COMPONENT,
                        field: "categorical slice sum",
                        value: sum,
                    });
                }
                let selected_axis_index = select(&slice_values);
                let flat =
                    outer_index * axis_length * inner + selected_axis_index * inner + inner_index;
                let indices = unravel_index(flat, &self.shape);
                samples.push(TensorSample {
                    element_id: self
                        .element_id(&indices)
                        .expect("selected tensor coordinate is valid"),
                    probability: self.values[flat] / sum,
                });
            }
        }
        Ok(samples)
    }
}

enum ResolvedAxisSlice {
    At(usize),
    Elements(Vec<usize>),
}

enum SourceAxisPositions {
    Fixed(usize),
    Retained {
        output_axis_index: usize,
        source_indices: Vec<usize>,
    },
}

impl TensorSlice {
    fn validate<'a>(
        &self,
        snapshot: &'a TensorSnapshot,
    ) -> Result<HashMap<&'a str, ResolvedAxisSlice>, ValidationError> {
        let mut resolved = HashMap::with_capacity(self.selections.len());
        for selection in &self.selections {
            let (axis_id, element_ids, fixed) = match selection {
                TensorAxisSlice::At {
                    axis_id,
                    element_id,
                } => (axis_id, std::slice::from_ref(element_id), true),
                TensorAxisSlice::Elements {
                    axis_id,
                    element_ids,
                } => (axis_id, element_ids.as_slice(), false),
            };
            let Some(axis) = snapshot.axes.iter().find(|axis| axis.id == *axis_id) else {
                return Err(ValidationError::UnknownIdentifier {
                    component: COMPONENT,
                    field: "slice axis",
                    value: axis_id.clone(),
                });
            };
            if resolved.contains_key(axis.id.as_str()) {
                return Err(ValidationError::DuplicateIdentifier {
                    component: COMPONENT,
                    field: "slice axes",
                    value: axis_id.clone(),
                });
            }
            if element_ids.is_empty() {
                return Err(ValidationError::Empty {
                    component: COMPONENT,
                    field: "slice axis elements",
                });
            }
            let mut unique = HashSet::with_capacity(element_ids.len());
            let mut indices = Vec::with_capacity(element_ids.len());
            for element_id in element_ids {
                if !unique.insert(element_id.as_str()) {
                    return Err(ValidationError::DuplicateIdentifier {
                        component: COMPONENT,
                        field: "slice axis elements",
                        value: element_id.clone(),
                    });
                }
                let Some(index) = axis
                    .element_ids
                    .iter()
                    .position(|candidate| candidate == element_id)
                else {
                    return Err(ValidationError::UnknownIdentifier {
                        component: COMPONENT,
                        field: "slice axis element",
                        value: element_id.clone(),
                    });
                };
                indices.push(index);
            }
            resolved.insert(
                axis.id.as_str(),
                if fixed {
                    ResolvedAxisSlice::At(indices[0])
                } else {
                    ResolvedAxisSlice::Elements(indices)
                },
            );
        }
        Ok(resolved)
    }
}

fn checked_shape_size(shape: &[usize], field: &'static str) -> Result<usize, ValidationError> {
    if shape.is_empty() {
        return Err(ValidationError::Empty {
            component: COMPONENT,
            field,
        });
    }
    shape.iter().try_fold(1_usize, |size, dimension| {
        size.checked_mul(*dimension)
            .ok_or(ValidationError::ShapeOverflow {
                component: COMPONENT,
                field,
            })
    })
}

fn unravel_index(mut flat: usize, shape: &[usize]) -> Vec<usize> {
    let mut indices = vec![0; shape.len()];
    for axis in (0..shape.len()).rev() {
        indices[axis] = flat % shape[axis];
        flat /= shape[axis];
    }
    indices
}

fn flat_index(indices: &[usize], shape: &[usize]) -> usize {
    indices
        .iter()
        .zip(shape)
        .fold(0, |flat, (&index, &length)| flat * length + index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::collection::common::tensor::TensorAxis;

    fn axis(id: &str, elements: &[(&str, &str)]) -> TensorAxis {
        TensorAxis::with_elements(id, id, elements.iter().copied())
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 1e-5, "{actual} != {expected}");
    }

    fn rank_four_activations() -> TensorSnapshot {
        TensorSnapshot::try_new(
            "activations",
            vec![2, 2, 2, 3],
            (0..24).map(|value| value as f32).collect(),
            vec![
                axis("batch", &[("b0", "0"), ("b1", "1")]),
                axis("head", &[("h0", "0"), ("h1", "1")]),
                axis("token", &[("t0", "A"), ("t1", "B")]),
                axis("feature", &[("f0", "x"), ("f1", "y"), ("f2", "z")]),
            ],
        )
        .unwrap()
    }

    #[test]
    fn semantic_slice_fixes_axes_and_reorders_retained_elements() {
        let sliced = rank_four_activations()
            .try_slice(
                "activations.slice",
                &TensorSlice::new()
                    .at("batch", "b1")
                    .at("head", "h0")
                    .elements("feature", ["f2", "f0"]),
            )
            .unwrap();

        assert_eq!(sliced.shape, [2, 2]);
        assert_eq!(sliced.values, [14.0, 12.0, 17.0, 15.0]);
        assert_eq!(sliced.axes[0].id, "token");
        assert_eq!(sliced.axes[1].element_ids, ["f2", "f0"]);
    }

    #[test]
    fn two_dimensional_projection_orders_axes_explicitly() {
        let projected = rank_four_activations()
            .try_project_2d(
                "activations.view",
                "feature",
                "token",
                &TensorSlice::new().at("batch", "b1").at("head", "h0"),
            )
            .unwrap();

        assert_eq!(projected.shape, [3, 2]);
        assert_eq!(projected.values, [12.0, 15.0, 13.0, 16.0, 14.0, 17.0]);
        assert_eq!(projected.axes[0].id, "feature");
        assert_eq!(projected.axes[1].id, "token");
    }

    #[test]
    fn slicing_rejects_ambiguous_or_invalid_semantics() {
        let tensor = rank_four_activations();
        assert!(matches!(
            tensor.try_slice(
                "duplicate",
                &TensorSlice::new().at("head", "h0").at("head", "h1")
            ),
            Err(ValidationError::DuplicateIdentifier {
                field: "slice axes",
                ..
            })
        ));
        assert!(matches!(
            tensor.try_slice(
                "missing",
                &TensorSlice::new().elements("feature", ["not-a-feature"])
            ),
            Err(ValidationError::UnknownIdentifier {
                field: "slice axis element",
                ..
            })
        ));
        assert!(matches!(
            tensor.try_project_2d(
                "too-many-axes",
                "token",
                "feature",
                &TensorSlice::new().at("batch", "b0")
            ),
            Err(ValidationError::RankMismatch {
                field: "2D projection retained axes",
                actual: 3,
                ..
            })
        ));
        assert!(matches!(
            tensor.try_slice(
                "scalar",
                &TensorSlice::new()
                    .at("batch", "b0")
                    .at("head", "h0")
                    .at("token", "t0")
                    .at("feature", "f0")
            ),
            Err(ValidationError::Empty {
                field: "slice retained axes",
                ..
            })
        ));
    }

    #[test]
    fn transpose_swaps_axes_shape_and_values() {
        let tensor = TensorSnapshot::try_new(
            "k",
            vec![2, 3],
            vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            vec![
                axis("key", &[("k0", "A"), ("k1", "B")]),
                axis("feature", &[("f0", "x"), ("f1", "y"), ("f2", "z")]),
            ],
        )
        .unwrap();

        let transposed = tensor.try_transpose_2d("k.transpose").unwrap();
        assert_eq!(transposed.shape, vec![3, 2]);
        assert_eq!(transposed.values, vec![1.0, 4.0, 2.0, 5.0, 3.0, 6.0]);
        assert_eq!(transposed.axes[0].id, "feature");
        assert_eq!(transposed.axes[1].id, "key");
    }

    #[test]
    fn matmul_aligns_contracted_elements_by_semantic_id() {
        let lhs = TensorSnapshot::try_new(
            "q",
            vec![2, 2],
            vec![1.0, 2.0, 3.0, 4.0],
            vec![
                axis("query", &[("q0", "A"), ("q1", "B")]),
                axis("feature", &[("f0", "x"), ("f1", "y")]),
            ],
        )
        .unwrap();
        let rhs = TensorSnapshot::try_new(
            "k.transpose",
            vec![2, 2],
            vec![5.0, 6.0, 7.0, 8.0],
            vec![
                axis("feature", &[("f1", "y"), ("f0", "x")]),
                axis("key", &[("k0", "A"), ("k1", "B")]),
            ],
        )
        .unwrap();

        let product = lhs.try_matmul(&rhs, "attention").unwrap();
        assert_eq!(product.values, vec![17.0, 20.0, 41.0, 48.0]);
        assert_eq!(product.axes[0].id, "query");
        assert_eq!(product.axes[1].id, "key");
    }

    #[test]
    fn matmul_rejects_incompatible_contraction_axes() {
        let lhs = TensorSnapshot::try_new(
            "lhs",
            vec![1, 1],
            vec![1.0],
            vec![axis("row", &[("r", "r")]), axis("feature", &[("f", "f")])],
        )
        .unwrap();
        let rhs = TensorSnapshot::try_new(
            "rhs",
            vec![1, 1],
            vec![1.0],
            vec![axis("other", &[("f", "f")]), axis("column", &[("c", "c")])],
        )
        .unwrap();

        assert!(matches!(
            lhs.try_matmul(&rhs, "product"),
            Err(ValidationError::Incompatible {
                field: "matmul contraction axis",
                ..
            })
        ));
    }

    #[test]
    fn scaling_and_causal_mask_preserve_tensor_identity() {
        let tensor = TensorSnapshot::try_new(
            "attention",
            vec![2, 2],
            vec![2.0, 4.0, 6.0, 8.0],
            vec![
                axis("query", &[("q0", "A"), ("q1", "B")]),
                axis("key", &[("k0", "A"), ("k1", "B")]),
            ],
        )
        .unwrap();

        let scaled = tensor.try_scaled(2.0).unwrap();
        assert_eq!(scaled.id, "attention");
        assert_eq!(scaled.values, vec![1.0, 2.0, 3.0, 4.0]);
        let masked = scaled.try_causal_masked(-10.0).unwrap();
        assert_eq!(masked.id, "attention");
        assert_eq!(masked.values, vec![1.0, -10.0, 3.0, 4.0]);
        assert!(matches!(
            tensor.try_scaled(0.0),
            Err(ValidationError::NonPositive { .. })
        ));
    }

    #[test]
    fn softmax_is_stable_and_normalizes_the_named_axis() {
        let tensor = TensorSnapshot::try_new(
            "attention",
            vec![2, 2],
            vec![1000.0, 1001.0, 0.0, 0.0],
            vec![
                axis("query", &[("q0", "A"), ("q1", "B")]),
                axis("key", &[("k0", "A"), ("k1", "B")]),
            ],
        )
        .unwrap();

        let result = tensor.try_softmax("key").unwrap();
        assert_close(result.values[0], 0.268_941_43);
        assert_close(result.values[1], 0.731_058_6);
        assert_close(result.values[2], 0.5);
        assert_close(result.values[3], 0.5);
        assert_close(result.values[0] + result.values[1], 1.0);
        assert_close(result.values[2] + result.values[3], 1.0);
    }

    #[test]
    fn layer_norm_and_rms_norm_follow_named_axis_semantics() {
        let tensor = TensorSnapshot::try_new(
            "residual",
            vec![2, 3],
            vec![1.0, 2.0, 3.0, 2.0, 4.0, 6.0],
            vec![
                axis("token", &[("t0", "A"), ("t1", "B")]),
                axis("feature", &[("f0", "x"), ("f1", "y"), ("f2", "z")]),
            ],
        )
        .unwrap();

        let layer = tensor
            .try_normalized(
                "layer_norm",
                "feature",
                TensorNormalization::LayerNorm,
                1e-5,
            )
            .unwrap();
        assert_close(layer.values[0] + layer.values[1] + layer.values[2], 0.0);
        assert_close(layer.values[3] + layer.values[4] + layer.values[5], 0.0);
        assert_eq!(layer.axes, tensor.axes);

        let rms = tensor
            .try_normalized("rms_norm", "feature", TensorNormalization::RmsNorm, 1e-5)
            .unwrap();
        let first_square_mean = rms.values[..3]
            .iter()
            .map(|value| value * value)
            .sum::<f32>()
            / 3.0;
        assert_close(first_square_mean, 1.0);
        assert!(rms.values.iter().all(|value| *value > 0.0));
    }

    #[test]
    fn normalization_rejects_unknown_axes_and_invalid_epsilon() {
        let tensor = TensorSnapshot::try_new(
            "residual",
            vec![2],
            vec![1.0, 2.0],
            vec![axis("feature", &[("f0", "x"), ("f1", "y")])],
        )
        .unwrap();
        assert!(matches!(
            tensor.try_normalized("out", "missing", TensorNormalization::LayerNorm, 1e-5),
            Err(ValidationError::UnknownIdentifier { .. })
        ));
        assert!(matches!(
            tensor.try_normalized("out", "feature", TensorNormalization::LayerNorm, 0.0),
            Err(ValidationError::NonPositive {
                field: "normalization epsilon",
                ..
            })
        ));
    }

    #[test]
    fn attention_operation_chain_produces_normalized_causal_rows() {
        let q = TensorSnapshot::try_new(
            "q",
            vec![2, 2],
            vec![1.0, 0.0, 0.0, 1.0],
            vec![
                axis("query", &[("t0", "A"), ("t1", "B")]),
                axis("feature", &[("f0", "x"), ("f1", "y")]),
            ],
        )
        .unwrap();
        let k = TensorSnapshot::try_new(
            "k",
            vec![2, 2],
            vec![1.0, 0.0, 0.0, 1.0],
            vec![
                axis("key", &[("t0", "A"), ("t1", "B")]),
                axis("feature", &[("f0", "x"), ("f1", "y")]),
            ],
        )
        .unwrap();

        let weights = q
            .try_matmul(&k.try_transpose_2d("k.transpose").unwrap(), "attention")
            .unwrap()
            .try_scaled(2.0_f32.sqrt())
            .unwrap()
            .try_causal_masked(-20.0)
            .unwrap()
            .try_softmax("key")
            .unwrap();
        assert_close(weights.values[0], 1.0);
        assert!(weights.values[1] < 1e-8);
        assert_close(weights.values[2] + weights.values[3], 1.0);
    }

    #[test]
    fn reshape_uses_explicit_axes_and_preserves_row_major_values() {
        let tensor = TensorSnapshot::try_new(
            "tokens",
            vec![2, 3],
            vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            vec![
                axis("token", &[("t0", "A"), ("t1", "B")]),
                axis("feature", &[("f0", "x"), ("f1", "y"), ("f2", "z")]),
            ],
        )
        .unwrap();

        let reshaped = tensor
            .try_reshape(
                "heads",
                vec![
                    axis("head", &[("h0", "0"), ("h1", "1"), ("h2", "2")]),
                    axis("channel", &[("c0", "0"), ("c1", "1")]),
                ],
            )
            .unwrap();
        assert_eq!(reshaped.shape, vec![3, 2]);
        assert_eq!(reshaped.values, tensor.values);
        assert_eq!(reshaped.axes[0].element_ids, vec!["h0", "h1", "h2"]);

        assert!(matches!(
            tensor.try_reshape(
                "invalid",
                vec![axis("too_short", &[("x0", "0"), ("x1", "1")])],
            ),
            Err(ValidationError::LengthMismatch {
                field: "reshape output values",
                ..
            })
        ));
    }

    #[test]
    fn split_and_merge_round_trip_a_non_leading_axis() {
        let tensor = TensorSnapshot::try_new(
            "activations",
            vec![2, 4],
            vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0],
            vec![
                axis("token", &[("t0", "A"), ("t1", "B")]),
                axis(
                    "feature",
                    &[("f0", "w"), ("f1", "x"), ("f2", "y"), ("f3", "z")],
                ),
            ],
        )
        .unwrap();

        let parts = tensor
            .try_split("feature", &[1, 3], &["gate", "values"])
            .unwrap();
        assert_eq!(parts[0].shape, vec![2, 1]);
        assert_eq!(parts[0].values, vec![1.0, 5.0]);
        assert_eq!(parts[1].values, vec![2.0, 3.0, 4.0, 6.0, 7.0, 8.0]);
        assert_eq!(parts[1].axes[1].element_ids, vec!["f1", "f2", "f3"]);

        let merged = TensorSnapshot::try_merge(&parts, "feature", "activations.joined").unwrap();
        assert_eq!(merged.shape, tensor.shape);
        assert_eq!(merged.values, tensor.values);
        assert_eq!(merged.axes, tensor.axes);
    }

    #[test]
    fn split_validates_lengths_and_output_ids() {
        let tensor = TensorSnapshot::try_new(
            "vector",
            vec![3],
            vec![1.0, 2.0, 3.0],
            vec![axis("feature", &[("f0", "x"), ("f1", "y"), ("f2", "z")])],
        )
        .unwrap();

        assert!(matches!(
            tensor.try_split("feature", &[1, 1], &["a", "b"]),
            Err(ValidationError::LengthMismatch {
                field: "split lengths",
                ..
            })
        ));
        assert!(matches!(
            tensor.try_split("feature", &[1, 2], &["only_one"]),
            Err(ValidationError::LengthMismatch {
                field: "split output ids",
                ..
            })
        ));
    }

    #[test]
    fn merge_rejects_duplicate_element_identity() {
        let part = TensorSnapshot::try_new(
            "part",
            vec![1],
            vec![1.0],
            vec![axis("feature", &[("f0", "x")])],
        )
        .unwrap();

        assert!(matches!(
            TensorSnapshot::try_merge(&[part.clone(), part], "feature", "invalid"),
            Err(ValidationError::DuplicateIdentifier {
                field: "axis elements",
                ..
            })
        ));
    }

    #[test]
    fn elementwise_aligns_reordered_elements_and_broadcasts_missing_axes() {
        let activations = TensorSnapshot::try_new(
            "activations",
            vec![2, 3],
            vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            vec![
                axis("token", &[("t0", "A"), ("t1", "B")]),
                axis("feature", &[("f0", "x"), ("f1", "y"), ("f2", "z")]),
            ],
        )
        .unwrap();
        let bias = TensorSnapshot::try_new(
            "bias",
            vec![3],
            vec![30.0, 10.0, 20.0],
            vec![axis("feature", &[("f2", "z"), ("f0", "x"), ("f1", "y")])],
        )
        .unwrap();

        let output = activations
            .try_elementwise(&bias, "biased", TensorElementwiseOp::Add)
            .unwrap();
        assert_eq!(output.shape, activations.shape);
        assert_eq!(output.axes, activations.axes);
        assert_eq!(output.values, vec![11.0, 22.0, 33.0, 14.0, 25.0, 36.0]);
    }

    #[test]
    fn elementwise_supports_singleton_broadcast_and_rejects_non_finite_results() {
        let tensor = TensorSnapshot::try_new(
            "tensor",
            vec![2, 2],
            vec![1.0, 2.0, 3.0, 4.0],
            vec![
                axis("token", &[("t0", "A"), ("t1", "B")]),
                axis("feature", &[("f0", "x"), ("f1", "y")]),
            ],
        )
        .unwrap();
        let scale = TensorSnapshot::try_new(
            "scale",
            vec![1],
            vec![2.0],
            vec![axis("token", &[("all", "All")])],
        )
        .unwrap();
        let zero = TensorSnapshot::try_new(
            "zero",
            vec![1],
            vec![0.0],
            vec![axis("feature", &[("all", "All")])],
        )
        .unwrap();

        let scaled = tensor
            .try_elementwise(&scale, "scaled", TensorElementwiseOp::Multiply)
            .unwrap();
        assert_eq!(scaled.values, vec![2.0, 4.0, 6.0, 8.0]);
        assert!(matches!(
            tensor.try_elementwise(&zero, "invalid", TensorElementwiseOp::Divide),
            Err(ValidationError::NonFinite {
                field: "elementwise result",
                ..
            })
        ));
    }

    #[test]
    fn categorical_sampling_is_deterministic_and_preserves_element_identity() {
        let probabilities = TensorSnapshot::try_new(
            "next_token",
            vec![2, 3],
            vec![0.1, 0.2, 0.7, 0.6, 0.3, 0.1],
            vec![
                axis("step", &[("s0", "0"), ("s1", "1")]),
                axis("vocabulary", &[("v0", "A"), ("v1", "B"), ("v2", "C")]),
            ],
        )
        .unwrap();

        let samples = probabilities
            .try_sample_categorical("vocabulary", &[0.25, 0.75])
            .unwrap();
        assert_eq!(samples[0].element_id.coordinates[1].element_id, "v1");
        assert_eq!(samples[1].element_id.coordinates[1].element_id, "v1");
        assert_close(samples[0].probability, 0.2);
        assert_close(samples[1].probability, 0.3);
    }

    #[test]
    fn categorical_sampling_rejects_invalid_variates_and_probabilities() {
        let probabilities = TensorSnapshot::try_new(
            "distribution",
            vec![2],
            vec![0.5, 0.5],
            vec![axis("vocabulary", &[("v0", "A"), ("v1", "B")])],
        )
        .unwrap();
        assert!(matches!(
            probabilities.try_sample_categorical("vocabulary", &[1.0]),
            Err(ValidationError::OutOfRange { .. })
        ));

        let negative = TensorSnapshot::try_new(
            "distribution",
            vec![2],
            vec![-0.1, 1.1],
            vec![axis("vocabulary", &[("v0", "A"), ("v1", "B")])],
        )
        .unwrap();
        assert!(matches!(
            negative.try_sample_categorical("vocabulary", &[0.5]),
            Err(ValidationError::Incompatible {
                field: "categorical values",
                ..
            })
        ));
    }
}
