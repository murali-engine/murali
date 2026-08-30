# Murali

Murali is a Rust-based animation engine for semantic graphics and mathematical scenes. It is built around deterministic timelines, a frontend scene model, CPU-side projection, and a GPU-backed runtime.

## Documentation and cookbook

- Project overview: [Project Overview](https://muraliengine.com)
- Scene and app docs: [Scene and App](https://muraliengine.com/docs/scene-and-app)
- Internal architecture: [Architecture Overview](https://muraliengine.com/docs/architecture/overview)
- AI visualization roadmap: [AI Visualization](https://muraliengine.com/docs/ai-visualization)
- Youtube showcase [Murali Youtube Channel](https://www.youtube.com/@muraliengine)
- Reference examples in this repo: [examples/README.md](./examples/README.md)
- Collection category architecture: [src/frontend/collection](./src/frontend/collection/README.md)


## Goals

- Predictable, explicit animation behavior
- World-space authoring instead of pixel-first APIs
- Clear separation between authored scene state and render/runtime state
- A modern GPU path built on `wgpu`

## Building Blocks And Comfort Tattvas

Murali is primarily a framework of building blocks. The stable core should make primitives, text,
timelines, layouts, camera movement, and rendering expressive enough that users can assemble most
visual elements on the fly.

Murali also intentionally includes a small number of opinionated composite tattvas. These are
comfort tattvas: higher-level components that make common video-making scenes easier to author,
especially for AI explainers, mathematical storytelling, and reusable visual UI.

That convenience has a cost. Too many composites can make the library bloated or too prescriptive,
so new opinionated components usually live in beta first. They may change quickly, move, be renamed,
or be removed while their ergonomics and visual language are tested in real productions. Components
are promoted into stable sections only after they prove mature and broadly useful.

## Current Shape

- `src/frontend/` contains user-facing tattvas, animations, layout helpers, and scene authoring APIs
- `src/projection/` contains backend-neutral render primitives and meshes
- `src/backend/` contains the sync boundary, ECS cache, and renderer
- `src/engine/` contains scene ownership, app lifecycle, timeline stepping, export, and config
- `docs/` contains the longer-form documentation site
- `examples/` contains the reference runnable examples for the crate

## Getting Started

The public authoring path is Python. Install Murali Kit, which pulls in a compatible engine:

```bash
python3 -m pip install murali-kit==0.1.1
```

```python
from murali_engine import Circle, Label, Scene, Timeline
from murali_kit.colors import GREEN_D, WHITE
from murali_kit.themes import DarkTheme, apply_theme

scene = apply_theme(Scene(), DarkTheme())
scene.add(Label("Hello Murali", height=0.38, color=WHITE), at=(0.0, 2.4, 0.0))
scene.add(Circle(radius=1.2, color=GREEN_D).with_stroke(0.04, WHITE))
scene.preview()
```

Prebuilt `murali-engine` wheels cover macOS arm64 and x86_64, Linux x86_64 and aarch64, and Windows
x86_64. Those installs do not need a local Rust toolchain.

```bash
python3 -m pip install murali-engine==0.2.6
```

Python examples live in [`murali-kit`](https://github.com/murali-engine/murali-kit). Docs:
[muraliengine.com](https://muraliengine.com).

### Local engine development

```bash
python3 -m venv .venv
.venv/bin/python -m pip install maturin
.venv/bin/maturin develop --features python
source .venv/bin/activate
python python/examples/hello_shapes.py
```

### Rust crate

Use the `murali` crate when you are working on the runtime:

```toml
[dependencies]
murali = "0.2.6"
anyhow = "1"
glam = "0.33"
```

```bash
git clone https://github.com/murali-engine/murali
cd murali
cargo run --example hello_shapes --release -- --preview
```

The published crate excludes `examples/**`. Reference Rust examples are in this repository. You need
Rust 1.85 or newer, a graphics environment for preview, and `ffmpeg` for video export.

Some in-progress APIs are feature-gated. For example, the linear-algebra visual toolkit currently
requires the `experimental` feature:

```toml
[dependencies]
murali = { version = "0.2.6", features = ["experimental"] }
```

Repository examples that use that API should be run with the feature enabled:

```bash
cargo run --features experimental --example linear_algebra_vectors
```

Quickly inspect a GLB/GLTF asset before using it in a scene:

```bash
cargo run --example model_inspector -- demo-apple
cargo run --example model_inspector -- /absolute/path/to/model.glb --rot-x -20
```

The inspector centers and frames the model automatically. Pass `--help` to see scale, rotation,
camera, and continuous-preview controls.

Some useful places to start:

- [Documentation](https://muraliengine.com/docs/intro)
- [Your first scene](https://muraliengine.com/docs/first-scene)
- [Murali Kit examples](https://github.com/murali-engine/murali-kit/tree/main/examples)
- [Release (crates.io + PyPI wheels)](./RELEASE.md)
- [Future roadmap](./ROADMAP.md)
- [YouTube showcase](https://www.youtube.com/@muraliengine)

## Who It's For

Murali is for people who want authored, programmatic control over mathematical, AI, and explainer-style visuals in Rust.

If you like the kind of mathematical storytelling associated with Manim and want a Rust-native workflow, Murali is built in that spirit.

Murali is also being grown as a long-term AI visualization engine. The
[collection category architecture](./src/frontend/collection/README.md) names the math, probability,
statistics, calculus, optimization, information theory, deep learning, LLM, and agentic-AI
components that will be developed steadily through the end of 2030.

## Preview And Export Config

Murali looks for the nearest `murali.toml` next to a `Cargo.toml`. If no config file is present, sensible defaults are used.

Example config:

```toml
[preview]
fps = 60

[export]
fps = 60
width = 1920
```

The scene owns its aspect ratio. Landscape is the default; portrait and square scenes are explicit:

```rust
let portrait = Scene::new().with_frame(Frame::portrait());
let square = Scene::new().with_frame(Frame::square());
```

Export `width` is literal pixel width. Murali derives height from the scene frame, so a portrait scene at `width = 1080` exports at `1080 × 1920`.

A sample file is included at [murali.toml.example](./murali.toml.example).


## Examples

### Shapes

[![Watch the video](./resources/shapes.png)](https://youtu.be/rzQZHta2PQM)

### Animation showcase

[![Watch the video](./resources/animation_showcase.png)](https://youtu.be/W8WQQbSo70Y)

## Status

Murali is under active development. The repository already includes:

- scene and timeline infrastructure
- preview and headless export paths
- text, LaTeX, and Typst support
- primitives, layout helpers, tables, graph tattvas, and utility tattvas
- write/unwrite, transform, text, and surface animation building blocks
- semantic tensor snapshots, operations, slicing, transitions, and versioned AI trace ingestion
- context-window, next-token sampling, KV-cache, and LayerNorm/RMSNorm teaching views

## License

Murali is dual-licensed under either the MIT License or the Apache License, Version 2.0.
