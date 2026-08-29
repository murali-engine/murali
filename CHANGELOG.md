# Changelog

All notable changes to Murali are recorded here.

## 0.2.5 - 2026-08-29

### Added

- Published the first experimental Python engine package as `murali-engine`, imported as
  `murali_engine`.
- Added Python bindings for core scene authoring, timelines, shapes, text, tables, axes, selected
  3D surfaces, SceneView, preview, and PNG export.
- Added Python smoke examples and binding tests for the new package surface.

### Changed

- Clarified the package boundary between Murali Engine, Murali Kit, and future add-on packages.
- Updated installation and Python binding docs for the `0.2.5` release.

## 0.2.4 - 2026-08-22

### Changed

- Renamed the public reusable-component namespace from `sangrah` back to `collection`.
- Renamed the math umbrella from `ganit` to `maths`, so math-facing components now live under
  paths such as `murali::collection::maths::linear_algebra`.
- Updated examples, documentation, and internal module paths to use the commercial-facing
  `collection` and `maths` names.

## 0.2.2 - 2026-08-22

### Added

- Experimental `math::linear_algebra` collection behind the `experimental` Cargo feature, including
  reusable 2D vector, basis, grid transform, matrix-vector flow, determinant, dot product,
  projection, orthogonality, dimension badge, and composition helpers.
- Linear algebra reference examples covering vectors, span, dot product, basis change, matrix
  transforms, matrix-vector flow, column combinations, determinants, composition order, and a
  SceneView-based transform-order comparison.
- Experimental feature documentation and example listings for opt-in linear algebra APIs.

### Changed

- `preview_all.sh` and `export_all.sh` can select examples by tags/ranges and automatically enable
  the `experimental` feature for tagged linear algebra examples.

## 0.2.1 - 2026-08-22

### Added

- `SceneView`, an independently timed child scene that renders as one transformable parent object
  with playback modes, backgrounds, borders, rounded corners, and configurable resolution.
- A `scene_view` reference example and dedicated guide covering child-scene construction, local
  time mapping, playback lifecycle, parent animation, and current compositing limits.
- `Letter3D` support for extruded ASCII capitals with hole-aware font tessellation, independent
  face colors, custom font loading, and image textures.
- `LetterParticles3D` and seekable scatter animations for deterministic glyph-to-particle
  transitions, including configurable destination palettes.
- The beta `composite::beta::opening::Opening` temporal composite for configurable 3D title drops,
  shake-and-burst choreography, particle dissolves, and tagline reveals.
- A `kavriq_opening` example that customizes the beta opening through Rust constants and exports
  it directly as a standalone scene.
- An `opening_scene_view` example that keeps the opening on an independent perspective child scene
  and fades it into continuing orthographic parent content.
- `ChatBubble`, a single-outline message-bubble primitive with configurable left or right tips, and
  the beta `ChatInputBox` composite with typewriter-ready text and an optional send button.
- `Prop3D` support for static local `.glb` and `.gltf` assets, including scene transforms,
  base-color materials and textures, local-space bounds, checked-in demo props, and reference
  examples for both formats.
- A `model_inspector` utility example with automatic centering and framing, continuous rotation,
  orbit and zoom controls, camera and fit options, and repository shorthand for bundled demo props.
- `TextureImage::builtin` and `BuiltinTexture::{BlackMarble, WhiteMarble}` for path-free access to
  Murali's embedded marble textures. `Letter3D` and the beta opening composite accept these reusable
  in-memory textures.
- `ContextWindow`, a role-aware semantic view of assembled model context with token budgets and
  explicit per-block truncation.
- `NextTokenDistribution`, a deterministic next-token teaching view with temperature, top-k, top-p,
  renormalization, and authored unit-interval sampling.
- `KvCacheView` for key/value cache occupancy backed by semantic tensors, plus the seekable
  `kv_cache_fill_to` timeline animation.
- `TensorSnapshot::try_normalized` with LayerNorm and RMSNorm operations, and `NormalizationView`
  for visualizing their input, output, and per-group statistics.
- Runnable examples and Docusaurus guides for the new conversational, 3D prop, opening, SceneView,
  and semantic AI APIs.

### Fixed

- Orbit camera controls now synchronize with the authored camera before interaction, preventing a
  jump on the first drag and keeping scroll zoom centered on the current target.

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
