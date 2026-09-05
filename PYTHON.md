# Murali Python authoring

First-party scene authoring is Python. The Rust runtime underneath is built for rendering
performance, while `murali-engine` exposes the Python frontend layer used by scenes and custom
integrations.

| Package | Import | Role |
| --- | --- | --- |
| `murali-engine==0.3.0` | `murali_engine` | Python frontend: scene, primitives, timeline, camera, preview, export |
| `murali-kit==0.3.0` | `murali_kit` | Themes, named colors, teaching views, examples |

APIs are unstable until **0.5.0**.

## Install

```bash
python3 -m pip install murali-kit==0.3.0
```

That pulls a compatible engine wheel on macOS arm64/x86_64, Linux x86_64/aarch64, and Windows
x86_64. No local Rust toolchain on those platforms.

Engine only:

```bash
python3 -m pip install murali-engine==0.3.0
```

Use `murali-engine` directly when you are building your own integration or toolkit and do not want
kit opinions. Use the Rust crate only when you need lower-level runtime embedding
([RUST.md](./RUST.md)).

## A first scene

```python
from murali_engine import Circle, Label, Scene, Timeline
from murali_kit.colors import GREEN_D, WHITE
from murali_kit.themes import DarkTheme, apply_theme

scene = apply_theme(Scene(), DarkTheme())
title = scene.add(Label("Hello Murali", height=0.38, color=WHITE), at=(0.0, 2.4, 0.0))
circle = scene.add(Circle(radius=1.2, color=GREEN_D).with_stroke(0.04, WHITE))

timeline = Timeline()
timeline.animate(title).at(0.0).for_duration(1.0).typewrite_text().spawn()
scene.play(timeline)
scene.preview()
```

`preview()`, `save_png(...)`, and `export_video(...)` consume the scene — call one of them.

```python
Scene()
Scene(frame="portrait")
Scene(frame="square")
scene.save_png("frame.png", width=1920)
scene.export_video("scene.mp4", width=1920, fps=60)
```

Walkthrough: [Your first scene](https://muraliengine.com/docs/first-scene).

## Examples

Python examples live in the kit repo, not in `murali/examples/`:

[murali-kit/examples](https://github.com/murali-engine/murali-kit/tree/main/examples)

```bash
python examples/hello_shapes.py
python preview_all.py --auto
```

## Develop bindings from this repo

```bash
python3 -m venv .venv
.venv/bin/python -m pip install maturin
.venv/bin/maturin develop --features python
```

Kit against an adjacent engine checkout: see
[murali-kit](https://github.com/murali-engine/murali-kit).
