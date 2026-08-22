//! Optimization teaching components.

pub mod path;

pub use path::OptimizationPath2D;

pub use crate::frontend::sangrah::common::{TensorSample, TensorSnapshot, TensorView};
pub use crate::frontend::sangrah::ganit::calculus::function_graph::FunctionGraph;
pub use crate::frontend::sangrah::ganit::calculus::parametric_surface::ParametricSurface;
pub use crate::frontend::sangrah::ganit::calculus::stream_lines::StreamLines;
pub use crate::frontend::sangrah::ganit::calculus::vector_field::VectorField;
#[cfg(feature = "experimental")]
pub use crate::frontend::sangrah::ganit::linear_algebra::ProjectionShadow;
pub use crate::frontend::sangrah::utility::TracedPath;
