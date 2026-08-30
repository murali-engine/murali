---
sidebar_position: 7
---

# Python API

`murali-engine` is the Python runtime, imported as `murali_engine`. It is a designed authoring
surface, not a mechanical dump of every Rust type.

Write scenes in Python. For themes, named colors, and teaching views, also install
[Murali Kit](./murali-kit).

```python
from murali_engine import Scene, Circle, Label, Timeline
from murali_kit.colors import GREEN_D, WHITE
from murali_kit.themes import DarkTheme, apply_theme

scene = apply_theme(Scene(), DarkTheme())

title = scene.add(Label("Hello Shapes", height=0.38, color=WHITE), at=(0, 2.8, 0))
circle = scene.add(
    Circle(radius=0.7, color=GREEN_D, segments=48).with_stroke(0.04, WHITE),
    at=(-1.8, 0.4, 0),
)

timeline = Timeline()
timeline.animate(title).at(0.0).for_duration(0.5).typewrite_text().spawn()
timeline.animate(circle).at(0.2).for_duration(0.8).ease("out_cubic").move_to((-0.2, 0.4, 0)).spawn()
scene.play(timeline)
scene.save_png("hello_shapes.png", width=960)
```

## Authoring conventions

- keyword constructors for common fields (`Circle(radius=0.7, color=...)`)
- chainable `with_*` methods that return the same object
- positions as `(x, y, z)` tuples
- colors as `(r, g, b, a)` tuples; names in `murali_kit.colors`
- `scene.add(...)` returns a handle; animate that handle
- ease names are strings: `linear`, `in_quad`, `out_quad`, `in_out_quad`, `in_cubic`, `out_cubic`,
  `in_out_cubic`

## Scene and output

```python
Scene()
Scene(frame="landscape" | "portrait" | "square")
scene.to_edge(handle, "up", margin=0.8)
scene.next_to(a, b, "right", padding)
scene.align_to(a, b, "center")
scene.set_view_width(16.0)
scene.set_camera(...)
scene.set_perspective_camera(fov_y_degrees, near, far)
scene.hide(handle) / scene.show(handle)
scene.play(timeline)
scene.preview()
scene.preview(auto_close=True, hold=3.0)
scene.save_png(path, width=None, fps=None, duration=None)
scene.export_video(path, width=None, fps=None, duration=None, preserve_frames=False)
```

`preview()`, `save_png()`, `export_video()`, and `export()` consume the scene.

## Timeline

```python
timeline.animate(handle).at(t).for_duration(d).ease("out_cubic").move_to((x, y, z)).spawn()
```

Verbs include `appear`, `draw`, `undraw`, `move_to`, `rotate_to`, `rotate_xyz`, `scale_to`,
`fade_to`, `typewrite_text`, `untypewrite_text`, `reveal_text`, `hide_text`, `indicate`,
`morph_from`, table and surface write/unwrite, plus `play_signal`, `animate_camera_frame`,
`zoom_camera`, `call_during`, `call_at`, and `wait_until`.

Always `.spawn()`.

## Engine objects

Core types currently on `murali_engine`:

`Scene`, `Timeline`, `Label`, `Letter3D`, `LetterParticles3D`, `Circle`, `Square`, `Rectangle`,
`RoundedRectangle`, `ChatBubble`, `Polygon`, `Line`, `Arrow`, `Path`, `ParticleBelt`, `CodeBlock`,
`Latex`, `Typst`, `Axes`, `NumberLine`, `NumberPlane`, `TracedPath`, `Table`, `Axes3D`,
`ParametricCurve3D`, `ParametricSurface`, `Prop3D`, `SceneView`, `EquationLayout`, `EquationPart`,
`Matrix`.

`with_stroke(...)`, `with_color(...)`, and other `with_*` methods return `self`.

## Kit-shaped leftovers

These still import from `murali_engine` for compatibility. Prefer kit once you are in murali-kit:

- `ContextBlock` / `ContextWindow` → `murali_kit.ai`
- `OptimizationPath2D` → `murali_kit.maths`
- `SignalFlow` — still the engine tattva until kit is the default

Named colors are not engine exports.

## Install

```bash
python3 -m pip install murali-engine==0.2.6
python3 -m pip install murali-kit==0.1.1
```

Prebuilt wheels cover macOS arm64 and x86_64, Linux x86_64 and aarch64, and Windows x86_64. Those
installs do not need a local Rust toolchain. Other platforms can still build from the source
distribution if Rust 1.85+ is available.

Local engine development from this repository:

```bash
python3 -m venv .venv
.venv/bin/python -m pip install maturin
.venv/bin/maturin develop --features python
.venv/bin/python python/examples/hello_shapes.py
```

Release wheels are built in CI. Pushing a `v*` tag publishes them to PyPI. The Rust crate is
published with `cargo publish`. See `RELEASE.md`.

## Related

- [Your First Scene](./first-scene)
- [Tattvas](./tattvas/)
- [Teaching views](./murali-kit)
- [Which API Should I Use?](./which-api-should-i-use)
