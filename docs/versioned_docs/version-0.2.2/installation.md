---
sidebar_position: 2
---

# Installation

This page covers the quickest way to get Murali running and the optional tools that improve the experience.

## Prerequisites

You should have:

- Rust 1.85 or newer
- `cargo`
- a working graphics environment for preview mode

If you do not have Rust yet, install it from the official site:

- [Install Rust](https://www.rust-lang.org/tools/install)

Optional but useful:

- `ffmpeg` for MP4 and GIF assembly during export
- `latex` and `dvisvgm` if you want LaTeX text rendering

Typst does not require a separate system install in the default setup.

## Add Murali To A Project

Install Murali from crates.io:

```toml
[dependencies]
murali = "0.2.2"
anyhow = "1"
glam = "0.33"
```

If you specifically want the latest unreleased changes, you can still depend on GitHub instead:

```toml
[dependencies]
murali = { git = "https://github.com/ravishankarkumar/murali" }
anyhow = "1"
glam = "0.33"
```

If you want a quick scratch project:

```bash
cargo new --bin my_scene
cd my_scene
mkdir -p examples
```

Then add the dependency snippet above to `Cargo.toml`.

Important packaging note:

- the published crate excludes `examples/**`
- the reference examples are available in the GitHub repository
- the crates.io package gives you the library surface, not the full example catalog

## Feature-Gated APIs

Some APIs are available only when a Cargo feature is enabled. The current linear-algebra visual
toolkit is experimental and requires the `experimental` feature:

```toml
[dependencies]
murali = { version = "0.2.2", features = ["experimental"] }
anyhow = "1"
glam = "0.33"
```

Repository examples that use `murali::frontend::sangrah::ganit::linear_algebra` should be run
with the same feature:

```bash
cargo run --features experimental --example linear_algebra_vectors
```

See [Experimental Features](./beta/experimental-features.md) for the current feature-gated APIs.

## Preview Vs Export Dependencies

Preview mode:

- needs a working graphics environment
- does not require `ffmpeg`

Export mode:

- can always render PNG frames
- uses `ffmpeg` when assembling video or GIF output

If `ffmpeg` is missing, Murali still exports frames and tells you where they were written.

## Project Config

Murali looks for a nearby `murali.toml` next to a `Cargo.toml`. A minimal config looks like this:

```toml
[preview]
fps = 60

[export]
fps = 60
width = 1920
```

`width` is the literal output width. Murali derives height from the scene's landscape, portrait, or square [video format](./video-formats.md).

The repo includes a sample file at `murali.toml.example` in the repository root.

## First Run

Once your dependency is added, the fastest next step is:

1. read [Your First Scene](./first-scene.md)
2. create `examples/my_scene.rs`
3. run it in preview mode from your own project:

```bash
cargo run --example my_scene --release -- --preview
```

## LaTeX Support

LaTeX text rendering requires system tools:

- `latex`
- `dvisvgm`

If you do not want to install them yet, use `Typst` or `Label` first.

## Related Docs

- [Introduction](./intro.mdx)
- [Your First Scene](./first-scene.md)
- [Text](./tattvas/text.md)
- [Export and Capture](./export-and-capture.md)
