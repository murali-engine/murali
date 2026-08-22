---
sidebar_position: 8
---

# AI Visualization

Murali is being built as a Rust-native animation engine for AI education and technical visual
storytelling. The long-term target is to grow the visualization surface steadily through the end of
2030, with reusable components for the math and systems ideas behind modern AI.

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

## Import Surface

Murali's collection is the author-facing collection of reusable visual tattvas. Its implementation
families remain organized by primitive family: `ai`, `maths`, `text`, `layout`, `storytelling`,
and related modules. Alongside those, Murali exposes domain-oriented category modules directly
under `collection`:

```rust
use murali::collection::ai::transformers_llms::*;
use murali::collection::maths::probability::*;
use murali::collection::maths::data_geometry::scatter_plot::ScatterPlot;
```

These category modules are re-export layers over real components. They are meant for lesson authors
who think in teaching domains rather than implementation folders. A component still has one
implementation owner; a domain facade is only a discovery surface. The experimental
linear-algebra module is available only when the `experimental` feature is enabled.

Support pieces stay in implementation families such as `primitives`, `text`, `maths`, `composite`,
`layout`, `storytelling`, `table`, and `utility`. Domain facades are kept only where
they clarify the teaching subject.

## Source Architecture

The source-visible category READMEs live in
[`src/frontend/collection`](https://github.com/ravishankarkumar/murali/tree/main/src/frontend/collection).
They document the category ownership rules and intended 2026-2030 evolution without pretending that
every planned area is already stable.

The rule for this surface is simple: add reusable teaching primitives and composites, not one-off
scene shortcuts. A category can stay planned until a real lesson proves that a component belongs in
Murali.
