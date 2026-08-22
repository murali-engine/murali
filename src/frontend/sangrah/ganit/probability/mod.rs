//! Probability teaching components.

pub mod next_token_distribution;

pub use next_token_distribution::{NextTokenCandidate, NextTokenDistribution, NextTokenSampling};

pub use crate::frontend::sangrah::common::{
    TensorAxis, TensorSelector, TensorSnapshot, TensorView,
};
pub use crate::frontend::sangrah::ganit::calculus::function_graph::FunctionGraph;
pub use crate::frontend::sangrah::ganit::data_geometry::scatter_plot::ScatterPlot;
pub use crate::frontend::sangrah::table::{Table, TableConfig, TableTitlePosition, TableV1};
