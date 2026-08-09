---
sidebar_position: 5
---

# Camera

Murali's camera is pure state — no input handling, no magic. It owns a position, a target, and a projection mode. The scene owns the camera.

```rust
scene.camera_mut().position = Vec3::new(0.0, 0.0, 10.0);
```

## Projection modes

### Orthographic (default)

The default for 2D math scenes. Objects don't shrink with distance — only `view_width` controls how much of the world is visible.

```rust
use murali::engine::camera::{Camera, Projection};

scene.camera_mut().projection = Projection::Orthographic {
    width: 16.0,   // visible world units horizontally
    height: 9.0,   // visible world units vertically
    near: -100.0,
    far: 100.0,
};
```

The default is `width: 16.0, height: 9.0` — giving a canonical `[-8, 8] × [-4.5, 4.5]` world space. Moving the camera position in orthographic mode does not change what's visible. Use `set_view_width` instead.

### Perspective

For 3D scenes where depth perception matters.

```rust
scene.camera_mut().projection = Projection::Perspective {
    fov_y_rad: std::f32::consts::FRAC_PI_4, // 45°
    aspect: 16.0 / 9.0,
    near: 0.1,
    far: 1000.0,
};
```

## Controlling the viewport (orthographic)

### set_view_width — the ground truth

Controls how much of the world is visible horizontally. Height is derived automatically to maintain 16:9.

```rust
scene.camera_mut().set_view_width(8.0);  // zoom in — less world visible, objects appear larger
scene.camera_mut().set_view_width(24.0); // zoom out — more world visible, objects appear smaller
```

### view_width — the getter

```rust
let w = scene.camera().view_width(); // 16.0 by default
```

Useful for relative adjustments:

```rust
// Zoom in to half the current view
scene.camera_mut().set_view_width(scene.camera().view_width() * 0.5);
```

### zoom_in / zoom_out

Convenience wrappers around `set_view_width`. The factor describes how much larger or smaller objects appear.

```rust
scene.camera_mut().zoom_in(2.0);  // objects appear 2× bigger  (width /= 2)
scene.camera_mut().zoom_out(2.0); // objects appear 2× smaller (width *= 2)
```

## Positioning the camera

```rust
scene.camera_mut().position = Vec3::new(0.0, 0.0, 10.0); // standard 2D setup
scene.camera_mut().target   = Vec3::ZERO;                 // look at origin
scene.camera_mut().up       = Vec3::Y;                    // Y is up
```

For 2D scenes, keep `position.z` positive and `target` at the origin. The Z value doesn't affect orthographic rendering but is needed for the view matrix.

For 3D scenes, orbit freely — the preview window's orbit controller handles mouse drag automatically.

## Animating the camera

Camera animations go through the timeline like any other animation:

```rust
timeline
    .animate_camera()
    .at(0.0)
    .for_duration(2.0)
    .ease(Ease::InOutQuad)
    .move_to(Vec3::new(2.0, 1.0, 10.0))
    .spawn();
```

Available camera animation kinds:

| Method | Effect |
|---|---|
| `.move_to(Vec3)` | Animate camera position |
| `.look_at(Vec3)` | Animate camera target |
| `.frame_to(position, target)` | Animate both position and target together |
| `.zoom_to(width)` | Animate orthographic view width |
| `.fov_to(radians)` | Animate perspective field of view |

Example — zoom in over 2 seconds:

```rust
timeline
    .animate_camera()
    .at(1.0)
    .for_duration(2.0)
    .ease(Ease::InOutCubic)
    .zoom_to(8.0)  // from default 16.0 → 8.0
    .spawn();
```

## Preview controls

When running in preview mode, the camera is controlled by mouse input:

| Action | Effect |
|---|---|
| Left drag | Orbit (3D) |
| Scroll | Zoom |
| Right drag | Pan |
