# Murali

Murali is a **Python-first animation engine** for deterministic, timeline-driven mathematical, AI,
and teaching visuals. You author scenes in Python. A Rust engine underneath does the heavy lifting
for performance, rendering on `wgpu` (Metal, Vulkan, DirectX).

| You want to | Install | Details |
| --- | --- | --- |
| Write Murali scenes | `pip install murali-kit==0.3.0` (pulls `murali-engine==0.3.0`) | [PYTHON.md](./PYTHON.md) |
| Build an integration or authoring layer | Python packages `murali-engine` + `murali-kit` | [PYTHON.md](./PYTHON.md) |
| Embed the runtime directly | Rust crate `murali = "0.3.0"` | [RUST.md](./RUST.md) |

The public authoring layer is Python. The core renderer is written in Rust to squeeze maximum
performance from the GPU stack. If you need a custom workflow, integration, or higher-level visual
toolkit, build it in Python on top of `murali-engine`; use the Rust crate directly only when you
are embedding or extending the runtime itself.

Python APIs are unstable until **0.5.0**. The last first-party **Rust scene-authoring** API is
[`murali` 0.2.4](https://crates.io/crates/murali/0.2.4) ([docs](https://muraliengine.com/docs/0.2.4/intro)).

Site: [muraliengine.com](https://muraliengine.com).

## Python (authoring + integration)

```bash
python3 -m pip install murali-kit==0.3.0
```

```python
from murali_engine import Circle, Label, Scene
from murali_kit.colors import GREEN_D, WHITE
from murali_kit.themes import DarkTheme, apply_theme

scene = apply_theme(Scene(), DarkTheme())
scene.add(Label("Hello Murali", height=0.38, color=WHITE), at=(0.0, 2.4, 0.0))
scene.add(Circle(radius=1.2, color=GREEN_D).with_stroke(0.04, WHITE))
scene.preview()
```

`murali-engine` is the Python frontend over the Rust runtime: scene, tattvas, timeline, camera,
preview, and export. `murali-kit` adds themes, named colors, reusable teaching views, and examples.

More: wheels, frames, export, maturin, kit examples — **[PYTHON.md](./PYTHON.md)**.

## Development

Murali uses [uv](https://docs.astral.sh/uv/) for its Python development environment and lockfile.
From a checkout:

```bash
uv sync
uv run pytest python/tests
```

After changing the Rust bindings, rebuild the editable extension with
`uv run maturin develop --features python`.

End-user wheels remain standard Python packages and do not require uv.

## Rust (runtime)

```toml
[dependencies]
murali = "0.3.0"
anyhow = "1"
glam = "0.33"
```

Use the crate directly when you are embedding Murali in a Rust program, extending the renderer, or
building a lower-level runtime integration.

More: crate layout, `murali.toml`, Cargo examples, experimental features — **[RUST.md](./RUST.md)**.

## Videos

[![Shapes](./resources/shapes.png)](https://youtu.be/rzQZHta2PQM)
[![Animation showcase](./resources/animation_showcase.png)](https://youtu.be/W8WQQbSo70Y)

## License

Murali is dual-licensed under either the MIT License or the Apache License, Version 2.0.
