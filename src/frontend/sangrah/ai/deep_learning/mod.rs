//! Deep-learning teaching components.

pub mod neural_network_diagram;

pub use neural_network_diagram::{
    ActivationFunc, IndicationStyle, NeuralNetworkDiagram, NeuralNetworkDiagramError,
};

pub use crate::frontend::sangrah::ai::transformers_llms::{
    AttentionMatrix, TransformerBlockDiagram, TransformerStage, TransformerStageFocusFrame,
    TransformerStageKind,
};
pub use crate::frontend::sangrah::common::{
    TensorAxis, TensorCellLayout, TensorCoordinate, TensorElementId, TensorElementwiseOp,
    TensorNormalization, TensorSample, TensorSelector, TensorSlice, TensorSnapshot, TensorView,
};
pub use crate::frontend::sangrah::ganit::statistics::{NormalizationStats, NormalizationView};
