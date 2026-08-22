// Main agentic flow chart module
//
// This module provides a flexible flowchart visualization system with animation support.
// The implementation is split across multiple files for maintainability:
//
// - types.rs: Core enums and basic types
// - node.rs: FlowNode struct and related functionality
// - edge.rs: FlowEdge struct and routing
// - chart.rs: Main AgenticFlowChart implementation

mod animation;
mod chart;
pub mod edge;
mod layout;
pub mod model;
pub mod node;
mod renderer;
mod routing;
mod shapes;
pub mod types;

// Re-export everything for backward compatibility
pub use chart::AgenticFlowChart;
pub use edge::FlowEdge;
pub use model::{FlowChartAnimation, FlowChartModel, FlowChartStyle};
pub use node::{FlowNode, FlowNodeContent, ProjectedFlowNodeContent};
pub use types::*;
