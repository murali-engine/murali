---
sidebar_position: 3
---

# Tattvas

A **tattva** is any object you can place, animate, and render. From Sanskrit: “element” or
“essence.”

Add it with `scene.add(...)`. Keep the handle.

```python
from murali_engine import Circle, Label, Scene
from murali_kit.colors import GREEN_D, WHITE
from murali_kit.themes import DarkTheme, apply_theme

scene = apply_theme(Scene(), DarkTheme())
title = scene.add(Label("Hello", height=0.38, color=WHITE), at=(0.0, 2.4, 0.0))
circle = scene.add(
    Circle(radius=0.8, color=GREEN_D, segments=48).with_stroke(0.04, WHITE),
    at=(0.0, 0.0, 0.0),
)
```

Engine primitives are the language. Kit teaching views are sentences built from that language.
Import each from the package that owns it.

## Engine primitives

| Need | Type |
| --- | --- |
| Shapes | `Circle`, `Square`, `Rectangle`, `RoundedRectangle`, `Polygon`, `Line`, `Arrow`, `Path` |
| Text | `Label`, `Latex`, `Typst`, `CodeBlock` |
| Structure | `Axes`, `NumberPlane`, `Table` |
| 3D | `Axes3D`, `ParametricSurface`, `ParametricCurve3D`, `Prop3D`, `Letter3D` |
| Motion traces | `TracedPath`, `ParticleBelt` |
| Nested scene | `SceneView` |

```python
from murali_engine import Circle, Label, Latex, Axes, Table, Path, Prop3D
```

`with_stroke`, `with_color`, and other `with_*` methods return the same object.

## Kit teaching views

| Need | Import |
| --- | --- |
| Title / opening | `murali_kit.composite` (`TitleCard`, `Opening`, `MuraliLogo`) |
| AI diagrams | `murali_kit.ai` (`NeuralNetworkDiagram`, `AttentionMatrix`, `TokenSequence`, `KvCacheView`, …) |
| Maths | `murali_kit.maths` (`FunctionGraph`, `NumberLine`, `VectorField`, linear-algebra views) |
| Story | `murali_kit.storytelling` (`Stepwise`) |
| Layout | `murali_kit.layout` (`Group`, stacks) |
| Theme / palette | `murali_kit.themes`, `murali_kit.colors` |

See [Teaching views](../murali-kit) for the catalog.

## Shared scene helpers

```python
scene.to_edge(handle, "up", margin=0.8)
scene.hide(handle)
scene.show(handle)
scene.set_position(handle, (1.0, 0.0, 0.0))
scene.set_scale(handle, (2.0, 2.0, 2.0))
scene.set_layer(handle, 1)
```

Constructor field lists for the native Rust types still live under
[Internals → Tattva details](./properties). Prefer this page and the [Python API](../python-bindings)
while authoring.

## 3D props

Load a GLB/GLTF with `Prop3D.from_file(...)` / `from_glb` / `from_gltf`. Asset notes:
[3D prop assets](../3d-prop-assets).
