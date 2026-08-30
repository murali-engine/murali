---
sidebar_position: 8
---

# SceneView

A `SceneView` draws one complete `Scene` as a single object inside another scene. The child keeps
its tattvas, camera, timeline, and time. The parent can move, scale, fade, and layer the result.

Use it for a looping inset, picture-in-picture, or a subsystem with its own clock. Use kit `Group`
when several handles only need a shared transform. Use one timeline when everything shares the
same narration.

## Child scene

```python
from murali_engine import Circle, Scene, Timeline
from murali_kit.colors import BLUE_D, WHITE

def build_child() -> Scene:
    child = Scene()
    node = child.add(Circle(radius=0.35, color=BLUE_D).with_stroke(0.03, WHITE), at=(-3.0, 0.0, 0.0))
    timeline = Timeline()
    timeline.animate(node).at(0.0).for_duration(2.0).ease("in_out_cubic").move_to((3.0, 0.0, 0.0)).spawn()
    child.play(timeline)
    return child
```

The child is an ordinary scene. It can use engine primitives and kit teaching views.

## Add it to a parent

`SceneView(child)` **consumes** the child scene. Then add the view to the parent:

```python
from murali_engine import Scene, SceneView
from murali_kit.themes import DarkTheme, apply_theme

parent = apply_theme(Scene(), DarkTheme())
child = build_child()

view = SceneView(child)
view.size(14.0, 7.5)
view.background((0.02, 0.03, 0.05, 1.0))
view.corner_radius(0.28)
view.border(0.05, (0.25, 0.75, 1.0, 0.9))
view.playback("loop", loop_duration=2.0)

view_id = parent.add_scene_view(view, at=(0.0, 0.0, 0.0))
```

`view_id` is a parent handle. Animate it like any other tattva.

Playback names include `"once"` and `"loop"`. Loop needs `loop_duration`.

## Kit example

[`scene_view.py`](https://github.com/murali-engine/murali-kit/blob/main/examples/scene_view.py)
runs a live child inside a parent explanation.

## Full-screen opening handoff

Put a perspective `Opening` in a full-frame `SceneView` when the ident must keep its own camera
and then reveal orthographic content in the parent. The parent treats the opening as one object:
fade it, scale it, or dock it without switching the parent projection.

The kit example is
[`opening_scene_view.py`](https://github.com/murali-engine/murali-kit/blob/main/examples/opening_scene_view.py).
Rust notes remain under Internals → Opening.
