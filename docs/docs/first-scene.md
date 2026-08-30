---
sidebar_position: 4
---

# Your First Scene

Build a small Python scene: a title, a square, a circle, and two moves. By the end you can preview
it in a window or write a PNG.

Rust engine contributors should use [Your First Scene (Rust)](./rust-first-scene.md) instead.

## Prerequisites

```bash
python3 -m pip install murali-kit
```

See [Installation](./installation) if that fails.

## The complete scene

Save this as `my_first_scene.py`:

```python
from murali_engine import Circle, Label, Scene, Square, Timeline
from murali_kit.colors import GREEN_D, RED_B, WHITE
from murali_kit.themes import DarkTheme, apply_theme

scene = apply_theme(Scene(), DarkTheme())

title = scene.add(Label("My First Scene", height=0.38, color=WHITE))
scene.to_edge(title, "up", margin=0.8)

square = scene.add(
    Square(size=1.2, color=RED_B).with_stroke(0.04, WHITE),
    at=(-4.0, 0.0, 0.0),
)
circle = scene.add(
    Circle(radius=0.65, color=GREEN_D, segments=48).with_stroke(0.04, WHITE),
    at=(4.0, 0.0, 0.0),
)

timeline = Timeline()
timeline.animate(square).at(0.0).for_duration(2.0).ease("in_out_quad").move_to((2.0, 0.0, 0.0)).spawn()
timeline.animate(circle).at(0.5).for_duration(2.0).ease("out_quad").move_to((-2.0, 0.0, 0.0)).spawn()
scene.play(timeline)

scene.preview()
```

Run it:

```bash
python3 my_first_scene.py
```

## What each part does

**Theme.** `Scene()` comes from the engine. `apply_theme(..., DarkTheme())` is kit styling. The
engine does not own named themes.

**Tattvas.** `scene.add(...)` places an object and returns a handle. Use that handle to lay it out
and to animate it.

**Position.** `at=(x, y, z)` is world space, not pixels. The origin is the center of the frame.
`to_edge(title, "up", margin=0.8)` is a layout helper; `"up"`, `"down"`, `"left"`, and `"right"`
are the direction names.

**Color.** Named swatches such as `WHITE` and `RED_B` live in `murali_kit.colors`. The engine
accepts any RGBA tuple `(r, g, b, a)` with values from `0.0` to `1.0`.

**Timeline.** `.animate(handle)` starts a builder. `.at(seconds)` is start time, `.for_duration(...)`
is length, `.ease(...)` is the curve, then a verb such as `.move_to(...)`. **`.spawn()` commits
the animation.** Without it, nothing is scheduled.

**Play, then run.** `scene.play(timeline)` installs the schedule. `scene.preview()` opens a window
and consumes the scene.

## Export instead of preview

Replace the last line with one of:

```python
scene.save_png("my_first_scene.png", width=1920)
```

```python
scene.export_video("my_first_scene.mp4", width=1920, fps=60)
```

Do not call `preview()`, `save_png()`, and `export_video()` on the same scene object. Each one
takes ownership of the scene.

A PNG at `duration=0.0` is the opening frame. Appear, typewrite, and draw animations keep those
tattvas hidden on frame one, so a duration-zero PNG can look empty. Export a later time, or use
video, when you want the motion visible.

## Portrait and other frames

```python
scene = apply_theme(Scene(frame="portrait"), DarkTheme())
```

`"landscape"`, `"portrait"`, and `"square"` are the frame names. See [Video Formats](./video-formats).

## Coordinate system

Right-handed world space:

- **X** — left (negative) to right (positive)
- **Y** — down (negative) to up (positive)
- **Z** — into the screen (negative) to toward the camera (positive)

If your numbers are in the hundreds, you are probably thinking in pixels. A circle of radius `1.0`
is a reasonable starting size.

## Kit examples

Broader Python examples live in
[`murali-kit/examples`](https://github.com/murali-engine/murali-kit/tree/main/examples).
`hello_shapes.py` and `motion_basics.py` are the next two to read.

## What's next

1. [Mental Model](./mental-model) — how Scene, tattva, and timeline fit
2. [Murali Kit](./murali-kit) — themes, colors, teaching views
3. [Which API Should I Use?](./which-api-should-i-use)
4. [Common First Mistakes](./common-first-mistakes)
5. [Python API](./python-bindings)

## If nothing appears

- Confirm `scene.play(timeline)` runs before `preview()` / export
- Confirm every animation ends with `.spawn()`
- Stay near the origin; try `at=(0.0, 0.0, 0.0)`
- Use alpha `1.0` so colors are opaque
- Remember that `preview()` cannot be followed by `save_png()` on the same object
