---
sidebar_position: 4
---

# Your First Scene (Rust)

Python authors should start with [Your First Scene](./first-scene.md). This page is for people
working on the Rust runtime or embedding Murali below the Python frontend layer.

The current first-party scene-authoring path is Python. If you want the last frozen Rust authoring
API, use [`murali` 0.2.4](https://crates.io/crates/murali/0.2.4) and the
[0.2.4 docs](/docs/0.2.4/intro).

## What you will build

A minimal engine scene with:

- a title label
- a red square and a green circle
- a timeline that moves both shapes
- the runtime app that previews or exports the scene

## Setup

From this repository:

```bash
cargo run --example hello_shapes --release -- --preview
```

For a separate Rust project:

```toml
[dependencies]
murali = "0.3.0"
anyhow = "1"
glam = "0.33"
```

You need Rust 1.85 or newer, a graphics environment for preview, and `ffmpeg` for video export.
The published crate excludes `examples/**`; clone the repository if you want the full example
catalog.

## Complete example

Create `examples/my_first_scene.rs` in a checkout of this repository, or use the same code in a
binary crate that depends on `murali`:

```rust
use glam::Vec3;
use murali::App;
use murali::colors::*;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::animation::Ease;
use murali::frontend::collection::primitives::{circle::Circle, square::Square};
use murali::frontend::collection::text::label::Label;
use murali::frontend::layout::Direction;
use murali::positions::CAMERA_DEFAULT_POS;

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    let title_id = scene.add_tattva(
        Label::new("My First Scene", 0.38).with_color(WHITE),
        Vec3::ZERO,
    );
    scene.to_edge(title_id, Direction::Up, 0.8);

    let square_id = scene.add_tattva(
        Square::new(1.2, RED_B).with_stroke(0.04, WHITE),
        Vec3::new(-4.0, 0.0, 0.0),
    );
    let circle_id = scene.add_tattva(
        Circle::new(0.65, 48, GREEN_D).with_stroke(0.04, WHITE),
        Vec3::new(4.0, 0.0, 0.0),
    );

    let mut timeline = Timeline::new();
    timeline
        .animate(square_id)
        .at(0.0)
        .for_duration(2.0)
        .ease(Ease::InOutQuad)
        .move_to(Vec3::new(2.0, 0.0, 0.0))
        .spawn();
    timeline
        .animate(circle_id)
        .at(0.5)
        .for_duration(2.0)
        .ease(Ease::OutQuad)
        .move_to(Vec3::new(-2.0, 0.0, 0.0))
        .spawn();

    scene.play(timeline)?;
    scene.camera_mut().position = CAMERA_DEFAULT_POS;

    App::new()?.with_scene(scene).run_app()
}
```

Run it from this repository:

```bash
cargo run --example my_first_scene --release -- --preview
```

Without `--preview`, Murali uses the configured export path. You can also pass `--export`
explicitly.

## What matters

**Scene.** `Scene::new()` is the source of truth for objects, timeline state, camera, and frame.
Python hides the runtime app; Rust code constructs it directly with `App::new()?.with_scene(scene)`.

**Tattvas.** `add_tattva(state, position)` wraps a visual state object and returns an ID. Save that
ID when you need to animate or lay out the object.

**Layout.** Use `Vec3` positions for exact placement. Helpers such as
`scene.to_edge(title_id, Direction::Up, 0.8)` operate on IDs after the object is in the scene.

**Timeline.** `.animate(id)` starts a builder. `.at(...)` sets the start time, `.for_duration(...)`
sets the length, `.ease(...)` sets interpolation, and a verb such as `.move_to(...)` declares the
change. `.spawn()` commits the animation to the timeline.

**Camera.** The default camera position is `CAMERA_DEFAULT_POS`, looking toward the origin. If
nothing appears, first check that objects are near the origin and inside the camera view.

## Preview controls

- **O** — orbit camera mode
- **P** — pan/zoom camera mode
- **Drag** — move the camera
- **Scroll** — zoom
- **Esc** — exit

## What's next

- [Scene and App](./scene-and-app)
- [Animations](./animations)
- [Tattva details](./tattvas/properties)
- [Architecture Overview](./architecture/overview)
