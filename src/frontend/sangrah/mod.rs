// src/frontend/sangrah/mod.rs

pub mod ai;
pub(crate) mod common;
pub mod composite;
pub mod ganit;
pub mod layout;
pub mod primitives;
pub mod storytelling;
pub mod table;
pub mod text;
pub mod utility;

pub mod prelude {
    pub use super::ai::*;
    pub use super::composite::*;
    pub use super::ganit::calculus::*;
    pub use super::ganit::data_geometry::*;
    pub use super::ganit::notation::*;
    pub use super::layout::*;
    pub use super::primitives::*;
    pub use super::storytelling::*;
    pub use super::table::*;
    pub use super::text::*;
    pub use super::utility::*;
}
