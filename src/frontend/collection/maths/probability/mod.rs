//! Probability teaching components.

pub mod next_token_distribution;

pub use next_token_distribution::{NextTokenCandidate, NextTokenDistribution, NextTokenSampling};

pub use crate::frontend::collection::common::{
    TensorAxis, TensorSelector, TensorSnapshot, TensorView,
};
pub use crate::frontend::collection::maths::calculus::function_graph::FunctionGraph;
pub use crate::frontend::collection::maths::data_geometry::scatter_plot::ScatterPlot;
pub use crate::frontend::collection::table::{Table, TableConfig, TableTitlePosition, TableV1};
