//! Shared sangrah infrastructure used by multiple teaching domains.
//!
//! Keep this module for domain-neutral building blocks only. If a component belongs to a
//! teaching subject, place it in that subject folder and re-export it where helpful.

pub mod tensor;
mod tensor_ops;

pub use tensor::{
    TensorAxis, TensorCellLayout, TensorCoordinate, TensorElementId, TensorSelector,
    TensorSnapshot, TensorView,
};
pub use tensor_ops::{TensorElementwiseOp, TensorNormalization, TensorSample, TensorSlice};
