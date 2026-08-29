---
sidebar_position: 8
---

# SceneView

A `SceneView` presents one complete Murali `Scene` as a single drawable object inside another
scene. The child keeps its own tattvas, camera, timeline, scene time, and updaters. The parent sees
the result as one object that it can move, scale, rotate, fade, layer, and remove.

Use a `SceneView` when a composition needs more than a group of objects:

- a diagram or simulation that keeps running while the main scene changes around it
- picture-in-picture, split-screen, or inset explanations
- a hand-authored subsystem with its own timeline and camera
- a scene that docks, tilts, shrinks, and later returns to full size
- a framed child scene with its own background, border, and rounded corners

Use a `Group` when several tattvas only need a shared transform. Use a `Clip` when several
animations only need reusable local-time authoring on the same scene clock. Use a `SceneView` when
the content needs an independent scene and playback clock.

## Build A Child Scene

The child is an ordinary `Scene`. It can contain primitives, text, graphs, custom hand-built
diagrams, library composites, updaters, and its own timeline.

```rust
use glam::{Vec3, Vec4};
use murali::frontend::animation::Ease;
use murali::frontend::collection::primitives::circle::Circle;
use murali::{Scene, Timeline};

fn build_child() -> anyhow::Result<Scene> {
    let mut child = Scene::new();
    let node = child.add_tattva(
        Circle::new(0.35, 40, Vec4::new(0.25, 0.75, 1.0, 1.0)),
        Vec3::new(-3.0, 0.0, 0.0),
    );

    let mut timeline = Timeline::new();
    timeline
        .animate(node)
        .at(0.0)
        .for_duration(2.0)
        .ease(Ease::InOutCubic)
        .move_to(Vec3::new(3.0, 0.0, 0.0))
        .spawn();
    child.play(timeline)?;

    Ok(child)
}
```

Nothing inside the child is forced through a specialized collection API. For example, a neural
network can be assembled directly from circles, lines, labels, and custom tattvas when the
provided `NeuralNetworkDiagram` is not the right visual model.

## Add It To A Parent

Configure the view, then add it with `Scene::add_scene_view`. The returned `TattvaId` is the
parent-facing handle.

```rust
use glam::{Vec3, Vec4, vec2};
use murali::{Scene, SceneView, SceneViewPlayback};

let child = build_child()?;
let mut parent = Scene::new();

let view_id = parent.add_scene_view(
    SceneView::new(child)
        .size(vec2(14.0, 7.5))
        .background(Vec4::new(0.02, 0.03, 0.05, 1.0))
        .corner_radius(0.28)
        .border(0.05, Vec4::new(0.25, 0.75, 1.0, 0.9))
        .playback(SceneViewPlayback::Loop { duration: 2.0 }),
    Vec3::ZERO,
);
```

`size` is expressed in parent world units. By default, the child is rendered at a resolution
derived from the parent output and the child's frame aspect ratio. Use `.resolution(width,
height)` when a fixed offscreen texture size is important.

Use `.transparent_background()` for composited content with no view fill. When a background is
set, it belongs to the view and travels with it. Rounded corners clip both the child render and its
background.

## Animate The Whole View

The parent handle uses the normal animation vocabulary. Animating it does not interrupt the child
timeline.

```rust
use glam::{Quat, Vec3};
use murali::frontend::animation::Ease;
use murali::Timeline;

let mut timeline = Timeline::new();

// Dock the still-running child scene in the upper-right.
timeline
    .animate(view_id)
    .at(2.0)
    .for_duration(0.9)
    .ease(Ease::InOutCubic)
    .move_to(Vec3::new(5.0, 2.6, 0.0))
    .spawn();
timeline
    .animate(view_id)
    .at(2.0)
    .for_duration(0.9)
    .ease(Ease::InOutCubic)
    .scale_to(Vec3::new(0.35, 0.35, 1.0))
    .spawn();
timeline
    .animate(view_id)
    .at(2.0)
    .for_duration(0.9)
    .ease(Ease::InOutCubic)
    .rotate_to(Quat::from_rotation_z(-0.04))
    .spawn();

// Bring it back later.
timeline
    .animate(view_id)
    .at(7.0)
    .for_duration(1.0)
    .ease(Ease::InOutCubic)
    .move_to(Vec3::ZERO)
    .spawn();
timeline
    .animate(view_id)
    .at(7.0)
    .for_duration(1.0)
    .ease(Ease::InOutCubic)
    .scale_to(Vec3::ONE)
    .spawn();

parent.play(timeline)?;
```

Parent layout helpers and render layers also work with `view_id`. Removing that ID with
`parent.remove_tattva(view_id)` removes the owned child scene and its rendering resources.

## Local Time

A `SceneView` has a real child clock. It does not inherit the parent's current time directly.
Instead, Murali maps parent time into child-local time:

```text
local time = local offset + max(parent time - start time, 0) * time scale
```

The defaults are `start_at(0.0)`, `local_time_offset(0.0)`, and `time_scale(1.0)`, so both clocks
advance together from zero. This configuration starts the child at parent time `3.0`, begins from
child time `1.0`, and runs it at half speed:

```rust
let view = SceneView::new(child)
    .start_at(3.0)
    .local_time_offset(1.0)
    .time_scale(0.5);
```

The parent can move or resize the view before its local clock starts. `SceneView::local_time()`
reports the current child time.

## Playback Modes

`SceneViewPlayback` controls what the child clock does:

| Mode | Behavior |
| --- | --- |
| `Continuous` | Local time keeps advancing. Timeline values hold at their final state while child updaters continue running. |
| `Once` | Local time stops at the end of the child timeline and the final frame remains visible. |
| `Loop { duration }` | The selected local-time interval repeats. |
| `Paused` | The child remains at its current local time. |

Reaching the end does **not** automatically hide or remove a `SceneView`. Fade, hide, move, or
remove the parent-facing `TattvaId` when that is the desired exit behavior.

Looping rewinds the child through deterministic seeking. The child must therefore be seekable:
non-reversible callbacks, frame-dependent updaters, and history-dependent traced paths cannot be
rewound as a loop. `Continuous` is the appropriate mode for simulations that depend on updaters.

`Once` and `Continuous` child timelines contribute to Murali's inferred export duration. Looped
and paused views do not create an unbounded export; set the parent timeline or explicit export
duration for those compositions.

## Runtime Access

Use the parent scene to inspect or adjust a view:

```rust
if let Some(view) = parent.scene_view(view_id) {
    println!("child time: {}", view.local_time());
}

if let Some(view) = parent.scene_view_mut(view_id) {
    view.set_time_scale(0.75);
    view.set_playback(SceneViewPlayback::Paused);
}
```

`scene()` and `scene_mut()` provide access to the owned child scene. `restart_at(parent_time)`
resets the child to local time zero and chooses a new parent start time.

## Full-Screen Opening Handoff

The beta [`Opening`](./tattvas/opening) composite is a useful full-frame SceneView case. Its 3D
arrival is designed for perspective projection, while the scene that follows may be an
orthographic explainer, graph, or interface. Wrapping the opening scene keeps both cameras active
without switching the parent projection.

### Build The Perspective Child

Build the opening as an ordinary child scene and author its local choreography into a `Clip`:

```rust
use glam::{Vec3, vec3};
use murali::engine::camera::Projection;
use murali::frontend::collection::composite::beta::opening::Opening;
use murali::{Clip, Scene, Timeline};

fn build_opening_scene() -> anyhow::Result<(Scene, f32)> {
    let mut child = Scene::new();
    child.camera_mut().projection = Projection::Perspective {
        fov_y_rad: 43.0_f32.to_radians(),
        aspect: child.frame().aspect_ratio(),
        near: 0.1,
        far: 80.0,
    };
    child.camera_mut().position = vec3(0.0, 2.15, 10.8);
    child.camera_mut().target = vec3(0.0, -0.35, 0.0);

    let ids = Opening::new("MURALI", "PROGRAMMATIC VISUALS")
        .add_to_scene(&mut child, Vec3::ZERO)?;
    let mut clip = Clip::new();
    ids.animate(&mut clip);
    let duration = clip.duration();

    let mut timeline = Timeline::new();
    timeline.append(clip);
    child.play(timeline)?;
    Ok((child, duration))
}
```

The returned duration comes from the authored clip. If title length or `OpeningTiming` changes,
the parent handoff can follow the new duration without duplicating a hard-coded timestamp.

### Add It At Full Frame

The parent can keep its default orthographic camera. Size the view from the parent frame and let
the view own the opening background:

```rust
use glam::{Vec3, Vec4, vec2};
use murali::{SceneView, SceneViewPlayback};

let (opening_scene, opening_duration) = build_opening_scene()?;
let (frame_width, frame_height) = parent.frame().logical_size();

let opening_view = parent.add_scene_view(
    SceneView::new(opening_scene)
        .size(vec2(frame_width, frame_height))
        .background(Vec4::new(0.039, 0.071, 0.11, 1.0))
        .playback(SceneViewPlayback::Once)
        .resolution(1280, 720),
    Vec3::ZERO,
);
```

The child is rendered with its perspective camera into the view texture. The parent content is
rendered with the parent's orthographic camera. No projection switch occurs.

For a full-frame view, use the exact logical frame size and zero corner radius or border. A fixed
resolution makes export cost predictable; choose a resolution appropriate for the final output.

### Reveal The Continuing Parent

Parent content may already exist underneath the opaque opening view. Stage it as hidden, then
start revealing it shortly before or during the view fade:

```rust
let reveal_start = opening_duration - 0.15;

parent_timeline
    .animate(opening_view)
    .at(opening_duration - 0.3)
    .for_duration(0.72)
    .ease(Ease::InOutCubic)
    .fade_to(0.0)
    .spawn();

parent_timeline
    .animate(main_heading)
    .at(reveal_start)
    .for_duration(0.65)
    .typewrite_text()
    .spawn();
```

Fading the parent-facing ID fades the child render and SceneView background together. Matching the
SceneView background to the parent export clear color produces a stable handoff before the parent
content becomes visible.

`SceneViewPlayback::Once` holds the child's final frame at the end of its timeline. It does not
hide or remove the view automatically. After fading it out, leave it hidden or call
`parent.remove_tattva(opening_view)` from appropriate application logic when its resources are no
longer needed.

Run the complete reference:

```bash
cargo run --release --example opening_scene_view
```

## Current Scope

- Multiple sibling `SceneView`s can run at the same time with different playback settings.
- A `SceneView` can contain any normal tattvas and custom hand-built content.
- Recursive SceneView compositing, where a child itself contains another `SceneView`, is not yet
  supported.
- Each view renders to an offscreen texture, so many large or high-resolution views increase GPU
  memory and render cost. Use `.resolution(...)` deliberately when tuning dense compositions.

## Complete Examples

Run the repository example to see a looping hand-built transformer dock while the parent scene
reveals a separate explanation and prediction panel:

```bash
cargo run --example scene_view -- --preview
```

Source: [`examples/scene_view.rs`](https://github.com/murali-engine/murali/blob/main/examples/scene_view.rs)

Run the full-frame beta Opening handoff to see a perspective child fade into an independently
authored orthographic parent:

```bash
cargo run --release --example opening_scene_view
```

Source: [`examples/opening_scene_view.rs`](https://github.com/murali-engine/murali/blob/main/examples/opening_scene_view.rs)
