# Deep Learning

Responsibility: neural-network internals, layers, activations, tensors, attention-like blocks,
normalization, gradients, residual paths, and recorded model state.

Functional anchors today:

- `NeuralNetworkDiagram`
- `TransformerBlockDiagram`
- `AttentionMatrix`
- `NormalizationView`
- tensor snapshots, tensor views, tensor slices, and tensor operations

Add here when a component explains learned model internals across architectures.

Do not put LLM-specific prompt, context, tool, or generation-loop components here; those belong in
`ai::transformers_llms` or `ai::systems_agentic_ai`.
