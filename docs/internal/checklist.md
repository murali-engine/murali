# Murali Python Release Checklist

This checklist tracks the work needed to make Murali coherent as a Python-first
animation system while keeping the Rust engine focused on runtime capabilities.

## 0.3.0 Must Ship

### Murali Engine

- [x] Define the public role of Murali Engine as the runtime and primitive authoring
  layer for mathematical and AI programmatic visualization.
- [x] Keep the engine largely unopinionated. Opinionated teaching components,
  themes, named colors, and authored collections live in Murali Kit.
- [ ] Review the exposed `murali_engine` Python API and make the surface feel
  coherent for Python users.
- [x] Make engine `with_*` methods return `self` where possible, so engine objects
  behave consistently with Murali Kit objects.
- [x] Keep only the engine exports needed for current compatibility, but mark
  kit-shaped exports such as `ContextBlock`, `ContextWindow`, `SignalFlow`, and
  `OptimizationPath2D` as migration candidates.
- [x] Ensure the core engine concepts behind migrated collections have Rust tests,
  Python binding tests, or visual examples where appropriate.
- [x] Improve wheel publishing so users do not need a local Rust toolchain on common
  platforms. Target prebuilt wheels for macOS arm64/x64, Linux x64/aarch64, and
  Windows x64 where feasible.

### Murali Kit

- [x] Define Murali Kit as the opinionated Python authoring layer on top of Murali
  Engine.
- [x] Ensure Murali Kit represents the old Rust reference collection in spirit:
  reusable Python APIs for people building their own animations, not only copied
  examples.
- [x] Review every migrated example and separate reusable component logic from
  example-only orchestration.
- [x] Prefer kit-owned imports for authored collections once kit-side APIs exist.
- [x] Keep Rust reference examples only as migration/comparison material.
- [x] Add compatibility checks that verify Murali Kit can import and use the
  required `murali_engine` symbols.
- [ ] Make the Murali Kit API structure and code organization clean enough for the
  first serious Python-first release.

### Documentation

- [x] Make documentation Python-first.
- [x] Structure public docs around two sibling sections:
  - Murali Engine
  - Murali Kit
- [x] Present Murali Kit prominently for users who want authored components and
  higher-level animation helpers.
- [x] Document Murali Engine as the Python runtime and primitive API surface it
  exposes.
- [x] Keep Core Rust Engine documentation as a secondary subsection under Murali
  Engine for implementation and architecture reference.
- [ ] Regenerate public example exports from Python code.
- [ ] Upload curated video exports where useful and link them from the docs.

## 0.3.x Follow-Up

### Engine Collection Migration

- Do not delete frontend collections blindly.
- Classify each collection module as one of:
  - engine primitive
  - generic renderable
  - kit collection
  - reference only
  - test only
- For each kit collection candidate, review Rust, Python binding, test, docs,
  and example usage before removal.
- Create a kit-side replacement before changing or removing the engine export.
- Update examples to use kit-owned APIs.
- Add or update contract tests.
- Remove or internalize Rust collection code only after no public engine behavior
  depends on it.

### API Architecture

- Deliberate further on the Python export architecture for `murali_engine`.
- Review mathematical helper APIs where Python representations may need a
  different shape than the Rust implementation, especially vector and map
  projection helpers.
- Improve theme interoperability between Murali Engine, Murali Kit, and future
  add-on packages.
- Identify a neutral engine palette base that can be overridden by Murali Kit,
  user themes, or future theme plugins.
- Document accepted string values for options such as frame, direction, anchor,
  depth mode, render mode, playback mode, texture name, and easing.

## 0.4.0 Stabilization

### Murali Engine

- Review ergonomics of all public engine APIs and improve names, return values,
  argument shapes, and error messages where needed.
- Decide which APIs are stable enough to document as long-term contracts.
- Further reduce or hide engine surfaces that mainly exist for old Rust
  frontend authoring.

### Murali Kit

- Review all exposed kit APIs for ergonomics and maintainability.
- Make reusable components feel consistent across namespaces.
- Add more examples that demonstrate composing custom animations from kit
  components instead of only running complete demos.
- Start marking stable kit APIs.

### Documentation

- Make the docs more user-friendly.
- Add more task-oriented guides.
- Keep the Rust/internal documentation clearly secondary to the Python user
  journey.

