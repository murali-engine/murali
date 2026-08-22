//! Classical machine-learning teaching components.

pub mod signal_flow;

pub use signal_flow::SignalFlow;

pub use crate::frontend::collection::ai::transformers_llms::TokenSequence;
pub use crate::frontend::collection::common::{
    TensorAxis, TensorCellLayout, TensorCoordinate, TensorElementId, TensorElementwiseOp,
    TensorNormalization, TensorSample, TensorSelector, TensorSlice, TensorSnapshot, TensorView,
};
pub use crate::frontend::collection::maths::data_geometry::ScatterPlot;
pub use crate::frontend::collection::maths::statistics::DecisionBoundaryPlot;
pub use crate::frontend::collection::table::{Table, TableConfig, TableTitlePosition, TableV1};
