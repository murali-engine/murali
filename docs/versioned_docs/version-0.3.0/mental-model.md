---
sidebar_position: 5
---

# Mental Model

Murali scenes are ordinary Python. The engine keeps one source of truth, then draws it.

```text
Scene → tattvas → Timeline → play → preview or export
```

1. Create a **Scene**
2. Add **tattvas** (visual objects)
3. Build a **Timeline** (what changes, when)
4. **Play** that timeline on the scene
5. **Preview** or **export**

There is no separate `App` object in Python. `scene.preview()` and `scene.export_video(...)` are
the runtime.

## Scene

The scene owns tattvas, the timeline, the camera, the logical frame, and current time.

```python
from murali_engine import Scene
from murali_kit.themes import DarkTheme

scene = Scene()
scene.apply_theme(DarkTheme())
scene = Scene(frame="portrait")
scene.apply_theme(DarkTheme())
```

Think of it as a stage: it holds the actors, knows what time it is, and follows the script.

The frame is composition (landscape, portrait, square). Export width is pixel quality, not
composition. See [Video Formats](./video-formats).

## Tattva

A **tattva** is any visual object. The word is from Sanskrit: “element” or “essence.”

Engine primitives include `Circle`, `Square`, `Label`, `Latex`, `Typst`, `CodeBlock`, `Path`,
`Axes`, `Table`, `ParametricSurface`. Kit adds teaching views such as `TitleCard`,
`NeuralNetworkDiagram`, and `FunctionGraph`.

`scene.add(...)` returns a handle. Animate and lay out through that handle. You do not pass raw
numeric IDs.

```python
from murali_engine import Circle, Scene
from murali_kit.colors import GREEN_D, WHITE

scene = Scene()
circle = scene.add(
    Circle(radius=1.0, color=GREEN_D).with_stroke(0.04, WHITE),
    at=(0.0, 0.0, 0.0),
)
```

Every tattva has a position in world space plus shared properties: scale, rotation, opacity,
visibility.

## Timeline

A timeline schedules changes against scene time, not frame numbers.

```python
from murali_engine import Timeline

timeline = Timeline()
timeline.animate(circle).at(0.0).for_duration(2.0).ease("in_out_quad").move_to((3.0, 0.0, 0.0)).spawn()
scene.play(timeline)
```

- `.at(0.0)` — start time in seconds
- `.for_duration(2.0)` — length
- `.ease("in_out_quad")` — curve (`linear`, `in_quad`, `out_quad`, `in_out_quad`, `in_cubic`,
  `out_cubic`, `in_out_cubic`)
- `.move_to(...)` — the verb
- `.spawn()` — commit; without this the animation is discarded

Think of it as a score: each instrument (tattva) gets an entrance, a duration, and a direction.

## Animation verbs

Common verbs:

- `.move_to((x, y, z))`
- `.scale_to(...)`
- `.rotate_to(...)` / `.rotate_xyz(...)`
- `.fade_to(opacity)`
- `.appear()` / `.draw()` / `.undraw()`
- `.typewrite_text()` / `.reveal_text()`

Filled shapes usually `.appear()`. Paths, strokes, and outlines usually `.draw()`. Text often
`.typewrite_text()` or `.appear()`.

Hide a tattva first when you want a later appear:

```python
scene.hide(circle)
timeline.animate(circle).at(1.0).for_duration(0.8).ease("out_cubic").appear().spawn()
```

## Preview and export

Python does not construct `App`. After `scene.play(timeline)`:

```python
scene.preview()
# or
scene.save_png("frame.png", width=1920)
# or
scene.export_video("scene.mp4", width=1920, fps=60)
```

Each of those consumes the scene. That is a current runtime constraint: the renderer takes
ownership. Build the scene, play the timeline, then call exactly one output method.

## Authored state vs pixels

**Authored state** is what you wrote: “a circle of radius 1.0 at (2, 3, 0).”

**Rendered output** is GPU meshes and buffers.

You author the first. The engine syncs the second when something actually changed. This is why
Murali can keep scenes semantic instead of making you think in vertices.

## World space

```python
circle = Circle(radius=1.0, color=GREEN_D)  # 1.0 world units, not 1 pixel
scene.add(circle, at=(2.0, 3.0, 0.0))
scene.set_view_width(16.0)  # 16 world units across the frame
```

World space stays the same at 720p and 4K. The camera maps it to pixels.

## One clock

A scene plays one timeline. Independent running content (an inset, a looping diagram) is a child
scene presented with `SceneView`. That child has its own timeline. Use `SceneView` when you need a
second clock; keep one timeline when everything shares the same narration.

Python does not currently expose Rust `Clip` composition. Author sections with `.at(...)` on one
timeline, or put independent content in a `SceneView`.

## Engine vs kit

| | Engine | Kit |
| --- | --- | --- |
| Package | `murali-engine` | `murali-kit` |
| Owns | Scene, primitives, timeline, camera, export | Themes, named colors, teaching views |
| Example | `Circle`, `Label`, `Axes` | `DarkTheme`, `WHITE`, `TitleCard`, `AttentionMatrix` |

If it is the language, it is engine. If it is a sentence, lesson, or style, it is kit.

## Key takeaways

1. The scene is the source of truth
2. Tattvas are semantic objects, not meshes you draw by hand
3. The timeline is time, not frames
4. `.spawn()` commits an animation
5. Preview and export consume the scene
6. Coordinates are world units
7. Named colors and themes come from kit

## What's next

- [Space and color](./visual-foundations)
- [Tattvas](./tattvas/)
- [Animations](./animations)
- [Teaching views](./murali-kit)
- [Python API](./python-bindings)
