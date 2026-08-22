//! Classical machine-learning teaching components.

pub mod signal_flow;

pub use signal_flow::SignalFlow;

pub use crate::frontend::sangrah::ai::transformers_llms::TokenSequence;
pub use crate::frontend::sangrah::common::{
    TensorAxis, TensorCellLayout, TensorCoordinate, TensorElementId, TensorElementwiseOp,
    TensorNormalization, TensorSample, TensorSelector, TensorSlice, TensorSnapshot, TensorView,
};
pub use crate::frontend::sangrah::ganit::data_geometry::ScatterPlot;
pub use crate::frontend::sangrah::ganit::statistics::DecisionBoundaryPlot;
pub use crate::frontend::sangrah::table::{Table, TableConfig, TableTitlePosition, TableV1};
