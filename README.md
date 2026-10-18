# Murali

This repository contains Murali's **Rust-based animation engine** for deterministic,
timeline-driven mathematical, AI, and teaching visuals. Its engine, scene model, timeline, and
renderer are written in Rust and render through `wgpu` (Metal, Vulkan, DirectX). Experimental
Python bindings are available for early authoring and integration work.

| You want to | Install | Details |
| --- | --- | --- |
| Build with or extend the engine | Rust crate `murali = "0.3.0"` | [Rust engine guide](./RUST.md) |
| Explore the experimental Python API | `pip install murali-kit==0.3.0` (pulls `murali-engine==0.3.0`) | [Experimental Python support](./PYTHON.md) |

Within this repository, the Rust crate is the primary engine and runtime surface. Use it to embed
Murali, extend rendering, or run computation- and performance-intensive workloads. The Python
packages expose an experimental subset of the engine; expect their APIs to change.

Python support is experimental and its APIs are unstable until at least **0.5.0**. For the stable,
high-level Rust scene-authoring API, see [`murali` 0.2.4](https://crates.io/crates/murali/0.2.4)
and the [Murali documentation](https://muraliengine.com/docs/intro).

Documentation: [Rust documentation site](https://muraliengine.com/) · [current Rust engine](./RUST.md)
· [experimental Python support](./PYTHON.md).

## Murali and Veena

Murali began as an exploration of how to build a capable programmatic animation system. After
experimenting with Python, C++, and Rust, we settled on Rust and developed the animation engine in
this repository. We then used it to create real content and learn what an authoring system needs in
practice.

That experience led us to create **Veena**, a separate JavaScript animation engine. JavaScript proved especially powerful
for general-purpose authoring because the browser provides an extraordinarily broad visual
platform: Canvas, SVG, WebGL, WebGPU, text and layout engines, media APIs, and the DOM. It also lets
Veena build on existing frontend frameworks such as React instead of recreating those capabilities
inside a native engine.

We also found that current AI systems tend to generate and work with JavaScript more effectively
than Rust, which makes AI-assisted authoring substantially stronger. Matching the browser's breadth
inside the Rust engine would require rebuilding a large part of the browser platform.

Rust remains the better tool for computation-heavy and performance-critical work. Depending on the
workload, native Rust can be orders of magnitude faster and more resource-efficient than a
JavaScript implementation.

For browser-native and JavaScript authoring, see [Veena](https://veenaengine.com/). Murali remains
the Rust engine for workloads that require intensive computation, predictable native performance,
or lower-level control. The two engines have distinct names and websites: Murali lives at
[muraliengine.com](https://muraliengine.com/) and Veena lives at
[veenaengine.com](https://veenaengine.com/).

## Rust engine

```toml
[dependencies]
murali = "0.3.0"
anyhow = "1"
glam = "0.33"
```

Use the crate when embedding Murali in a Rust program, extending the renderer, or building an
authoring or runtime integration.

More: crate layout, `murali.toml`, Cargo examples, experimental features — **[RUST.md](./RUST.md)**.

## Experimental Python support

```bash
python3 -m pip install murali-kit==0.3.0
```

```python
from murali_engine import Circle, Label, Scene
from murali_kit.colors import GREEN_D, WHITE
from murali_kit.themes import DarkTheme

scene = Scene()
scene.apply_theme(DarkTheme())
scene.add(Label("Hello Murali", height=0.38, color=WHITE), at=(0.0, 2.4, 0.0))
scene.add(Circle(radius=1.2, color=GREEN_D).with_stroke(0.04, WHITE))
scene.preview()
```

`murali-engine` provides experimental Python bindings for scenes, tattvas, timelines, cameras,
preview, and export. `murali-kit` adds themes, named colors, reusable teaching views, and examples.

More: wheels, frames, export, maturin, kit examples — **[PYTHON.md](./PYTHON.md)**.

## Development

Development of the optional Python bindings uses [uv](https://docs.astral.sh/uv/) for its
environment and lockfile. From a checkout:

```bash
uv sync
uv run pytest python/tests
```

After changing the Rust bindings, rebuild the editable extension with
`uv run maturin develop --features python`.

End-user wheels remain standard Python packages and do not require uv.

## Videos

[![Shapes](./resources/shapes.png)](https://youtu.be/rzQZHta2PQM)
[![Animation showcase](./resources/animation_showcase.png)](https://youtu.be/W8WQQbSo70Y)

## License

Murali is dual-licensed under either the MIT License or the Apache License, Version 2.0.
