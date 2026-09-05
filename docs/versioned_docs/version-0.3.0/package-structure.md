---
sidebar_position: 3
---

# Engine vs Kit

Murali is a Python-first animation system with a Rust performance core:

```text
murali-kit        Python authoring: themes, colors, teaching views
murali-engine     Python frontend: Scene, primitives, timeline, preview, export
murali            Rust runtime: renderer, engine internals, lower-level embedding
```

Install kit. It depends on the engine.

```bash
python3 -m pip install murali-kit==0.3.0
```

```python
from murali_engine import Scene, Timeline, Circle
from murali_kit.colors import WHITE
from murali_kit.themes import DarkTheme
```

The usual path is to author animations and visuals in Python. The Rust runtime is there to make
rendering fast, not to make most users write Rust.

The engine package does not depend on the kit. User integrations can sit on `murali-engine` without
taking kit opinions.

## Who owns what

**Engine** is the Python language layer:

- scene identity, handles, timeline, camera, frames
- preview and export
- primitives (shapes, text, paths)
- generic renderables (axes, tables, surfaces, 3D props)

**Kit** is sentences and style:

- theme selection (`DarkTheme`, `LightTheme`, `apply_theme`)
- named colors (`WHITE`, `BLUE_D`, …)
- teaching views (networks, attention, graphs, stepwise)
- title cards, openings, layout helpers
- the Python example catalog

Rule of thumb:

```text
If it is the language, keep it in the engine.
If it is a sentence, lesson, or look, put it in the kit.
```

## Themes belong in kit

```python
from murali_engine import Scene
from murali_kit.themes import LightTheme

scene = Scene()
scene.apply_theme(LightTheme())
```

The engine accepts a background as an RGBA tuple. It does not export `DarkTheme`.

## Docs shape

The public guide is one product, **Murali**. A scene file already mixes both packages, so the
sidebar follows authoring: Start, Write scenes, Teaching views, Reference. The package split is
this page, not two top-level products.

Rust architecture and native constructor notes live under **Internals**. Ignore that section while
you are writing scenes.

## Rust crate

The native API is still the `murali` crate:

```toml
[dependencies]
murali = "0.3.0"
```

Use it when you are changing the runtime. Scene authoring in this docs set is Python.

## 0.3.0

The locked public docs are 0.3.0. Unreleased work lives under **Next**. Frozen 0.2.x pages remain
for historical Rust-era authoring, especially 0.2.4. Kit currently depends on
`murali-engine>=0.3.0,<0.4.0`.
