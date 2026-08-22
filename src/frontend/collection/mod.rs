// src/frontend/collection/mod.rs

pub mod ai;
pub(crate) mod common;
pub mod composite;
pub mod layout;
pub mod maths;
pub mod primitives;
pub mod storytelling;
pub mod table;
pub mod text;
pub mod utility;

pub mod prelude {
    pub use super::ai::*;
    pub use super::composite::*;
    pub use super::layout::*;
    pub use super::maths::calculus::*;
    pub use super::maths::data_geometry::*;
    pub use super::maths::notation::*;
    pub use super::primitives::*;
    pub use super::storytelling::*;
    pub use super::table::*;
    pub use super::text::*;
    pub use super::utility::*;
}
