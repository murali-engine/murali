---
sidebar_position: 8
---

# Utility

Utility tattvas live under `murali::frontend::collection::utility`.

## TracedPath

Records and renders the trajectory of a point on a moving tattva. Similar to Manim's `TracedPath`.

`TracedPath` implements `TattvaTrait` directly (not via the `Tattva<T>` wrapper) because it needs to manage its own state across frames.

For normal user code, prefer `scene.add_tattva(...)`. The `scene.add(...)` call shown below is a lower-level exception used here because `TracedPath` is already a concrete tattva implementation rather than a regular state object.

```rust
use murali::frontend::collection::utility::traced_path::TracedPath;

// Trace the center of a moving object
let trace_id = scene.add(TracedPath::new(
    moving_id,
    |pos, _rot| pos,  // trace the object's own center
    Vec4::new(0.4, 0.8, 1.0, 0.8),
    0.03, // thickness
));
```

The second argument is a closure `(position: Vec3, rotation: Quat) -> Vec3` that returns the world-space point to trace. This lets you trace any point on the object, not just its center:

```rust
// Trace a point offset from the object's center
TracedPath::new(id, |pos, rot| {
    pos + rot * Vec3::new(1.0, 0.0, 0.0)
}, color, thickness)
```

### Recording control

```rust
// Stop recording (path stops growing)
if let Some(t) = scene.get_tattva_any_mut(trace_id) {
    t.as_any_mut()
        .downcast_mut::<TracedPath>()
        .unwrap()
        .stop_recording();
}
```

### Configuration

```rust
TracedPath::new(id, point_fn, color, thickness)
    .with_max_points(5000)    // limit memory usage
    .with_min_distance(0.02)  // minimum distance between recorded points
```

Points are recorded by calling `record_point` from an updater each frame:

```rust
scene.add_updater(trace_id, move |scene, tid, _dt| {
    // Get the tracked object's position
    if let Some(source) = scene.get_tattva_any(moving_id) {
        let pos = DrawableProps::read(source.props()).position;
        if let Some(trace) = scene.get_tattva_any_mut(tid) {
            trace.as_any_mut()
                .downcast_mut::<TracedPath>()
                .unwrap()
                .record_point(pos);
        }
    }
});
```

## ScreenshotMarker

A utility tattva that marks a specific frame for export. Useful for capturing a precise moment during a preview session without exporting the full video.

```rust
use murali::frontend::collection::utility::screenshot_marker::ScreenshotMarker;

scene.add_tattva(ScreenshotMarker::at(2.5), Vec3::ZERO); // capture at t=2.5s
```
