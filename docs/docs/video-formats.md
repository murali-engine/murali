---
sidebar_position: 6
---

# Video Formats

The scene frame is the source of truth for aspect ratio. Export `width` is pixel quality. Height is
derived from the frame.

```text
scene frame + export width = output dimensions
```

| Frame | Aspect | Logical bounds | Typical output |
| --- | --- | --- | --- |
| `"landscape"` | 16:9 | X `-8..8`, Y `-4.5..4.5` | 1920×1080 |
| `"portrait"` | 9:16 | X `-4.5..4.5`, Y `-8..8` | 1080×1920 |
| `"square"` | 1:1 | X `-8..8`, Y `-8..8` | 1200×1200 |

Changing export width does not turn a landscape scene into a portrait scene.

## Choose the frame first

```python
from murali_engine import Scene
from murali_kit.themes import DarkTheme, apply_theme

scene = apply_theme(Scene(), DarkTheme())
scene = apply_theme(Scene(frame="portrait"), DarkTheme())
scene = apply_theme(Scene(frame="square"), DarkTheme())
```

`to_edge` and camera aspect use that frame. Do not call `to_edge` and then change the frame.

Murali does not reflow tattvas across formats. Share components if you want, but compose each
frame on purpose.

## Portrait

```python
from murali_engine import Circle, Label, Scene, Timeline
from murali_kit.colors import BLUE_D, WHITE
from murali_kit.themes import DarkTheme, apply_theme

scene = apply_theme(Scene(frame="portrait"), DarkTheme())

title = scene.add(Label("Attention In 30 Seconds", height=0.55, color=WHITE))
scene.to_edge(title, "up", margin=0.8)

focus = scene.add(
    Circle(radius=1.3, color=BLUE_D, segments=64).with_stroke(0.05, WHITE),
    at=(0.0, 1.5, 0.0),
)

caption = scene.add(Label("Each token decides what matters.", height=0.32, color=WHITE))
scene.to_edge(caption, "down", margin=1.0)

timeline = Timeline()
timeline.animate(title).at(0.0).for_duration(0.8).ease("linear").typewrite_text().spawn()
timeline.animate(focus).at(0.6).for_duration(0.9).ease("out_cubic").appear().spawn()
timeline.animate(caption).at(1.2).for_duration(1.0).ease("linear").typewrite_text().spawn()
scene.play(timeline)
scene.export_video("portrait.mp4", width=1080, fps=60)
```

The kit example is
[`portrait_video.py`](https://github.com/murali-engine/murali-kit/blob/main/examples/portrait_video.py).

## Export width

```toml
[export]
fps = 60
width = 1080
```

Or per call: `scene.export_video("out.mp4", width=1080)`.

```text
Landscape + width 1080 → 1080×608
Portrait  + width 1080 → 1080×1920
Square    + width 1080 → 1080×1080
```

Full HD landscape is `width=1920`. Portrait Shorts are usually `width=1080`.

## Square PNG

```python
scene = apply_theme(Scene(frame="square"), DarkTheme())
scene.set_view_width(10.0)
scene.add(Circle(radius=2.0, color=BLUE_D))
scene.save_png("square_mark.png", width=1200)
```

## Common mistakes

- Setting `[export] width = 1080` does **not** select portrait. Use `Scene(frame="portrait")`.
- `to_edge` then `set_frame("portrait")` is too late. Set the frame in `Scene(...)`.
- There is no automatic responsive layout between landscape and portrait.
