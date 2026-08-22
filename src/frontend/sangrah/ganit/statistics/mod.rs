//! Statistics teaching components.

pub mod decision_boundary_plot;
pub mod normalization_view;

pub use decision_boundary_plot::DecisionBoundaryPlot;
pub use normalization_view::{NormalizationStats, NormalizationView};

pub use crate::frontend::sangrah::common::{TensorAxis, TensorSample, TensorSnapshot, TensorView};
pub use crate::frontend::sangrah::ganit::calculus::function_graph::FunctionGraph;
pub use crate::frontend::sangrah::ganit::data_geometry::scatter_plot::ScatterPlot;
pub use crate::frontend::sangrah::table::{Table, TableConfig, TableTitlePosition, TableV1};
