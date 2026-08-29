---
sidebar_position: 3
---

# Package Structure

Murali is the product umbrella. Under that umbrella, packages should stay boring and explicit:

```text
Murali
  Murali Engine
    Rust crate: murali
    Python package: murali-engine

  Murali Kit
    Python package: murali-kit

  Future packages
    Python packages such as murali-premium, murali-ai, or course-specific packs
```

## Murali Engine

Murali Engine is the core runtime and rendering system.

In Rust, the engine is the `murali` crate:

```toml
[dependencies]
murali = "0.2.5"
```

In Python, the same engine is packaged as `murali-engine` and imported as `murali_engine`:

```bash
python3 -m pip install murali-engine==0.2.5
```

```python
from murali_engine import Scene, Timeline, Circle
```

The engine owns scene identity, timelines, preview, export, camera, frames, rendering, core visual
objects, and stable extension points. It should remain useful without any kit or premium package.

## Murali Kit

Murali Kit is a companion Python package on top of Murali Engine. It should contain examples,
ergonomic helpers, educational components, and free add-ons that help people build faster in
Python.

Murali Kit depends on `murali-engine`. Installing the engine does not install the kit.

The current plan is Python-first for Murali Kit. A Rust-based kit is not required for the initial
product structure. If Rust add-on crates become useful later, they should appear because Rust users
need them, not because the Python package hierarchy exists.

## Future Premium Packages

Future paid packages should also depend directly on Murali Engine, not on Murali Kit by default.
That keeps the dependency graph clean:

```text
murali-engine
  <- murali-kit
  <- murali-premium
  <- course-specific or commercial packages
```

Premium packages can reuse Murali Kit only when they intentionally want those helpers. They should
not inherit it accidentally.

## Documentation Shape

The docs should follow the product boundary:

- **Murali**: overview, installation, package structure, roadmap
- **Murali Engine**: Rust engine docs, Python engine bindings, scene/timeline/rendering concepts
- **Murali Kit**: Python add-ons, examples, migration guides, educational helpers
- **Future Packages**: package boundaries, compatibility, licensing, and extension contracts

This keeps the public story simple: learn Murali first, install the engine, then add kit or premium
packages only when they help the project.

