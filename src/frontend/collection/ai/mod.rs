pub mod agentic_flow_chart;
pub mod attention_matrix;
pub mod decision_boundary_plot;
pub mod murali_ai_indicator;
pub mod neural_network_diagram;
pub mod signal_flow;
pub mod templates;
pub mod tensor;
mod tensor_ops;
pub mod token_sequence;
pub mod trace;
pub mod transformer_block_diagram;

pub use murali_ai_indicator::{
    MURALI_AI_INDICATOR_DURATION, MURALI_AI_INDICATOR_LOOP_CYCLE, MURALI_AI_INDICATOR_LOOP_START,
    MuraliAiIndicator, MuraliAiIndicatorIds,
};
pub use tensor::{
    TensorAxis, TensorCellLayout, TensorCoordinate, TensorElementId, TensorSelector,
    TensorSnapshot, TensorView,
};
pub use tensor_ops::{TensorElementwiseOp, TensorSample, TensorSlice};
pub use token_sequence::TokenSequence;
pub use trace::{
    AI_TRACE_SCHEMA_VERSION, AiModelMetadata, AiTrace, AiTraceError, AiTraceEvent,
    AiTraceEventKind, TraceToken, TraceTokenRole,
};
pub use transformer_block_diagram::{
    TRANSFORMER_INPUT_ID, TransformerBlockDiagram, TransformerStage, TransformerStageFocusFrame,
    TransformerStageKind,
};
