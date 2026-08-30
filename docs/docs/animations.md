---
sidebar_position: 4
---

# Animations

Animations are functions of **time**, not frame number. Build them on a `Timeline`, then
`scene.play(timeline)`.

```python
from murali_engine import Timeline

timeline = Timeline()
timeline.animate(circle).at(0.0).for_duration(2.0).ease("in_out_quad").move_to((3.0, 0.0, 0.0)).spawn()
scene.play(timeline)
```

Always `.spawn()`. Without it, nothing is scheduled.

Ease names are strings: `"linear"`, `"in_quad"`, `"out_quad"`, `"in_out_quad"`, `"in_cubic"`,
`"out_cubic"`, `"in_out_cubic"`.

## Which verb

| Want | Use | Not |
| --- | --- | --- |
| Move | `.move_to((x, y, z))` | Custom paths → `call_during` |
| Size | `.scale_to(...)` | |
| Rotate | `.rotate_to(...)` / `.rotate_xyz(...)` | |
| Fade in | `.appear()` after `scene.hide(handle)` | Instant → `scene.show` |
| Fade to a value | `.fade_to(opacity)` | Instant → `scene.hide` |
| Stroke / path | `.draw()` / `.undraw()` | Filled circles (use `.appear()`) |
| Type text | `.typewrite_text()` | Instant text → `.appear()` |
| Title reveal | `.reveal_text()` | |
| Table rows | `.write_table()` | |
| Morph geometry | `.morph_from(source)` | Very different shapes |
| One-shot logic | `timeline.call_at(t, fn)` | Simple property change |
| Every-frame logic | `timeline.call_during(start, duration, fn)` | Simple property change |

## Setup

```python
from murali_engine import Circle, Scene, Timeline
from murali_kit.colors import GREEN_D, WHITE
from murali_kit.themes import DarkTheme, apply_theme

scene = apply_theme(Scene(), DarkTheme())
circle = scene.add(Circle(radius=0.7, color=GREEN_D).with_stroke(0.04, WHITE), at=(-4.0, 0.0, 0.0))

timeline = Timeline()
timeline.animate(circle).at(0.0).for_duration(2.0).ease("out_cubic").move_to((2.0, 1.0, 0.0)).spawn()
scene.play(timeline)
scene.preview()
```

- `.at(seconds)` — start
- `.for_duration(seconds)` — length
- `.ease(...)` — curve
- verb — what changes
- `.spawn()` — commit

Python does not expose Rust `Clip`. Author sections with `.at(...)` on one timeline, or give
independent content a [SceneView](./scene-views).

## Staging a reveal

```python
scene.hide(circle)
timeline.animate(circle).at(1.0).for_duration(0.8).ease("out_cubic").appear().spawn()
```

Appear, typewrite, and draw keep those tattvas hidden on frame 1. A PNG at `duration=0.0` can look
empty. See [Export](./export-and-capture).

## Custom motion

```python
import math

def orbit(scene, t):
    angle = t * 2 * math.pi
    scene.set_position(circle, (math.cos(angle) * 3.0, math.sin(angle) * 3.0, 0.0))

timeline.call_during(0.0, 2.0, orbit)
```

Prefer verbs when they say what you mean. Use callbacks when they do not.

## Camera

```python
timeline.zoom_camera(0.0, 2.0, 8.0, ease="in_out_quad")
timeline.animate_camera_frame(
    0.0, 2.0, position=(2.0, 1.0, 10.0), target=(0.0, 0.0, 0.0), ease="in_out_quad"
)
```

See [Camera](./camera) and [Timelines](./timelines).
