---
sidebar_position: 7
---

# Core and Kit

Murali has three package surfaces:

```text
murali          Rust crate for the engine
murali-engine   PyPI package: Python binding and frontend for the engine
murali-kit      PyPI package: Python-only collections, themes, colors, examples
```

Author animations and visuals in Python. The Rust crate exists to make the engine fast and to
support direct runtime embedding when someone truly needs that level.

Install `murali-kit` unless you intentionally want the engine package alone.

## What each package does

**`murali`** is the Rust engine crate. It owns the renderer, runtime internals, and lower-level
engine APIs.

**`murali-engine`** is the Python binding and frontend for that engine. It exposes scene, tattva,
timeline, camera, preview, and export APIs for Python authors and integrations.

**`murali-kit`** is Python-only. It owns themes, named colors, teaching views, collections, and
examples built on top of `murali-engine`.

Avoid referring to packages or crates that do not exist.

## Engine responsibilities

The Python engine package should expose the core scene language:

- scene identity, handles, timeline, camera, frames
- preview and export
- primitives such as shapes, text, paths, axes, tables, surfaces, and 3D props
- renderer-backed resources such as fonts, textures, and generated assets
- stable APIs that Python kit collections can build on

The engine should not depend on Murali Kit.

## Kit responsibilities

Murali Kit collects reusable Python components that help authors build lessons and explainers:

- light and dark themes
- named colors
- common layouts and composites
- math, AI, graphing, storytelling, and teaching views
- runnable Python examples

The exact internal module split can evolve inside `murali-kit`. The important rule is that teaching
collections stay on top of `murali-engine` instead of becoming required engine dependencies.

## Python usage

Start with the engine:

```python
from murali_engine import Scene, Timeline, Circle, Label
```

Then add the kit when you want themes, named colors, or teaching views:

```python
from murali_kit.ai import NeuralNetworkDiagram, AttentionMatrix
from murali_kit.maths.linear_algebra import VectorArrow2D, TransformableGrid2D
from murali_kit.themes import DarkTheme
```

## Decision rules

Keep something in `murali-engine` when:

- most Murali scenes need it
- `murali-kit` needs it as infrastructure
- it is required for preview, export, rendering, scene composition, or timelines
- it is small, stable, and domain-neutral

Keep something in `murali-kit` when:

- it is domain-specific
- it is a teaching composition
- it brings heavier optional dependencies
- it is useful but not necessary for the engine to run
- it belongs to a focused lesson workflow or asset pack
