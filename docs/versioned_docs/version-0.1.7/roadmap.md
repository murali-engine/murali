---
sidebar_position: 7
---

# Roadmap

Murali's direction is an AI-first Rust animation engine for deterministic technical storytelling.
It should visualize recorded model and system state rather than become an inference framework, and
it should preserve explicit author control while making common educational compositions easier to
build.

This page contains future work only. Released capabilities are documented in the guides and
release notes.

## Reliability And Quality Gates

- Cross-platform and headless `wgpu` runtime coverage.
- Golden-image tests for core rendering and representative AI scenes.
- Property tests for mesh limits, transformed bounds, capture ordering, and invalid input.
- Structured validation across remaining token, attention, label, color, dimension, and route APIs.
- A warning-free library build, broader strict Clippy, dependency auditing, and validated docs
  samples.

## AI Teaching Semantics

- A domain RFC defining snapshots, tokens, operations, semantic IDs, trace events, and data
  ownership.
- General image and image-grid compositions for datasets, feature maps, and multimodal lessons.
- Residual-stream, KV-cache, normalization, MLP, and mixture-of-experts semantics.
- Computation graphs with deterministic forward and reverse-mode playback.
- Parameter, gradient, loss, optimizer, activation, feature-map, and attribution views.
- A complete semantic backpropagation lesson spanning equations, tensors, networks, and plots.
- Recorded agent and RAG events for retrieval, tools, memory, retries, branches, and handoffs.
- Aggregation, sampling, clipping, and virtualization for long sequences and dense models.

## Narration And Production

- First-class narration segments, audio tracks, cue bookmarks, captions, and subtitles.
- Cue-aligned clips and machine-readable render manifests.
- Reusable teaching layouts for title, equation, diagram, comparison, and recap shots.
- A deterministic author-render-inspect-revise workflow using screenshots and scene metadata.

## Authoring Architecture

- Stable semantic scene names alongside numeric tattva IDs.
- A versioned JSON `SceneSpec` with source locations, validation, and repair diagnostics.
- `murali validate`, `render`, `doctor`, and `inspect` commands built on that declarative boundary.
- Tighter public module boundaries and smaller animation modules with clear ownership.
- Scene-owned themes and deliberate removal or integration of unused global state.
- Consolidated process-diagram APIs and overridable lesson templates.

## Renderer And Scale

- Order-independent transparency for intersecting translucent 3D geometry.
- Reused line buffers and bind groups, plus batched uniform uploads.
- A deliberate renderer mesh-cache policy and complete text-resource caching.
- Tiling and resource policies for large raster-backed text and code surfaces.
- Separate CPU projection and GPU upload measurements, backed by representative AI benchmarks.

## Project Sustainability

- A deliberate crates.io examples-packaging policy.
- Contribution and security policies, changelog discipline, and release automation.
- Continued synchronization between the website, API docs, examples, and release notes.

The detailed maintainers' version of this plan lives in the repository's `ROADMAP.md`.
