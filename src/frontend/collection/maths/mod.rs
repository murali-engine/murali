//! Mathematical and quantitative teaching components.
//!
//! `maths` groups Murali's math-facing notation, graphing, and education-domain surfaces.

pub mod basic_math;
pub mod calculus;
pub mod data_geometry;
pub mod graph;
pub mod information_theory;
#[cfg(feature = "experimental")]
pub mod linear_algebra;
pub mod notation;
pub mod optimization;
pub mod probability;
pub mod statistics;

pub mod prelude {
    pub use super::calculus::*;
    pub use super::data_geometry::*;
    pub use super::graph::*;
    #[cfg(feature = "experimental")]
    pub use super::linear_algebra::*;
    pub use super::notation::*;
}

pub use calculus::{FunctionGraph, ParametricCurve, ParametricCurve3D, ParametricSurface};
pub use data_geometry::ScatterPlot;
pub use graph::{PlotLegend, PlotLegendEntry};
#[cfg(feature = "experimental")]
pub use linear_algebra::{LabeledVector2D, MatrixVectorFlow, TransformableGrid2D, VectorArrow2D};
pub use notation::{Matrix, VectorEquation};
