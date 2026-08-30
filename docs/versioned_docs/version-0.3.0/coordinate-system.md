---
sidebar_position: 5
---

# Coordinate system

Murali uses a right-handed **world space** with the origin at the center of the frame. Positions
are not pixels.

## Axes

- **X** — left (negative) to right (positive)
- **Y** — down (negative) to up (positive)
- **Z** — into the screen (negative) to toward the camera (positive)

```python
from murali_engine import Circle, Scene
from murali_kit.colors import GREEN_D, WHITE
from murali_kit.themes import DarkTheme, apply_theme

scene = apply_theme(Scene(), DarkTheme())
scene.add(Circle(radius=0.8, color=GREEN_D).with_stroke(0.04, WHITE), at=(0.0, 0.0, 0.0))
scene.add(Circle(radius=0.4, color=GREEN_D), at=(-4.0, 2.5, 0.0))
```

If your numbers are in the hundreds, you are thinking in pixels. A circle of radius `1.0` is a
reasonable starting size.

## Frames

Every scene owns a composition frame. Landscape is the default:

```python
Scene()
Scene(frame="portrait")
Scene(frame="square")
```

| Frame | Aspect | X bounds | Y bounds |
| --- | --- | --- | --- |
| `"landscape"` | 16:9 | `-8` to `8` | `-4.5` to `4.5` |
| `"portrait"` | 9:16 | `-4.5` to `4.5` | `-8` to `8` |
| `"square"` | 1:1 | `-8` to `8` | `-8` to `8` |

Selecting portrait does not reflow existing objects. Compose inside that frame. Layout helpers such
as `to_edge` use the frame immediately.

See [Video Formats](./video-formats.md).

## Layout

```python
title = scene.add(Label("Title", height=0.38, color=WHITE))
scene.to_edge(title, "up", margin=0.8)
scene.next_to(caption, title, "down", 0.4)
scene.align_to(left, right, "center")
```

Directions are `"up"`, `"down"`, `"left"`, `"right"`. Kit `Group`, `HStack`, and `VStack` stack
handles when several objects should move together.

## Camera

Orthographic by default. In 2D, `set_view_width` controls how much world fits across the frame.
Camera Z does not change the ortho crop.

```python
scene.set_view_width(16.0)
scene.set_camera(position=(0.0, 0.0, 10.0), target=(0.0, 0.0, 0.0))
```

See [Camera](./camera.md).

## Colors

The engine takes RGBA tuples `(r, g, b, a)` in `0.0…1.0`. Named swatches live in kit:

```python
from murali_kit.colors import WHITE, BLUE_D, GOLD_C

WHITE          # (1, 1, 1, 1)
(1.0, 0.0, 0.0, 0.5)  # also fine
```

The color table is on [Visual Foundations](./visual-foundations).
