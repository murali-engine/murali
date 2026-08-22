//! Optimization teaching components.

pub mod path;

pub use path::OptimizationPath2D;

pub use crate::frontend::collection::common::{TensorSample, TensorSnapshot, TensorView};
pub use crate::frontend::collection::maths::calculus::function_graph::FunctionGraph;
pub use crate::frontend::collection::maths::calculus::parametric_surface::ParametricSurface;
pub use crate::frontend::collection::maths::calculus::stream_lines::StreamLines;
pub use crate::frontend::collection::maths::calculus::vector_field::VectorField;
#[cfg(feature = "experimental")]
pub use crate::frontend::collection::maths::linear_algebra::ProjectionShadow;
pub use crate::frontend::collection::utility::TracedPath;
