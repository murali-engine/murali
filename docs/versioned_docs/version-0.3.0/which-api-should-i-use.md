---
sidebar_position: 7
---

# Which API Should I Use?

Prefer the highest-level Python API that still says what you mean. Drop down only when that surface
runs out.

## `murali-kit` vs `murali-engine`

### Start with the kit

**When:** you are writing a scene, a lesson, or an explainer.

```bash
python3 -m pip install murali-kit
```

```python
from murali_engine import Scene, Circle, Label
from murali_kit.colors import WHITE
from murali_kit.themes import DarkTheme

scene = Scene()
scene.apply_theme(DarkTheme())
```

**Why:** kit installs the engine, gives you named colors and themes, and owns teaching views.

### Import primitives from the engine

**When:** the object is a language-level building block.

```python
from murali_engine import Circle, Label, Latex, Axes, Table, Path
```

**Why:** those stay on `murali_engine` even when kit collections exist.

### Import teaching views from the kit

**When:** the object is a lesson diagram or composed view.

```python
from murali_kit.composite import TitleCard
from murali_kit.ai import NeuralNetworkDiagram, AttentionMatrix
from murali_kit.maths import FunctionGraph, NumberLine
from murali_kit.storytelling import Stepwise
```

**Why:** kit *is* the collection layer. Do not look for those types on `murali_engine`.

---

## Adding objects

Use `scene.add(tattva, at=(x, y, z))`. It returns a handle. There is no Python `add_tattva`.

```python
circle = scene.add(Circle(radius=0.7, color=GREEN_D).with_stroke(0.04, WHITE), at=(-1.8, 0.4, 0.0))
```

---

## Text: Label vs Typst vs Latex vs CodeBlock

| Need | Use |
| --- | --- |
| Title, caption, UI text | `Label` |
| Math | `Latex` (needs system LaTeX) or `Typst` |
| Rich markup | `Typst` |
| Highlighted code | `CodeBlock` |

```python
Label("Hello", height=0.32, color=WHITE)
Latex(r"\frac{a}{b} + \sqrt{c}", height=0.5)
Typst("*bold* and _italic_", height=0.5)
CodeBlock("print('hello')", language="python", font_size=0.22)
```

Start with `Label`. Reach for `Latex` or `Typst` when the text is actually math or markup.

---

## Motion: appear vs draw vs typewrite vs fade

| Object | Reveal | Hide |
| --- | --- | --- |
| Filled shape | `.appear()` | `.fade_to(0.0)` or hide |
| Path, stroke, arrow | `.draw()` | `.undraw()` |
| Text | `.typewrite_text()` or `.appear()` | `.untypewrite_text()` |
| Instant | `scene.show(handle)` | `scene.hide(handle)` |

Stage a later appear:

```python
scene.hide(circle)
timeline.animate(circle).at(0.0).for_duration(1.0).ease("out_cubic").appear().spawn()
```

Do not use `.draw()` on a filled circle when you mean fade in.

Ease names are strings: `"linear"`, `"in_quad"`, `"out_quad"`, `"in_out_quad"`, `"in_cubic"`,
`"out_cubic"`, `"in_out_cubic"`.

---

## Layout: helpers vs `at=`

```python
scene.to_edge(title, "up", margin=0.8)
scene.next_to(label, circle, "right", 0.5)
scene.align_to(left, right, "center")
scene.add(circle, at=(2.5, 1.3, 0.0))
```

Use helpers for edges and relative placement. Use `at=` when the coordinates are the point.

Kit `Group`, `HStack`, and `VStack` are for several handles that should move or stack together.

---

## Camera

Orthographic is the default and the right choice for diagrams.

```python
scene.set_view_width(16.0)
scene.set_perspective_camera(fov_y_degrees=45.0, near=0.1, far=100.0)
```

Use perspective when the scene is actually 3D and depth should read as depth.

---

## Output: preview vs PNG vs video

```python
scene.preview()
scene.save_png("frame.png", width=1920)
scene.export_video("scene.mp4", width=1920, fps=60)
```

Call one. Each consumes the scene.

A PNG with `duration=0.0` is the first frame. Appear/typewrite/draw tattvas are hidden there.

---

## Independent content: SceneView

Use `SceneView` when a child needs its own scene, timeline, or camera (a looping inset, a
picture-in-picture). Keep one timeline when everything is the same narration.

Python does not expose Rust `Clip`. Schedule sections with `.at(...)` on one `Timeline`, or give
independent content a `SceneView`.

---

## Quick reference

| Task | Use |
| --- | --- |
| Install | `pip install murali-kit` |
| Theme | `scene.apply_theme(DarkTheme())` |
| Named color | `from murali_kit.colors import WHITE` |
| Add object | `scene.add(...)` |
| Place on edge | `scene.to_edge(handle, "up", margin=0.8)` |
| Animate | `timeline.animate(handle).at(...).for_duration(...).verb().spawn()` |
| Instant hide | `scene.hide(handle)` |
| Reveal shape | `.appear()` |
| Reveal path | `.draw()` |
| Reveal text | `.typewrite_text()` or `.appear()` |
| Teaching diagram | kit module, not `murali_engine` |
| Preview | `scene.preview()` |
| PNG | `scene.save_png(...)` |
| Video | `scene.export_video(...)` |

## What's next

- [Tattvas](./tattvas/)
- [Animations](./animations)
- [Teaching views](./murali-kit)
- [Python API](./python-bindings)
