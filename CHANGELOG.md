# Changelog

All notable changes to Murali are recorded here.

## 0.2.0 - 2026-08-09

### Added

- First-class `Frame::portrait()` support for intentionally composed 9:16 Shorts and vertical
  videos, including matching preview windows, layout bounds, camera projection, and exports.
- `Frame::square()` support for square compositions, while retaining `Frame::landscape()` as the
  default 16:9 frame.
- A runnable `portrait_video` reference example for 1080x1920 output.

### Changed

- The scene frame is now the single source of truth for aspect ratio. Export configuration accepts
  a literal pixel `width`, and Murali derives output height from the selected frame.
- `ExportSettings::height` and `RenderOptions::resolution` were replaced by frame-derived height
  and `RenderOptions::width`, respectively.

## 0.1.8 - 2026-08-09

### Fixed

- Restored the intended `MIT OR Apache-2.0` dual-license declaration and included both canonical
  license texts in the repository and published package.
- Corrected the documentation release metadata so `0.1.8` is the explicit stable version across
  the version selector, installation snippets, and frozen documentation.
- Added a CI release-metadata check to keep crate, lockfile, documentation, and license signals in
  sync.

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
- Declared the package metadata as Apache-2.0 to match the repository's existing license file.
