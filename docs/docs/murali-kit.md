---
sidebar_position: 4
---

# Murali Kit

Murali Kit is the Python authoring layer. Install it and you also get a compatible engine.

```bash
python3 -m pip install murali-kit==0.1.1
```

```python
from murali_engine import Scene, Label
from murali_kit.colors import WHITE
from murali_kit.composite import TitleCard
from murali_kit.themes import DarkTheme, apply_theme

scene = apply_theme(Scene(), DarkTheme())
TitleCard("Murali Kit", "Python add-ons for Murali Engine").add_to_scene(scene)
scene.add(Label("Hello", height=0.3, color=WHITE))
scene.preview()
```

## What kit owns

- **Themes** — `DarkTheme`, `LightTheme`, `apply_theme(scene, theme)`
- **Named colors** — `WHITE`, `BLUE_D`, `GOLD_C`, and the rest of the Manim-style palette
- **Teaching views** — diagrams and composed objects for lessons
- **Layout helpers** — `Group`, stacks, and related authoring helpers
- **Examples** — [`murali-kit/examples`](https://github.com/murali-engine/murali-kit/tree/main/examples)

Kit is opinionated on purpose. Themes, palettes, and lesson diagrams will keep moving as the
authoring style settles.

## What stays in the engine

- `Scene`, `Timeline`, handles, camera, frames, `SceneView`
- preview and export
- primitives: shapes, `Label`, `Latex`, `Typst`, `CodeBlock`, `Path`
- generic renderables: `Axes`, `NumberPlane`, `Table`, `ParametricSurface`, `Prop3D`

If you can say it with a circle, a label, and a timeline, it is engine. If it is a lesson diagram
or a look, it is kit.

## Themes

```python
from murali_engine import Scene
from murali_kit.composite import TitleCard
from murali_kit.themes import LightTheme, apply_theme

theme = LightTheme()
scene = apply_theme(Scene(), theme)
TitleCard("Murali Kit", "Light theme", **theme.title_card_kwargs()).add_to_scene(scene)
```

`apply_theme` mutates the scene (background and related styling) and returns it. Themes do not
construct scenes.

## Colors

```python
from murali_kit.colors import BLUE_D, GOLD_C, WHITE
```

Unsuffixed names are the C step of the scale (`BLUE` is `BLUE_C`). The engine will accept any
`(r, g, b, a)` tuple; the names are a kit convenience.

## Teaching views

Import from the category, not from a `collection` package:

```python
from murali_kit.ai import (
    AttentionMatrix,
    ContextWindow,
    KvCacheView,
    NeuralNetworkDiagram,
    NextTokenDistribution,
    TokenSequence,
    TransformerBlockDiagram,
)
from murali_kit.maths import FunctionGraph, NumberLine, VectorField
from murali_kit.storytelling import Stepwise
from murali_kit.composite import TitleCard, Opening
```

Those modules map to `src/ai`, `src/maths`, `src/composite`, and so on in the kit repo.

## Examples

From a kit checkout:

```bash
python3 -m pip install murali-kit
python examples/hello_shapes.py
python examples/motion_basics.py
python examples/title_card.py
```

Local development against an adjacent engine repo uses `requirements-local.txt` in
[murali-kit](https://github.com/murali-engine/murali-kit).

## Dependency range

Murali Kit currently tracks `murali-engine>=0.2.6,<0.3.0`. When the engine makes a breaking 0.3.0
cut, kit will follow with its own version bump.

## Related docs

- [Your First Scene](./first-scene)
- [Package Structure](./package-structure)
- [Python API](./python-bindings)
- [Which API Should I Use?](./which-api-should-i-use)
