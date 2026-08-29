---
sidebar_position: 4
---

# Murali Kit

Murali Kit is the first companion package for Murali Engine. It is planned as a Python package that
depends on `murali-engine` and provides higher-level authoring helpers, examples, and educational
components.

The package boundary is:

```text
murali-engine   # core runtime and Python bindings
murali-kit      # free Python helpers and examples built on top
```

Installing `murali-engine` gives you the engine only. Installing `murali-kit` should install a
compatible `murali-engine` automatically through normal Python package dependencies.

## What Belongs Here

Murali Kit is a good home for:

- Python examples that teach real authoring workflows
- reusable title cards, scene templates, layout helpers, and style helpers
- free educational components for math, graphing, AI, and technical explainer videos
- migration experiments while Rust examples are translated into Python
- helpers that make Python authoring pleasant without changing the engine

## What Stays In The Engine

Murali Engine should keep:

- scene graph and tattva identity
- timelines, clips, animations, easing, and seeking
- camera, frames, SceneView, preview, export, and capture
- renderer/backend integration
- core primitives, text, axes, tables, paths, and basic 3D surfaces
- stable extension points used by kit and future packages

## Rust And Python Boundary

For now, Murali Kit should be Python-first. Rust users can keep using the `murali` crate and the
Rust examples in the engine repository.

A Rust kit can exist later if there is a clear Rust-user need for reusable add-on crates. It does
not need to mirror every Python package. The package structure should follow actual user workflows,
not symmetry for its own sake.

