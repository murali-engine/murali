# Changelog

All notable changes to Murali are recorded here.

## 0.1.7 - 2026-08-09

### Added

- Local-time `Clip` composition onto one deterministic global timeline.
- Stable authored render layers, improved opaque and transparent 3D depth behavior, and exact
  terminal export sampling.
- Deterministic backward seeking with explicit behavior for reversible callbacks, one-shot
  callbacks, updaters, and traced paths.
- Semantic tensor snapshots, selectors, transitions, matrix operations, reshaping, split/merge,
  broadcasting, deterministic sampling, higher-rank slicing, and explicit rank-2 projections.
- Versioned JSON AI trace ingestion with tokens, tensors, model metadata, and typed events.
- Semantic transformer stages and a JSON-backed end-to-end self-attention lesson.
- Centralized validation for timelines, streamlines, and parametric surfaces.
- Rust CI, expanded regression tests, and automatic preview closing for example sweeps.

### Fixed

- Stale geometry after normal mutable scene access and incorrect replacement tattva identity.
- Silent renderer truncation above 1,000 meshes and mesh indices limited to `u16`.
- Layout bounds for rotation, signed scale, moving transforms, and perspective cameras.
- Typst resize-cache keys, textured-surface wireframe visibility, deterministic contour extraction,
  capture ordering, neural-network route activation, and several timeline boundary behaviors.

### Changed

- Upgraded the rendering and supporting dependency stack.
- Aligned release documentation with Rust 1.85, `glam` 0.33, and Apache-2.0 licensing.
