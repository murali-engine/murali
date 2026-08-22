//! Transformer and LLM teaching components.

pub mod attention_matrix;
pub mod context_window;
pub mod kv_cache;
pub mod token_sequence;
pub mod trace;
pub mod transformer_block_diagram;

pub use attention_matrix::AttentionMatrix;
pub use context_window::{ContextBlock, ContextBlockRole, ContextTruncation, ContextWindow};
pub use kv_cache::KvCacheView;
pub use token_sequence::TokenSequence;
pub use trace::{
    AI_TRACE_SCHEMA_VERSION, AiModelMetadata, AiTrace, AiTraceError, AiTraceEvent,
    AiTraceEventKind, TraceToken, TraceTokenRole,
};
pub use transformer_block_diagram::{
    TRANSFORMER_INPUT_ID, TransformerBlockDiagram, TransformerStage, TransformerStageFocusFrame,
    TransformerStageKind,
};

pub use crate::frontend::collection::common::{
    TensorAxis, TensorSelector, TensorSnapshot, TensorView,
};
pub use crate::frontend::collection::maths::probability::{
    NextTokenCandidate, NextTokenDistribution, NextTokenSampling,
};
