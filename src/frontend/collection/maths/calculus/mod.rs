//! Calculus and continuous-geometry teaching components.

pub mod function_graph;
pub mod parametric_curve;
pub mod parametric_curve3d;
pub mod parametric_surface;
pub mod stream_lines;
pub mod vector_field;

pub use function_graph::FunctionGraph;
pub use parametric_curve::ParametricCurve;
pub use parametric_curve3d::ParametricCurve3D;
pub use parametric_surface::{ParametricSurface, SurfaceRenderMode};
pub use stream_lines::StreamLines;
pub use vector_field::VectorField;

pub use crate::frontend::collection::composite::axes::Axes;
pub use crate::frontend::collection::composite::axes3d::Axes3D;
pub use crate::frontend::collection::utility::TracedPath;
