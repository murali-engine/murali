# Murali Rust engine

Murali is a Rust-based animation engine. The `murali` crate contains the scene model, timeline,
projection layer, and GPU renderer, and is the primary surface for embedding or extending the
engine.

Use the current `murali` crate when embedding Murali in a Rust program, extending the renderer, or
building an authoring or integration layer. Experimental Python bindings are also available for
early adopters ([PYTHON.md](./PYTHON.md)).

If you want the last first-party Rust scene-authoring API, pin
[`murali` 0.2.4](https://crates.io/crates/murali/0.2.4).

## Add the crate

```toml
[dependencies]
murali = "0.3.0"
anyhow = "1"
glam = "0.33"
```

Needs Rust 1.85+, a graphics environment for preview, and `ffmpeg` for video export.

Experimental linear-algebra visuals:

```toml
[dependencies]
murali = { version = "0.3.0", features = ["experimental"] }
```

```bash
cargo run --features experimental --example linear_algebra_vectors --release -- --preview
```

## What you get

- `src/engine/` — scene, timeline, camera, preview, export
- `src/frontend/` — tattvas, animations, layout
- `src/projection/` — meshes independent of the GPU backend
- `src/backend/` — wgpu renderer

The published crate **excludes** `examples/**`. Clone this repository to run engine-dev examples.

```bash
git clone https://github.com/murali-engine/murali
cd murali
cargo run --example hello_shapes --release -- --preview
```

Inspect a GLB/GLTF:

```bash
cargo run --example model_inspector -- demo-apple
cargo run --example model_inspector -- /absolute/path/to/model.glb --rot-x -20
```

Catalog: [examples/README.md](./examples/README.md). Architecture:
[Rust `0.2.4` architecture reference](./docs/versioned_docs/version-0.2.4/architecture/overview.md).

## Config

Murali walks upward from the working directory and uses the nearest `murali.toml`.

```toml
[preview]
fps = 60

[export]
fps = 60
width = 1920
```

```rust
let portrait = Scene::new().with_frame(Frame::portrait());
let square = Scene::new().with_frame(Frame::square());
```

Export `width` is pixels. Height follows the scene frame. Sample:
[murali.toml.example](./murali.toml.example).

## Experimental Python bindings from this tree

The same crate builds `murali-engine` when the `python` feature is on. See
[PYTHON.md](./PYTHON.md) for `maturin develop`.
