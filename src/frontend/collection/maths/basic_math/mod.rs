//! Basic math teaching components.

pub mod number_line;

pub use number_line::NumberLine;

pub use crate::frontend::collection::composite::axes::Axes;
pub use crate::frontend::collection::composite::number_plane::NumberPlane;
pub use crate::frontend::collection::maths::calculus::function_graph::FunctionGraph;
pub use crate::frontend::collection::maths::notation::equation::{
    EquationLayout, EquationPart, EquationPartLayout, VectorEquation, VectorEquationHandle,
    VectorLatexEquation, VectorTypstEquation,
};
pub use crate::frontend::collection::maths::notation::matrix::{
    Matrix, MatrixCell, MatrixCellLayout,
};
pub use crate::frontend::collection::table::{Table, TableConfig, TableTitlePosition, TableV1};
pub use crate::frontend::collection::text::label::Label;
pub use crate::frontend::collection::text::latex::Latex;
pub use crate::frontend::collection::text::typst::Typst;
