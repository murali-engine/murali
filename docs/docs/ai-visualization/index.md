---
sidebar_position: 8
---

# AI Visualization

Murali is a Python animation engine for AI education and technical visual storytelling. Teaching
views such as attention matrices and neural diagrams live in [Murali Kit](../murali-kit). The
runtime is written in Rust for performance; authoring and integrations are Python-first.

The long-term target is to grow that visualization surface steadily through the end of 2030.

Murali is not trying to become an inference framework. It is meant to visualize authored or recorded
state: tensors, tokens, matrices, distributions, model traces, optimization steps, geometric
intuition, and the surrounding mathematical structure.

## Current Status

- **AI visualization exists today.** Murali includes components for attention matrices, context
  windows, KV cache, next-token distributions, neural-network diagrams, transformer blocks,
  normalization views, tensor snapshots, semantic tensor transitions, decision boundaries, and AI
  traces.
- **Linear algebra is experimental.** The `experimental` feature exposes reusable linear-algebra
  pieces for vectors, transformed grids, matrix-vector flow, column combinations, projections,
  orthogonality, dimension badges, transform composition, and basis changes.
- **Other education categories are planned.** Probability, statistics, calculus, optimization,
  information theory, data geometry, and systems/agentic AI will be expanded over the next several
  years as reusable teaching components prove themselves in real examples.

## Category Surface

| Category | Functional anchor today | Direction |
| --- | --- | --- |
| Basic math | equations, matrices, labels, LaTeX, Typst, axes | algebra steps, intervals, symbolic transforms |
| Linear algebra | experimental `linear_algebra` components | rank, eigenspaces, SVD, subspaces |
| Probability | `NextTokenDistribution`, graphs, tables | distributions, sampling, Bayes diagrams |
| Statistics | `NormalizationView`, `DecisionBoundaryPlot`, scatter plots | regression, uncertainty, metrics |
| Calculus | function graphs, parametric curves/surfaces, vector fields | derivatives, integrals, gradients |
| Optimization | surfaces, vector fields, traced paths | descent, optimizers, loss landscapes |
| Information theory | next-token probabilities, tensors, equations | entropy, KL, cross-entropy, calibration |
| ML components | tensors, decision boundaries, signal flow | datasets, embeddings, losses, evaluations |
| Deep learning | neural-network, transformer, attention, normalization views | gradients, residuals, feature maps |
| Transformers and LLMs | context, tokens, attention, KV cache, sampling, traces | generation loops, RAG, tools, memory |
| Data geometry | scatter plots, surfaces, tensor projections | embeddings, PCA, clustering, manifolds |
| Systems and agentic AI | agentic flow charts, traces, context windows | tools, retries, branches, evaluations |

## Import surface

Murali Kit is the author-facing collection of reusable teaching views. Import domain components
from Python modules that match the lesson you are writing:

```python
from murali_kit.ai import AttentionMatrix, NeuralNetworkDiagram
from murali_kit.maths import FunctionGraph, NumberLine
from murali_engine import Scene, Timeline
```

Use `murali-engine` for core scene objects, primitives, timelines, preview, and export. Use
`murali-kit` for named colors, themes, examples, and composed teaching views.

The lower-level Rust collection remains the implementation and extension surface for engine work.
Public lesson authoring should stay in Python.

## Source Architecture

The source-visible category READMEs live in
[`src/frontend/collection`](https://github.com/murali-engine/murali/tree/main/src/frontend/collection).
They document the category ownership rules and intended 2026-2030 evolution without pretending that
every planned area is already stable.

The rule for this surface is simple: add reusable teaching primitives and composites, not one-off
scene shortcuts. A category can stay planned until a real lesson proves that a component belongs in
Murali.
