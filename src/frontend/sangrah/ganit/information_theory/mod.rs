//! Information-theory teaching components.

pub mod entropy;

pub use entropy::EntropyMeter;

pub use crate::frontend::sangrah::ai::transformers_llms::attention_matrix::AttentionMatrix;
pub use crate::frontend::sangrah::ai::{
    NextTokenCandidate, NextTokenDistribution, NextTokenSampling, TensorAxis, TensorSelector,
    TensorSnapshot, TensorView,
};
pub use crate::frontend::sangrah::ganit::calculus::function_graph::FunctionGraph;
pub use crate::frontend::sangrah::ganit::notation::equation::{
    VectorEquation, VectorLatexEquation,
};
pub use crate::frontend::sangrah::table::{Table, TableConfig, TableTitlePosition, TableV1};
