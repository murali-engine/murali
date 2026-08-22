//! Embedding, projection, and data-geometry teaching components.

pub mod scatter_plot;

pub use scatter_plot::ScatterPlot;

pub use crate::frontend::sangrah::common::{
    TensorAxis, TensorSelector, TensorSnapshot, TensorView,
};
pub use crate::frontend::sangrah::composite::axes::Axes;
pub use crate::frontend::sangrah::composite::axes3d::Axes3D;
pub use crate::frontend::sangrah::ganit::calculus::parametric_curve3d::ParametricCurve3D;
pub use crate::frontend::sangrah::ganit::calculus::parametric_surface::ParametricSurface;
#[cfg(feature = "experimental")]
pub use crate::frontend::sangrah::ganit::linear_algebra::{
    BasisGrid2D, BasisVectors2D, ProjectionShadow, SpanRegion2D,
};
