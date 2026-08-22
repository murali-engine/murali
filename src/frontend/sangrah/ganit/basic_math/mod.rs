//! Basic math teaching components.

pub mod number_line;

pub use number_line::NumberLine;

pub use crate::frontend::sangrah::composite::axes::Axes;
pub use crate::frontend::sangrah::composite::number_plane::NumberPlane;
pub use crate::frontend::sangrah::ganit::calculus::function_graph::FunctionGraph;
pub use crate::frontend::sangrah::ganit::notation::equation::{
    EquationLayout, EquationPart, EquationPartLayout, VectorEquation, VectorEquationHandle,
    VectorLatexEquation, VectorTypstEquation,
};
pub use crate::frontend::sangrah::ganit::notation::matrix::{Matrix, MatrixCell, MatrixCellLayout};
pub use crate::frontend::sangrah::table::{Table, TableConfig, TableTitlePosition, TableV1};
pub use crate::frontend::sangrah::text::label::Label;
pub use crate::frontend::sangrah::text::latex::Latex;
pub use crate::frontend::sangrah::text::typst::Typst;
