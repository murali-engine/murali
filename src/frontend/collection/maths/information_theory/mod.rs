//! Information-theory teaching components.

pub mod entropy;

pub use entropy::EntropyMeter;

pub use crate::frontend::collection::ai::transformers_llms::attention_matrix::AttentionMatrix;
pub use crate::frontend::collection::ai::{
    NextTokenCandidate, NextTokenDistribution, NextTokenSampling, TensorAxis, TensorSelector,
    TensorSnapshot, TensorView,
};
pub use crate::frontend::collection::maths::calculus::function_graph::FunctionGraph;
pub use crate::frontend::collection::maths::notation::equation::{
    VectorEquation, VectorLatexEquation,
};
pub use crate::frontend::collection::table::{Table, TableConfig, TableTitlePosition, TableV1};
