---
sidebar_position: 7
---

# Python Bindings

Murali packages its Python engine as `murali-engine`, imported as `murali_engine`.
The package is built from this repository with PyO3 and maturin, and it exposes a designed
authoring surface rather than a direct mechanical export of every Rust type.

The goal is to keep one Rust engine and expose a Pythonic layer for scene authoring, preview, and
export. The Python API is still experimental, but it is real enough for checked examples and early
user feedback.

## Target Shape

The Python API should preserve Murali's mental model while using Python-friendly constructors,
handles, and conversions:

```python
from murali_engine import Scene, Circle, Label, GREEN, WHITE, Timeline

scene = Scene()

title = scene.add(
    Label("Hello Shapes", height=0.38, color=WHITE),
    at=(0, 2.8, 0),
)

circle_shape = Circle(radius=0.7, color=GREEN, segments=48)
circle_shape.with_stroke(0.04, WHITE)

circle = scene.add(circle_shape, at=(-1.8, 0.4, 0))

timeline = Timeline()
timeline.animate(title).at(0.0).for_duration(0.5).typewrite_text().spawn()
timeline.animate(circle).at(0.2).for_duration(0.8).ease("out_cubic").move_to((-0.2, 0.4, 0)).spawn()
scene.play(timeline)

scene.save_png("rendered_output/python_examples/hello_shapes.png", width=960)
```

This overlaps with familiar animation-authoring patterns, but the underlying model should remain
Murali's: explicit scenes, explicit timelines, semantic components, deterministic Rust-side
rendering, and reusable collection modules.

## Current Experimental Slice

Behind the `python` Cargo feature, Murali currently exposes:

- `Scene`
- `TattvaHandle`
- `Timeline`
- `AnimationBuilder`
- `Label`
- `Circle`
- `Square`
- `Rectangle`
- `Polygon`
- `Line`
- `Arrow`
- `Path`
- `CodeBlock`
- `Latex`
- `Typst`
- `Axes`
- `NumberPlane`
- `Table`
- `Axes3D`
- `ParametricCurve3D.named(...)`
- `ParametricSurface.named(...)`
- `Prop3D.from_file(...)`
- `SceneView`
- `Scene(frame="landscape" | "portrait" | "square")`
- `Scene.set_frame(...)`
- `Scene.frame()`
- `Scene.frame_size()`
- `Scene.to_edge(...)`
- `Scene.next_to(...)`
- `Scene.align_to(...)`
- `Scene.set_camera(...)`
- `Scene.set_perspective_camera(...)`
- `Scene.set_depth_mode(...)`
- `Scene.set_position(...)`
- `Scene.set_scale(...)`
- `Scene.set_rotation_z(...)`
- `Scene.add_scene_view(...)`
- `Scene.preview()`
- `Scene.save_png(...)`
- `Scene.play(timeline)`
- timeline animations for `appear`, `draw`, `undraw`, `move_to`, `rotate_to`, `scale_to`,
  `fade_to`, `typewrite_text`, `untypewrite_text`, `reveal_text`, `hide_text`, `indicate`,
  `write_table`, `unwrite_table`, `write_surface`, and `unwrite_surface`
- the named Murali color palette as RGBA tuples, including shade constants such as `GRAY_B`,
  `BLUE_D`, and `GOLD_C`

Full video export is part of the target API, but it is not exposed to Python yet. `preview()` and
`save_png()` currently consume the scene because the Rust runtime owns the scene during rendering.

## Ergonomics

Use hybrid ergonomics:

- keyword constructors for common fields
- chainable `with_*` methods for optional refinements
- tuple/list conversion for vectors
- color constants and later hex-string color parsing
- Python exceptions mapped from Rust errors
- scene handles instead of exposing raw Rust lifetimes or requiring users to manage `TattvaId`

Rust-style builders usually consume and return `Self`. Python wrappers should instead mutate the
wrapped Rust object and return `self`, so this remains natural:

```python
Circle(radius=0.7, color=GREEN_D).with_stroke(0.04, WHITE)
```

## Packaging

For local engine development, create a virtual environment in this repository and install the
extension module with maturin:

```bash
python3 -m venv .venv
.venv/bin/python -m pip install maturin
.venv/bin/maturin develop --features python
.venv/bin/python python/examples/hello_shapes.py
```

For release builds and publishing:

```bash
.venv/bin/maturin build --release --features python
.venv/bin/maturin publish --features python
```

After the first PyPI release, users should install the engine package directly:

```bash
python3 -m pip install murali-engine
```

Companion Python examples and add-on experiments live in the `murali-kit` repository. `murali-kit`
depends on `murali-engine`; installing `murali-engine` does not install the kit.

## First Release Boundary

The first PyPI release should be explicitly experimental, but it should not feel like a toy binding.

The release is ready when Python can:

- create a scene
- add core shapes, text, math text, tables, axes, simple paths, and basic 3D surfaces
- animate them with the timeline builder
- preview or export a PNG
- run checked-in examples from `murali-engine` and `murali-kit`

After that, expand gradually into richer Python-side callbacks, notebook-friendly display helpers,
video export wrappers, and optional domain packages.

## Complications

The hard parts are not PyO3 syntax. The hard parts are API design and runtime boundaries:

- Rust ownership and borrowing need Python wrapper handles.
- Python users expect objects to be held and mutated freely.
- callbacks and updaters require careful GIL and performance handling, so they should come later.
- Rust `Result` errors need useful Python exceptions.
- WGPU preview/export behavior needs to be tested from installed wheels.
- wheels for macOS, Linux, and Windows need their own CI path before PyPI is treated as stable.

The right first milestone is a coherent Python authoring surface backed by the existing Rust engine:
useful enough for real examples, still honest about what is experimental, and small enough to
iterate after PyPI feedback.
