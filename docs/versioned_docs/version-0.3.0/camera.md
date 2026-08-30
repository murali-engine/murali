---
sidebar_position: 5
---

# Camera

The scene owns the camera: a position, a target, and a projection. In Python you set it through
scene methods, not by mutating a raw camera struct.

```python
scene.set_camera(position=(0.0, 0.0, 10.0), target=(0.0, 0.0, 0.0))
scene.set_view_width(16.0)
```

## Orthographic (default)

Objects do not shrink with distance. `set_view_width` is how much world is visible horizontally.
Height follows the scene frame.

```python
scene.set_view_width(8.0)   # zoom in
scene.set_view_width(24.0)  # zoom out
```

Moving the camera in orthographic mode does not change the 2D crop. Use `set_view_width`.

Default view width is `16.0`, which matches landscape bounds `[-8, 8] × [-4.5, 4.5]`.

## Perspective

For 3D scenes where depth should read as depth:

```python
scene.set_perspective_camera(fov_y_degrees=45.0, near=0.1, far=100.0)
scene.set_camera(position=(4.0, 3.0, 8.0), target=(0.0, 0.0, 0.0))
```

`up` defaults to `(0.0, 1.0, 0.0)`.

## Animating the camera

```python
timeline.animate_camera_frame(
    start_time=0.0,
    duration=2.0,
    position=(2.0, 1.0, 10.0),
    target=(0.0, 0.0, 0.0),
    ease="in_out_quad",
)
timeline.zoom_camera(start_time=1.0, duration=2.0, zoom=8.0, ease="in_out_quad")
```

`zoom` here is the target orthographic view width.

## Preview orbit

In `scene.preview()`, you can orbit and pan in the window. That is inspection, not authored
camera motion. Authored motion goes on the timeline so export matches preview timing.
