---
sidebar_position: 3
---

# Package Structure

The public docs are organized around two top-level products:

```text
Murali Engine
  Python package: murali-engine
  Core Rust Engine
    Rust crate: murali

Murali Kit
  Python package: murali-kit
```

## Murali Engine

Murali Engine is the core runtime and rendering system. The top-level Engine docs should become
Python-first over time, because `murali-engine` is the package most Python authors will install and
use directly.

In Python, the engine is packaged as `murali-engine` and imported as `murali_engine`:

```bash
python3 -m pip install murali-engine==0.2.5
```

```python
from murali_engine import Scene, Timeline, Circle
```

The native Rust API remains available as the `murali` crate:

```toml
[dependencies]
murali = "0.2.5"
```

Rust-specific docs live inside the **Core Rust Engine** section. That section is for the native API,
complete Rust examples, architecture, feature gates, and contributor-facing engine details.

```text
Murali Engine
  Python-first authoring docs
  Core Rust Engine
    Rust API and internals
```

The engine owns the language of Murali: scene identity, timelines, preview, export, camera, frames,
rendering, core visual objects, and stable extension points. It should remain useful without any
kit package.

Engine-owned APIs should be stable, general, and useful across many unrelated animation projects.
Good engine candidates include:

- primitive shapes such as circles, rectangles, lines, arrows, polygons, and paths
- text primitives such as labels, LaTeX, Typst, and code blocks
- structural math primitives such as axes, number planes, curves, surfaces, and tables
- frame, camera, transform, layer, depth, layout, preview, and export behavior
- low-level style primitives such as colors, fonts, strokes, background color, and basic materials
- asset-loading infrastructure and reusable primitive textures

The engine should not own user-facing theme selection in the Python API. It should provide default
colors and explicit style primitives that a theme can apply:

```python
from murali_engine import BLUE, WHITE, Scene

scene = Scene(background=WHITE)
```

Theme selection belongs in Murali Kit, because a theme is an authored design choice that combines
colors, typography, layout defaults, surfaces, animation taste, and composition-level styling:

```python
from murali_kit.themes import LightTheme

scene = LightTheme().scene()
```

Future packages can provide more themes without changing the engine. For example, a future premium
package could provide designer-made themes while depending directly on `murali-engine`.

## Murali Kit

Murali Kit is a companion Python package on top of Murali Engine. It should contain examples,
ergonomic helpers, educational components, and free add-ons that help people build faster in
Python.

Kit-owned APIs should be opinionated, composed, educational, or likely to evolve quickly. Good kit
candidates include:

- title cards, section headers, intro/outro scenes, and callouts
- reusable lesson layouts, comparison layouts, and animation recipes
- AI explainer components, model diagrams, pipeline views, and token-flow helpers
- curated example galleries and demo scenes
- designed visual presets such as explainer, lecture, course, or dashboard styles
- default objects that are assembled from engine primitives rather than atomic primitives
- theme selection and named theme packs, including dark, light, and future designer themes

Install it from PyPI:

```bash
python3 -m pip install murali-kit==0.1.0
```

Murali Kit depends on `murali-engine>=0.2.5,<0.3.0`. Installing the engine does not install the kit;
installing the kit installs a compatible engine package automatically.

The current plan is Python-first for Murali Kit. A Rust-based kit is not required for the initial
product structure. If Rust add-on crates become useful later, they should appear because Rust users
need them, not because the Python package hierarchy exists.

## Documentation Shape

The docs should follow the product boundary:

- **Murali Engine**: Python-first core engine docs, with Core Rust Engine nested inside
- **Murali Kit**: Python add-ons, examples, migration guides, educational helpers

This keeps the public story simple: learn Murali first, install the engine, then add Murali Kit only
when Python helpers and examples are useful.

## Frontend Collection Migration

The current Rust engine contains a broad `frontend::collection` tree. For `0.3.0`, that tree should
not be deleted wholesale. Instead, each collection item should be classified and moved deliberately:

- **Keep in Murali Engine** when it is atomic, structural, broadly useful, or required for core
  examples.
- **Expose through Python in Murali Engine** when Python examples and docs need it as a core
  building block.
- **Move to Murali Kit** when it is a composition, recipe, teaching helper, visual preset, or
  fast-moving experiment.
- **Leave Rust-only temporarily** when the feature is useful internally but not ready for the public
  Python surface.

The practical rule is:

```text
If it is the language, keep it in the engine.
If it is a sentence, template, lesson, or style, put it in the kit.
```

That means `0.3.0` is a migration point, not a mass deletion point. Some collection code may move
out of the engine, some may remain as core primitives, and some may get a Python wrapper before any
package movement happens.
