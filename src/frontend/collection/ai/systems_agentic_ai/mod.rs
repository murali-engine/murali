//! Systems, RAG, and agentic-AI teaching components.

pub mod agentic_flow_chart;

pub use agentic_flow_chart::AgenticFlowChart;

pub use crate::frontend::collection::ai::transformers_llms::{
    AI_TRACE_SCHEMA_VERSION, AiModelMetadata, AiTrace, AiTraceError, AiTraceEvent,
    AiTraceEventKind, ContextBlock, ContextBlockRole, ContextTruncation, ContextWindow,
    TokenSequence, TraceToken, TraceTokenRole,
};
pub use crate::frontend::collection::storytelling::stepwise::{
    Stepwise, StepwiseDirection, StepwiseLayout, StepwiseStyle,
};
