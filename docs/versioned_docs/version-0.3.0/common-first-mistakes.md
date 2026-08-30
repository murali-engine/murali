---
sidebar_position: 8
---

# Common First Mistakes

A short list of traps that show up in the first hour of Python authoring.

## 1. Forgetting `.spawn()`

```python
# This schedules nothing.
timeline.animate(circle).at(0.0).for_duration(2.0).move_to((3.0, 0.0, 0.0))
```

The builder is inert until `.spawn()` commits it to the timeline.

```python
timeline.animate(circle).at(0.0).for_duration(2.0).move_to((3.0, 0.0, 0.0)).spawn()
```

## 2. Thinking in pixels

```python
Circle(radius=100.0, color=WHITE)          # probably not
scene.add(circle, at=(640.0, 360.0, 0.0))  # probably not
```

Murali is world space. A radius of `1.0` and a position near the origin are the usual starting
point. `scene.set_view_width(16.0)` controls how many world units fit across the frame.

If the numbers are in the hundreds, you are in pixel habits.

## 3. Importing named colors from the engine

```python
from murali_engine import WHITE  # this is not an engine export
```

The engine takes RGBA tuples. Names live in kit:

```python
from murali_kit.colors import WHITE, BLUE_D, GOLD_C
circle = Circle(radius=0.7, color=(0.2, 0.8, 0.3, 1.0))  # also fine
```

## 4. Calling `DarkTheme().scene()`

Themes do not create scenes. The engine owns `Scene`. The kit applies style to it.

```python
from murali_engine import Scene
from murali_kit.themes import DarkTheme, apply_theme

scene = apply_theme(Scene(), DarkTheme())
scene = apply_theme(Scene(frame="portrait"), DarkTheme())
```

## 5. Preview and export on the same object

```python
scene.preview()
scene.save_png("out.png")  # the scene is already consumed
```

`preview()`, `save_png()`, `export_video()`, and `export()` take ownership. Call exactly one.

## 6. A blank PNG at duration 0

Appear, typewrite, and draw keep those tattvas hidden on the opening frame. A
`save_png(..., duration=0.0)` of a scene that only reveals things over time looks empty.

Export a later duration, skip the hide-on-frame-one verbs, or write a video.

## 7. Forgetting `scene.play(timeline)`

Building a `Timeline` does not attach it. `scene.play(timeline)` installs the schedule. Then
preview or export.

## 8. Using kit teaching views from `murali_engine`

`NeuralNetworkDiagram`, `AttentionMatrix`, `TitleCard`, `FunctionGraph`, and friends are kit
imports. A few similarly named engine exports still exist as migration leftovers; prefer the kit
module.

```python
from murali_kit.composite import TitleCard
from murali_kit.ai import NeuralNetworkDiagram
```

## 9. Ease as a Rust enum

Python ease values are strings, not `Ease::OutCubic`.

```python
timeline.animate(circle).at(0.0).for_duration(1.0).ease("out_cubic").appear().spawn()
```

## 10. Example imports vs installed imports

Kit repo examples often say `from colors import WHITE` and `from themes import DarkTheme` because
they run with `PYTHONPATH=src`. In your own project, import the package:

```python
from murali_kit.colors import WHITE
from murali_kit.themes import DarkTheme, apply_theme
```
