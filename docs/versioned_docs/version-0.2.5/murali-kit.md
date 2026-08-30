---
sidebar_position: 4
---

# Murali Kit

Murali Kit is the first companion package for Murali Engine. It is a Python package that depends on
`murali-engine` and provides higher-level authoring helpers, examples, and educational components.

The package boundary is:

```text
murali-engine   # core runtime and Python bindings
murali-kit      # free Python helpers and examples built on top
```

Install it from PyPI:

```bash
python3 -m pip install murali-kit==0.1.0
```

Installing `murali-engine` gives you the engine only. Installing `murali-kit` installs a compatible
`murali-engine` automatically through normal Python package dependencies.

The first release includes a small `TitleCard` composition and a starter set of Python examples:

```python
from murali_engine import Scene
from murali_kit.composite import TitleCard

scene = Scene()
handles = TitleCard("Murali Kit", "Python add-ons for Murali Engine").add_to_scene(scene)
```

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
- stable extension points used by kit

## Rust And Python Boundary

For now, Murali Kit should be Python-first. Rust users can keep using the `murali` crate and the
Rust examples in the engine repository.

A Rust kit can exist later if there is a clear Rust-user need for reusable add-on crates. It does
not need to mirror every Python package. The package structure should follow actual user workflows,
not symmetry for its own sake.

## Near-Term Plan

The near-term split should be:

- Rust users install `murali` and use the engine docs and Rust examples.
- Python users install `murali-engine` for the core runtime.
- Python users install `murali-kit==0.1.0` when they want higher-level helpers or example
  collections.
- Murali Kit tracks `murali-engine` with a normal dependency range such as
  `murali-engine>=0.2.5,<0.3.0`.

That gives Rust a strong engine-first path and gives Python a friendlier package layer without
requiring both ecosystems to mirror each other.
