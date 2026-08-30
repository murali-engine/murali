# Python API Inventory

This inventory tracks the path from the current Rust engine surface to a coherent Python-first
`murali-engine` API. Murali `0.3.0` is the target for the first coherent Python release line.

The goal is not to expose every Rust type. The goal is to expose enough stable, general engine
surface for Python examples, documentation, and `murali-kit` to work naturally.

## Decision Rules

| Question | Decision |
| --- | --- |
| Is it an atomic primitive, runtime concept, structural math object, or stable scene building block? | Keep in `murali-engine`. |
| Is it a composed visual, lesson helper, design preset, authoring recipe, or fast-moving experiment? | Put in `murali-kit`. |
| Is it useful in Rust but not ready for Python docs or stable examples? | Leave Rust-only for now. |
| Is it needed by the selected Python examples for `0.3.0`? | Prioritize the binding. |

Practical shorthand:

```text
If it is the language, keep it in the engine.
If it is a sentence, template, lesson, or style, put it in the kit.
```

## Current Python Surface

The Python extension is registered in `src/python/module.rs` and currently exposes these classes:

| Area | Python classes |
| --- | --- |
| Scene lifecycle | `Scene`, `SceneView`, `TattvaHandle` |
| Timeline | `Timeline`, `AnimationBuilder` |
| Shapes | `Circle`, `Square`, `Rectangle`, `Polygon`, `Line`, `Arrow`, `Path`, `TracedPath` |
| Text | `Label`, `CodeBlock`, `Latex`, `Typst` |
| Math visuals | `NumberLine`, `OptimizationPath2D` |
| Axes and tables | `Axes`, `NumberPlane`, `Table` |
| 3D | `Axes3D`, `ParametricCurve3D`, `ParametricSurface`, `Prop3D` |
| Style constants | RGBA tuples; named palettes such as `WHITE` / `GOLD_C` live in kit |

Current Python examples:

| Example | Status | Notes |
| --- | --- | --- |
| `python/examples/hello_shapes.py` | Works against current surface | Exports one PNG by default. |
| `murali-kit/examples/motion_basics.py` | Works against current surface | Move, scale, rotate, fade, and easing on engine primitives. |

Current Python tests cover basic scene additions, timeline creation, layout helpers, text/math/table
objects, 3D objects, and scene views.

## Inventory

| Area | Rust API / Feature | Python exposed? | Needed for 0.3.0 example parity? | Owner | Priority | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| Packaging | `murali-engine` via Maturin/PyO3 | Yes | Yes | Engine | P0 | CI builds prebuilt wheels for macOS arm64/x64, Linux x64/aarch64, and Windows x64. |
| Import surface | `murali_engine` module | Yes | Yes | Engine | P0 | Keep stable through `0.2.x`; breaking cleanup waits for `0.3.0`. |
| Scene lifecycle | `Scene::new`, `play`, `preview`, capture/export path | Partial | Yes | Engine | P0 | `preview()`, `save_png()`, and video `export()` / `export_video()` work from Python. |
| Scene ownership | Scene is consumed by `preview()`, `save_png()`, and child `SceneView` creation | Yes | Yes | Engine | P0 | Decide whether Python should get non-consuming preview/export helpers for coherence. |
| Frames | landscape, portrait, square | Partial | Yes | Engine | P0 | Python accepts frame names and exposes `frame()` / `frame_size()`. |
| Camera | camera position/target/up, perspective projection, timeline camera frames | Partial | Yes for 3D | Engine | P0 | Python exposes direct camera setup, `set_view_width(...)`, `Timeline.animate_camera_frame(...)`, and `Timeline.zoom_camera(...)`. |
| Transforms | position, scale, rotation, opacity, layer, depth mode | Partial | Yes | Engine | P0 | Python exposes setter methods on `Scene`; object-level transform builders are not exposed. |
| Layout | `to_edge`, `next_to`, `align_to`; groups/stacks in Rust | Partial | Yes | Engine + Kit | P0 | Core layout helpers stay in engine, including `Scene.position(...)` and `Scene.world_bounds(...)`. `Group`, `HStack`, and `VStack` live in `murali-kit`. |
| Timeline basics | `Timeline`, animation builder, start/duration/ease | Partial | Yes | Engine | P0 | Python has a single builder flow ending in `spawn()`, plus signal playback, camera frame/zoom helpers, and `play_epicycles(...)` for current parity examples. Needs ergonomic review. |
| Animation kinds | appear, move, rotate, scale, fade, draw, text/table/surface writes, indicate, morph | Partial | Yes | Engine | P0 | `morph_from` is exposed so kit collections can interpolate path geometry. Remaining advanced Rust animations should be added only when examples require them. |
| Timeline composition | clips, overlay, append, wait, callbacks | Partial | Likely | Engine | P1 | `Timeline.call_during(...)` and `Timeline.call_at(...)` exist so kit collections can animate without becoming engine tattvas. Broader clip composition can wait. |
| Updaters | frame/time-dependent scene updates | Partial | Maybe | Engine | P1 | `Scene.add_updater(...)` lets kit collections mutate primitives each frame. Python callbacks must not store the tick object. |
| Basic shapes | circle, square, rectangle, polygon, line, arrow, path | Yes | Yes | Engine | P0 | Keep in engine. |
| Extra primitives | rounded rectangle, chat bubble, ellipse, cube, noisy circle, noisy horizon, particle belt | Partial | Maybe | Engine + Kit | P1 | `RoundedRectangle` and `ChatBubble` are exposed as engine primitives for kit composites; stylized/noisy components still belong in kit unless they become atomic geometry. |
| Text | label, code block, LaTeX, Typst | Yes | Yes | Engine | P0 | CodeBlock exposes theme, surface, title, controls, line numbers, content box size, and content offset. `Typst.outline_points(...)` samples formula geometry for path-based examples. `Typst.vector_paths(...)` and `Latex.vector_paths(...)` compile formulas into filled morphable path glyphs. |
| 3D text | `Letter3D`, `LetterParticles3D` | Yes | Yes | Engine | P0 | Extruded capitals and glyph particle clouds are engine primitives used by the kit `Opening` composite. |
| Tables | `Table` | Yes | Yes | Engine | P0 | Keep in engine as structural data display. |
| Axes | `Axes`, `Axes3D`, `NumberPlane` | Yes | Yes | Engine | P0 | Keep in engine as structural math objects. |
| Graphs | function graph, parametric curve, vector field, stream lines, scatter plot, legend, number line | Partial | Yes | Engine + Kit | P0/P1 | Engine keeps axes/planes/paths. `FunctionGraph`, `ScatterPlot`, `PlotLegend`, and `VectorField` live in `murali-kit`. Streamlines still pending. |
| Surfaces | parametric surface, surface render mode | Partial | Yes for 3D examples | Engine | P0 | Python exposes named surfaces, `ParametricSurface.from_function(...)` for sampled callback surfaces, and native `from_map_projection(...)` for lon/lat morphs. `Scene.update_parametric_surface(...)` replaces geometry. |
| AI / LLM | `ContextBlock`, `ContextWindow`, `SignalFlow` | Partial | Yes | Engine + Kit | P0 | Context window and path signal flow stay engine tattvas for now. `KvCacheView` and tensor snapshots live in `murali-kit`. |
| Assets | `Prop3D`, GLB/GLTF loading, textures | Partial | Yes for 3D | Engine | P1 | `Prop3D.from_file` is exposed along with `center()`, `dimensions()`, and bounds. Built-in texture/material API needs classification. |
| Default textures | built-in primitive textures and material textures | Partial | Yes | Engine | P1 | `Letter3D.with_texture(...)` and `ParametricSurface.with_texture(...)` accept builtin names including `earth`. Designed texture packs belong in kit. |
| Themes | dark/light selection and named theme packs | No direct Python theme API | Yes | Kit | P1 | Theme selection moves to `murali-kit`; engine keeps explicit colors/background/style primitives. |
| Colors | named authoring colors | Yes | Yes | Kit | P0 | Engine accepts RGBA tuples. `murali_kit.colors` owns names. |
| Math notation | equation layout, equation parts, matrix notation, equation continuity, matrix focus steps | Partial | Yes for parity | Engine + Kit | P1 | `EquationPart`, `EquationLayout`, `Matrix`, and the current equation/matrix animation steps are exposed. Vectorized Typst/LaTeX equations live in `murali-kit` on top of `vector_paths` and `morph_from`. |
| Linear algebra | vectors, basis, transform, projection, composition, meters, badges | Partial | Maybe | Engine + Kit | P1/P2 | Engine keeps `NumberPlane` style primitives and `Matrix` cell color/highlight. Kit now includes labeled vectors, bases, column combination, and the matrix-as-columns panel. Remaining projection/composition lesson views still pending. |
| Probability | next-token distribution | No | Maybe | Kit first | P2 | Teaching-specific; likely kit unless reduced to general chart primitives. |
| Statistics | normalization view, decision boundary plot | No | Maybe | Kit first | P2 | Teaching/chart composition; keep Rust-only or kit until stable. |
| Information theory | entropy meter | No | Maybe | Kit first | P2 | Teaching composition. |
| Optimization | optimization path | Partial | Maybe | Engine | P2 | `OptimizationPath2D` is exposed as a simple structural path primitive. Broader optimization teaching compositions can stay in kit. |
| Common tensors | tensor, tensor operations, tensor transitions | No | Maybe | Engine + Kit | P2 | Data model may be engine later; teaching views likely kit. Needs an RFC before Python binding. |
| AI components | transformer, KV cache, context window, neural network, agentic flow | No | No for initial coherence | Kit first | P3 | Do not expose broadly in `murali-engine` until the Python core is coherent. |
| Storytelling | stepwise scripts and scene templates | No | Maybe | Kit | P2 | Strong kit candidate. |
| Composite beta | opening, chat input | No | No | Kit | P3 | Do not promote into engine. |
| Logo/card composites | logo, card | No | No | Kit | P3 | Branded/designed components belong in kit. |
| Utility | traced path, screenshot marker | Partial | Maybe | Engine | P1 | `TracedPath` tracks a handle with identity sampling. Screenshot markers remain Rust-only. |

## 0.3.0 Minimum Coherent Engine Surface

The minimum `murali-engine` Python surface for `0.3.0` should cover:

- scene creation, preview, image export, and video export
- landscape, portrait, and square frames
- camera setup for 2D and 3D scenes
- stable handles and scene mutation helpers
- core shapes, paths, text, LaTeX, Typst, code blocks, tables, axes, and number planes
- timeline basics and common animation kinds
- enough layout helpers for readable examples
- 2D graph basics: function graph, parametric curve, scatter plot, legend
- 3D basics: axes, parametric curve, parametric surface, GLB/GLTF prop loading
- explicit colors/background style primitives, with dark/light theme selection supplied by
  `murali-kit`
- a predictable error and validation story

Everything above should have at least one Python example and one smoke test.

## Murali Kit Candidates

The following should default to `murali-kit` unless a later review promotes part of them into the
engine:

- title cards, intro/outro scenes, section headers, callouts
- Fourier epicycle compositions and other teaching reconstructions built from Path/Circle primitives
- vector-field arrow grids and live dipole/charge teaching views
- 2D function graphs, scatter marks, and plot legends built from Path/Line/Label
- 3D title openings assembled from `Letter3D` and `LetterParticles3D`
- 2D labeled vectors, basis arrows, skewed basis grids, coordinate readouts, and dimension badges
- column-combination views, matrix-as-columns panels, matrix-vector flows, transformed grids, determinant area views, projection shadows, angle marks, dot-product meters, span regions, linear-combination views, scalar-multiplication views, and SceneView transform-order lessons
- map-projection morphs that bend a textured Earth surface through equirectangular, sinusoidal, Mollweide, Hammer, and Mercator
- horizontal/vertical stacks and handle groups built from `next_to`, `align_to`, and bounds
- KV-cache occupancy views assembled from rectangles, labels, and paths
- vectorized Typst/LaTeX equations assembled from formula path glyphs
- cards, branded logo compositions (including the transparent bezier Murali mark), and beta openings
- Prop3D model-inspector framing: auto-fit, camera distance, and overlay labels
- chat input and other UI-style composites
- lesson layouts and animation recipes
- AI explainer compositions
- transformer, KV-cache, context-window, and neural-network teaching views
- probability/statistics/information-theory teaching views
- storytelling and stepwise scene builders
- designed visual presets that combine theme, typography, layout, and animation
- dark/light themes and future named theme packs

## Open Decisions

- Should Python object mutator methods return `None` or `self` consistently?
- Should `Scene.preview()` and `Scene.save_png()` consume the scene, or should Python get a
  reusable scene workflow?
- What is the Python video export API shape?
- Do we expose arbitrary Python callbacks for curves, surfaces, timelines, and updaters, or keep
  named/preset functions only for `0.3.0`?
- Should colors remain flat constants or move toward a namespace such as `colors.WHITE`?
- Should engine `Scene` accept `background=` directly, or should background be set through an
  explicit method?
- What shared theme protocol lets `murali-kit` and future premium packages provide themes without
  either package depending on the other?
- Should Rust-only experimental modules remain inside the engine crate but hidden from Python, or
  move physically to kit once Python equivalents exist?

## Immediate Next Steps

1. Choose the canonical Python examples for `0.3.0`.
2. For each example, mark the missing rows in this inventory as blocking or optional.
3. Expose missing engine primitives before adding kit abstractions.
4. Keep `murali-kit` examples running against published `murali-engine` versions whenever possible.
5. Update this file whenever a Python binding is added, moved, or deliberately rejected.
