# AI

`ai` owns AI-specific semantic views, model-state components, and AI teaching-domain facades.

Functional anchors today:

- `AttentionMatrix`
- `ContextWindow`
- `KvCacheView`
- `NextTokenDistribution`
- `NormalizationView`
- tensor snapshots, tensor slices, tensor views, and tensor operations
- `TokenSequence`
- `TransformerBlockDiagram`
- `AiTrace`
- `AgenticFlowChart`

Teaching domains:

- `deep_learning`
- `ml_components`
- `transformers_llms`
- `systems_agentic_ai`

Add implementation code here when the component is specifically about AI state, model internals,
tokens, traces, tensors, or AI system behavior. Domain subfolders may re-export components for
discovery, but should not duplicate implementations.
