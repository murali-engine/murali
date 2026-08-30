# Murali Roadmap

Murali's direction is an AI-first Rust animation engine for deterministic technical storytelling.
It should visualize recorded model and system state rather than become an inference framework, and
it should preserve explicit author control while making common educational compositions easier to
build.

This roadmap contains future work only. Completed work belongs in release notes and documentation,
not in the plan.

## P0: Python Coherence And 0.3.0

Murali `0.3.0` is the target for the first coherent Python-first release line. The goal is for
`murali-engine` to feel like a real Python library rather than a partial Rust binding, and for
`murali-kit` examples to run against it without local setup surprises.

- Freeze the intended Python API shape for scenes, objects, timelines, layout, preview, export,
  and examples.
- Inventory Rust examples and engine features against the currently exposed Python surface.
- Keep `0.2.x` releases additive and compatibility-focused; reserve breaking Python API cleanup for
  `0.3.0`.
- Make the Python binding architecture predictable after the `src/python/` split, with consistent
  constructors, validation, return values, and module registration.
- Expose missing APIs only when they are needed by real examples, documentation, or `murali-kit`.
- Classify the existing `frontend::collection` tree before `0.3.0`: keep atomic and structural
  building blocks in the engine, move opinionated compositions and teaching recipes to
  `murali-kit`, and leave unstable surfaces Rust-only until they are ready.
- Move Python-facing theme selection to `murali-kit`; keep engine styling explicit through colors,
  backgrounds, fonts, materials, and conservative renderer defaults.
- Port examples in stages: basic shapes, text, timelines, layout, axes, tables, 3D, scene views,
  and advanced demos.
- Use the examples as the backbone for Python documentation: quickstart, concepts, API guide,
  `murali-kit` guide, and the hidden Core Rust Engine reference.
- Add release confidence checks for Python import, Python examples, `murali-kit` examples, Maturin
  wheel builds, and normal Rust tests.

The working inventory for this effort lives in `docs/internal/python-api-inventory.md`.

## P0: Reliability And Quality Gates

- Add Linux, macOS, and headless-render coverage for the `wgpu` runtime.
- Add headless golden-image tests for layering, opacity, text, paths, and representative AI scenes.
- Add property tests for mesh limits, transformed layout bounds, capture ordering, and invalid
  author input.
- Extend structured validation and projection diagnostics across remaining token, attention,
  dimension, label, color, and route APIs; remove silent no-ops and runtime `println!` failures.
- Clear standard compiler warnings, then expand strict Clippy beyond correctness and suspicious
  lints.
- Add dependency and documentation-sample auditing to CI.

## P1: AI Teaching Semantics

- Write a short domain and ownership RFC covering tensor snapshots, tokens, operations, selectors,
  semantic IDs, trace events, and the boundary between imported data and authored visuals.
- Add a general 2D image tattva and image-grid composition for datasets, feature maps, and
  multimodal lessons.
- Add semantic transformer internals for residual streams, MLPs, and mixture-of-experts routing.
- Add an autoregressive-generation composition that connects successive next-token distributions,
  selected tokens, appended context, and the repeat-until-stop streaming cycle.
- Add computation-graph state with deterministic forward and reverse-mode playback.
- Add parameter, gradient, loss-series, optimizer-step, embedding, activation, feature-map, and
  attribution views.
- Produce one complete backpropagation lesson that shares semantic objects across equations,
  tensors, network structure, and plots.
- Define recorded agent and RAG events for retrieval, tools, memory, retries, branches, and
  handoffs, with adapters layered over the general AI trace contract.
- Add scalable aggregation, sampling, clipping, and virtualization policies for long token
  sequences, dense attention, and large networks.

## P1: Narration And Production

- Make narration segments, audio tracks, cue bookmarks, captions, and subtitle import/export
  first-class timeline data.
- Add cue-aligned clip helpers and a machine-readable render manifest containing durations,
  captures, diagnostics, and artifact paths.
- Add reusable, overridable teaching layouts for title, equation, diagram, comparison, and recap
  shots.
- Support a deterministic author-render-inspect-revise workflow using screenshots and scene
  metadata.

## P2: Authoring Architecture

- Add stable semantic scene names alongside numeric `TattvaId` values.
- Define a versioned `SceneSpec` with stable string IDs, source locations, JSON ingestion,
  structured repair diagnostics, and Rust as the escape hatch.
- Add `murali validate`, `render`, `doctor`, and `inspect` commands once `SceneSpec` establishes a
  useful CLI boundary.
- Keep backend, ECS, and low-level resource modules private unless they are intentionally part of
  the compatibility contract.
- Split the large animation module by property, geometry, text, table, surface, and composition
  ownership.
- Remove or deliberately connect unused global/configuration state, and make themes owned by a
  scene or render job instead of a process-global singleton.
- Consolidate `Stepwise` and `AgenticFlowChart`, move branded compositions out of the core AI
  teaching inventory, and replace color-only templates with overridable lesson templates.

## P2: Renderer And Scale

- Add order-independent transparency for intersecting or self-overlapping translucent 3D geometry.
- Reuse line storage buffers and bind groups, and batch uniform uploads instead of writing once per
  drawable.
- Implement or remove the unused renderer mesh cache.
- Cache LaTeX GPU resources and Typst bind groups with complete render-target keys.
- Establish tiling and resource policies for large raster-backed text and code surfaces.
- Measure CPU projection and GPU upload independently, then add benchmarks for dense attention,
  neural networks, text, and Stepwise scenes.

## P3: Project Sustainability

- Decide whether reference examples should remain excluded from crates.io packages.
- Add a contribution guide, security policy, changelog discipline, and release automation.
- Keep the website, API docs, examples, and release notes synchronized whenever public behavior
  changes.
