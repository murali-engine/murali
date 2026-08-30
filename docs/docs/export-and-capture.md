---
sidebar_position: 11
---

# Preview and export

Preview opens a window. Export writes files. Both consume the scene — call **one** of them at the
end of the script.

```python
scene.play(timeline)
scene.preview()
```

```python
scene.save_png("frame.png", width=1920)
scene.export_video("scene.mp4", width=1920, fps=60)
```

`export(...)` is the same as `export_video(...)`.

## Preview

Needs a working graphics environment. Does not need `ffmpeg`. In the window you can orbit and pan
for inspection; authored motion still comes from the timeline.

Kit examples use a small runner:

```bash
python examples/hello_shapes.py              # preview
python examples/hello_shapes.py --auto       # close 3s after the timeline ends
python examples/hello_shapes.py --png
python examples/hello_shapes.py --video --width 1920
python preview_all.py --auto
```

## PNG

```python
scene.save_png("out.png", width=1920, fps=1, duration=0.0)
```

`duration=0.0` is the opening frame. Appear, typewrite, and draw hide those tattvas on frame 1, so
that PNG can look empty. Pass a later `duration`, skip those verbs, or export video.

## Video

```python
scene.export_video(
    "out.mp4",
    width=1920,
    fps=60,
    duration=None,          # from the timeline if omitted
    preserve_frames=False,
)
```

MP4 assembly uses `ffmpeg`. If it is missing, Murali still writes frames and tells you where.

`preserve_frames=True` keeps the PNG sequence after the video is built.

## Project config

`murali.toml` next to the project:

```toml
[preview]
fps = 60

[export]
fps = 60
width = 1920
```

`width` is pixels. Height follows the scene [frame](./video-formats.md).

## Transparent and stills

For a still with no MP4, use `save_png`. Square and logo stills in kit examples follow that path.

## Related

- [Video Formats](./video-formats)
- [Your First Scene](./first-scene)
- [Camera](./camera)
