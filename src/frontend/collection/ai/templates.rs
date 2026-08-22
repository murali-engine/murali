use crate::frontend::collection::ai::deep_learning::neural_network_diagram::NeuralNetworkDiagram;
use crate::frontend::collection::ai::transformers_llms::attention_matrix::AttentionMatrix;
use crate::frontend::collection::ai::transformers_llms::token_sequence::TokenSequence;
use crate::frontend::collection::ai::transformers_llms::transformer_block_diagram::TransformerBlockDiagram;
use crate::frontend::theme::Theme;

pub struct AiUnderTheHoodTemplates;

impl AiUnderTheHoodTemplates {
    pub fn neural_network(layers: Vec<usize>) -> NeuralNetworkDiagram {
        let theme = Theme::global();
        let mut diagram = NeuralNetworkDiagram::new(layers);
        diagram.node_color = theme.accent;
        diagram.edge_color = theme.surface_alt;
        diagram
    }

    pub fn token_sequence(tokens: Vec<impl Into<String>>, token_height: f32) -> TokenSequence {
        let theme = Theme::global();
        let mut sequence = TokenSequence::new(tokens, token_height);
        sequence.text_color = theme.text_primary;
        sequence.box_color = theme.accent;
        sequence
    }

    pub fn attention_matrix(values: Vec<Vec<f32>>, tokens: Option<Vec<String>>) -> AttentionMatrix {
        let theme = Theme::global();
        let mut matrix = AttentionMatrix::new(values, tokens);
        matrix.low_color = theme.surface;
        matrix.high_color = theme.accent;
        matrix.grid_color = theme.text_muted;
        matrix
    }

    pub fn transformer_block() -> TransformerBlockDiagram {
        let theme = Theme::global();
        let mut block = TransformerBlockDiagram::new();
        block.accent_color = theme.accent_alt;
        block.frame_color = theme.surface_alt;
        block.text_color = theme.text_primary;
        block
    }
}
