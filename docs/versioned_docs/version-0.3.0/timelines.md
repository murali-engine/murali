---
sidebar_position: 6
---

# Timelines

A **timeline** is the scene's one global time axis. It schedules mutations. The scene holds the
visual state.

```python
from murali_engine import Timeline

timeline = Timeline()
timeline.animate(title).at(0.0).for_duration(1.0).typewrite_text().spawn()
timeline.animate(circle).at(0.4).for_duration(1.2).ease("out_cubic").draw().spawn()
timeline.wait_until(3.0)
scene.play(timeline)
```

`.at(...)` values are **absolute scene seconds**. There is one runtime clock per scene.

## Builder

```python
timeline.animate(handle)   # what
    .at(0.0)               # when
    .for_duration(2.0)     # how long
    .ease("out_cubic")     # how it feels
    .move_to((3.0, 0.0, 0.0))
    .spawn()               # commit
```

Without `.spawn()`, the animation is discarded.

## Sections without Clip

Rust `Clip` is not on the Python surface. Keep local arithmetic on one timeline:

```python
intro = 0.0
body = 3.0
timeline.animate(title).at(intro).for_duration(1.0).appear().spawn()
timeline.animate(graph).at(body).for_duration(2.0).draw().spawn()
```

## A second clock

When content must run on its own timeline (a looping inset, a simulation that ignores the parent
narration), put it in a child `Scene` and present it with [SceneView](./scene-views). A SceneView
keeps the child's clock. A group of handles that only need a shared transform should use kit
`Group`, not a SceneView.

## Callbacks

```python
timeline.call_at(2.0, lambda scene: scene.hide(circle))

def follow(scene, t):
    scene.set_position(label, scene.position(circle))

timeline.call_during(0.0, 2.0, follow)
```

`call_at` is a one-shot. `call_during` runs across an interval. Seeking across callbacks is not
always reconstructable; prefer verbs when you can.

## Play, then output

```python
scene.play(timeline)
scene.preview()           # or save_png / export_video
```

Building a `Timeline` does not attach it. `play` installs the schedule. Preview and export consume
the scene.
