//! `Stepwise` — A professional-grade storytelling component for Murali.
//!
//! Stepwise enables "Journey-based" animations where complex diagrams or flows are
//! revealed step-by-step. It supports:
//!
//! - **Deterministic Grid Routing**: Move paths precisely between node centers.
//! - **Cyclic Journeys**: Sequences like `[A, B, C, B, D]` with automatic feedback loop handling.
//! - **Spatial Anchoring**: Intelligent entrance/exit side selection (Top, Bottom, Left, Right).
//! - **Builder API**: Fluent script-building for complex transitions.

pub mod layout;
pub mod model;
pub mod script;
pub mod state;
pub mod tattva;
pub mod timeline;

pub use layout::{StepwiseDirection, StepwiseLayout};
pub use tattva::{Stepwise, StepwiseStyle};
