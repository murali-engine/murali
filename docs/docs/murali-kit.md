---
sidebar_position: 4
---

# Teaching views

Install **Murali Kit** and you get themes, named colors, and lesson diagrams on top of the engine.

```bash
python3 -m pip install murali-kit==0.3.0
```

```python
from murali_engine import Scene, Label
from murali_kit.colors import WHITE
from murali_kit.composite import TitleCard
from murali_kit.themes import DarkTheme

scene = Scene()
scene.apply_theme(DarkTheme())
TitleCard("Murali Kit", "Teaching views on Murali Engine").add_to_scene(scene)
scene.add(Label("Hello", height=0.3, color=WHITE))
scene.preview()
```

This is the usual authoring path. You still import primitives from `murali_engine`. Kit does not
replace `Scene` or `Timeline`.

## Themes and colors

```python
from murali_kit.themes import DarkTheme, LightTheme
from murali_kit.colors import WHITE, BLUE_D, GOLD_C

theme = LightTheme()
scene = Scene()
scene.apply_theme(theme)
```

`scene.apply_theme(...)` styles an existing scene and returns it. Themes do not construct scenes.
The palette table is on [Space and color](./visual-foundations).

## Composite

```python
from murali_kit.composite import TitleCard, Opening, MuraliLogo, ChatInputBox
```

Title cards, openings, and the logo are authored compositions. `Opening` can also live inside a
[SceneView](./scene-views) when the ident needs its own camera.

## AI

```python
from murali_kit.ai import (
    AttentionMatrix,
    ContextWindow,
    KvCacheView,
    NeuralNetworkDiagram,
    NextTokenDistribution,
    TokenSequence,
    TransformerBlockDiagram,
    TensorView,
)
```

These are lesson diagrams, not a model runtime. The category roadmap is
[AI Visualization](./ai-visualization).

## Maths

```python
from murali_kit.maths import FunctionGraph, NumberLine, VectorField, StreamLines
```

Linear-algebra teaching views also live under `murali_kit.maths` (`LabeledVector2D`,
`MatrixVectorFlow`, and related). Engine `Axes` / `NumberPlane` stay on `murali_engine` when you
need the generic renderable.

## Story and layout

```python
from murali_kit.storytelling import Stepwise
from murali_kit.layout import Group
```

`Stepwise` is the guided-reveal helper. `Group` / stacks share transforms without a second clock.

## Examples

```bash
uv run python examples/hello_shapes.py
uv run python examples/motion_basics.py
uv run python examples/title_card.py
uv run python examples/neural_networks.py
```

Catalog: [`murali-kit/examples`](https://github.com/murali-engine/murali-kit/tree/main/examples).

Kit currently depends on `murali-engine>=0.3.0,<0.4.0`.
