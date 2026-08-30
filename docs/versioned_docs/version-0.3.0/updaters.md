---
sidebar_position: 10
---

# Updaters

Updaters are callbacks that run every frame. They let you drive tattva properties with arbitrary logic — useful for reactive layouts, physics-like motion, and procedural animation that doesn't fit the timeline model.

## Adding an updater

```rust
scene.add_updater(tattva_id, |scene, id, dt| {
    // dt = delta time in seconds since last frame
    let t = scene.scene_time;
    scene.set_position_2d(id, glam::Vec2::new(t.sin() * 3.0, t.cos() * 2.0));
});
```

The closure receives:
- `scene` — mutable reference to the full scene
- `id` — the `TattvaId` this updater is attached to
- `dt` — frame delta time in seconds

## Circular motion example

```rust
let radius = 3.0;
let speed = 1.0; // radians per second

scene.add_updater(particle_id, move |scene, id, _dt| {
    let t = scene.scene_time * speed;
    scene.set_position_2d(id, glam::Vec2::new(t.cos() * radius, t.sin() * radius));
});
```

## Reactive tattvas

One tattva can read another's state and respond to it. This is how force field visualizations work — each vector arrow reads the particle position and updates its own rotation:

```rust
scene.add_updater(vector_id, move |scene, vid, _dt| {
    if let Some(source) = scene.get_tattva_any(particle_id) {
        let props = murali::frontend::props::DrawableProps::read(source.props());
        let particle_pos = props.position;
        drop(props);

        let delta = vector_pos - particle_pos;
        let angle = delta.y.atan2(delta.x);

        scene.set_rotation(vid, glam::Quat::from_rotation_z(angle));
    }
});
```

## Updaters vs timeline animations

| | Timeline | Updater |
|---|---|---|
| Scheduling | Time-based, declarative | Runs every frame |
| Use case | Defined start/end animations | Reactive, physics, procedural |
| Easing | Built-in | Manual |
| Seeking | Supported or explicitly rejected | Returns `SeekError` |

Use timeline animations for anything with a clear start and end. Use updaters for continuous reactive behavior.

Updaters receive frame `dt`, so Murali cannot reconstruct them at an arbitrary time without replaying every frame with the original step sequence. A scene containing updaters therefore rejects seeks with `SeekError::FrameDependentUpdaters`. Move seekable behavior into a core animation or a reversible timeline callback.
