pub mod equation;
/// Experimental linear algebra visual components.
///
/// This module is under active development. Public names, builder methods, layout defaults, and
/// rendering details may change before this API is promoted to stable.
///
/// Every public item exported from this module is experimental unless its own documentation says
/// otherwise.
#[cfg(feature = "experimental")]
pub mod linear_algebra;
pub mod matrix;
