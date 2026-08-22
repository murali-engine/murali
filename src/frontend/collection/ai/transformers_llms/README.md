# Transformers And LLMs

Responsibility: tokenization, context assembly, attention, KV cache, transformer blocks,
next-token prediction, sampling, generation loops, and LLM-specific traces.

Functional anchors today:

- `ContextWindow`
- `TokenSequence`
- `AttentionMatrix`
- `KvCacheView`
- `NextTokenDistribution`
- `TransformerBlockDiagram`
- `AiTrace`

Add here when a component is specific to transformers, language models, or autoregressive
generation.

Do not put general neural-network pieces here unless the LLM framing is essential. General model
internals belong in `ai::deep_learning`.
