---
sidebar_position: 2
---

# Installation

Install Murali Kit. That is the usual authoring path, and it pulls in a compatible engine:

```bash
python3 -m pip install murali-kit
```

Then:

```python
from murali_engine import Scene
from murali_kit.themes import DarkTheme, apply_theme

scene = apply_theme(Scene(), DarkTheme())
scene.preview()
```

Pinned versions:

```bash
python3 -m pip install murali-engine==0.3.0
python3 -m pip install murali-kit==0.3.0
```

`murali-kit` depends on `murali-engine>=0.3.0,<0.4.0`. Installing the engine alone does not install
the kit.

## What you get

| Package | Import | Role |
| --- | --- | --- |
| `murali-kit` | `murali_kit` | Themes, named colors, teaching views, examples |
| `murali-engine` | `murali_engine` | Scene, primitives, timeline, preview, export |

Python 3.10 or newer. A GPU-capable graphics environment for preview. `ffmpeg` if you want MP4 or
GIF export. `latex` and `dvisvgm` only if you use `Latex`. Typst is embedded; it does not need a
system install.

## Prebuilt wheels

`murali-engine` ships prebuilt wheels for:

- macOS arm64 and x86_64
- Linux x86_64 and aarch64
- Windows x86_64

Those installs do not need a local Rust toolchain. Other platforms can still build from the source
distribution if [Rust 1.85+](https://www.rust-lang.org/tools/install) is available.

## Preview vs export

Preview (`scene.preview()`):

- needs a working graphics environment
- does not require `ffmpeg`

Export:

- `scene.save_png(path)` always writes a PNG
- `scene.export_video(path)` uses `ffmpeg` to assemble MP4 or GIF
- if `ffmpeg` is missing, Murali still writes frames and tells you where they landed

`preview()`, `save_png()`, `export_video()`, and `export()` consume the scene. Call one of them at
the end of the script.

## Local engine development

Only if you are changing the engine itself, from a checkout of this repository:

```bash
python3 -m venv .venv
.venv/bin/python -m pip install maturin
.venv/bin/maturin develop --features python
.venv/bin/python python/examples/hello_shapes.py
```

Kit examples live in the [`murali-kit`](https://github.com/murali-engine/murali-kit) repository.
Develop against an adjacent engine checkout with that repo's `requirements-local.txt`.

## Core Rust Engine

Use this path when you are working on the runtime, not when you are writing new scenes. For
Rust-authored animations on a frozen API, pin [`murali` 0.2.4](https://crates.io/crates/murali/0.2.4)
and follow the [0.2.4 docs](/docs/0.2.4/intro). Current crate versions keep Rust as the engine only.

You need Rust 1.85 or newer, `cargo`, and a graphics environment for preview.

```toml
[dependencies]
murali = "0.3.0"
anyhow = "1"
glam = "0.33"
```

```bash
git clone https://github.com/murali-engine/murali
cd murali
cargo run --example hello_shapes --release -- --preview
```

The published crate excludes `examples/**`. Reference examples are in the GitHub repository.

Some Rust APIs are feature-gated. The linear-algebra visual toolkit currently needs `experimental`:

```toml
[dependencies]
murali = { version = "0.3.0", features = ["experimental"] }
```

```bash
cargo run --features experimental --example linear_algebra_vectors
```

See [Experimental Features](./beta/experimental-features.md) and
[Your First Scene (Rust)](./rust-first-scene.md).

## Project config

Murali looks for a nearby `murali.toml`. A minimal config:

```toml
[preview]
fps = 60

[export]
fps = 60
width = 1920
```

`width` is the output width in pixels. Height follows the scene's landscape, portrait, or square
[video format](./video-formats.md). The repo includes `murali.toml.example`.

## Related docs

- [Introduction](./intro.mdx)
- [Your First Scene](./first-scene.md)
- [Murali Kit](./murali-kit.md)
- [Python API](./python-bindings.md)
