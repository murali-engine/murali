---
sidebar_position: 6
---

# Video Formats

Murali supports three intentional composition frames:

| Frame | Aspect | Logical bounds | Typical output |
| --- | --- | --- | --- |
| `Frame::landscape()` | 16:9 | X `-8..8`, Y `-4.5..4.5` | 1920x1080 |
| `Frame::portrait()` | 9:16 | X `-4.5..4.5`, Y `-8..8` | 1080x1920 |
| `Frame::square()` | 1:1 | X `-8..8`, Y `-8..8` | 1200x1200 |

The scene frame is the source of truth for aspect ratio and composition. Export `width` is the literal output width in pixels. Murali derives the output height from the scene frame.

```text
scene frame + export width = output dimensions
```

Changing export width changes image quality. It does not convert a landscape composition into portrait or rearrange any objects.

## Choose The Frame First

Select the frame when creating the scene, before calling frame-relative layout helpers:

```rust
use murali::{Frame, Scene};

let mut landscape = Scene::new();
let mut portrait = Scene::new().with_frame(Frame::portrait());
let mut square = Scene::new().with_frame(Frame::square());
```

`Scene::new()` is landscape by default. Once a frame is selected, helpers such as `to_edge`, `frame_bounds`, and camera zoom use that aspect ratio.

Murali does not automatically reflow content between formats. Compose each scene deliberately inside its frame.

## Portrait And Shorts Example

This complete scene is authored directly inside the `9:16` coordinate space:

```rust
use glam::Vec3;
use murali::colors::{BLUE_D, WHITE};
use murali::engine::timeline::Timeline;
use murali::frontend::animation::Ease;
use murali::frontend::sangrah::primitives::circle::Circle;
use murali::frontend::sangrah::text::label::Label;
use murali::frontend::layout::Direction;
use murali::{App, Frame, RenderOptions, Scene};

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new().with_frame(Frame::portrait());

    let title = scene.add_tattva(
        Label::new("Attention In 30 Seconds", 0.55).with_color(WHITE),
        Vec3::ZERO,
    );
    scene.to_edge(title, Direction::Up, 0.8);

    let focus = scene.add_tattva(
        Circle::new(1.3, 64, BLUE_D).with_stroke(0.05, WHITE),
        Vec3::new(0.0, 1.5, 0.0),
    );

    let caption = scene.add_tattva(
        Label::new("Each token decides what matters.", 0.32).with_color(WHITE),
        Vec3::ZERO,
    );
    scene.to_edge(caption, Direction::Down, 1.0);

    let mut timeline = Timeline::new();
    timeline
        .animate(title)
        .at(0.0)
        .for_duration(0.8)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    timeline
        .animate(focus)
        .at(0.6)
        .for_duration(0.9)
        .ease(Ease::OutCubic)
        .appear()
        .spawn();
    timeline
        .animate(caption)
        .at(1.2)
        .for_duration(1.0)
        .ease(Ease::Linear)
        .typewrite_text()
        .spawn();
    scene.play(timeline)?;

    App::new()?
        .with_scene(scene)
        .with_render_options(RenderOptions {
            width: Some(1080),
            ..RenderOptions::default()
        })
        .run_app()
}
```

Preview it with:

```bash
cargo run --release --example portrait_video -- --preview
```

Export it with:

```bash
cargo run --release --example portrait_video
```

The repository contains the complete runnable source in [`examples/portrait_video.rs`](https://github.com/ravishankarkumar/murali/blob/main/examples/portrait_video.rs).

## Configure Export Width

For projects where all scenes use the same output width, configure it once:

```toml
[export]
fps = 60
width = 1080
```

The same width produces different heights because the scene owns the aspect ratio:

```text
Landscape + width 1080 -> 1080x608
Portrait  + width 1080 -> 1080x1920
Square    + width 1080 -> 1080x1080
```

For standard Full HD output, use `width = 1920` for landscape and `width = 1080` for portrait. Use `RenderOptions::width` when one example needs to override the project setting.

## Square Output

Square scenes use the same workflow. This complete example exports one `1200x1200` PNG:

```rust
use glam::{Vec3, Vec4};
use murali::colors::BLUE_D;
use murali::engine::export::{ExportSettings, export_scene};
use murali::frontend::sangrah::primitives::circle::Circle;
use murali::{Frame, Scene};

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new().with_frame(Frame::square());
    scene.camera_mut().set_view_width(10.0);

    scene.add_tattva(Circle::new(2.0, 64, BLUE_D), Vec3::ZERO);
    scene.capture_screenshots_named([(0.0, Some("square_mark.png"))]);

    let settings = ExportSettings {
        width: 1200,
        video_enabled: false,
        clear_color: Vec4::new(0.0, 0.0, 0.0, 0.0),
        artifact_dir: "square_mark".into(),
        ..ExportSettings::default()
    };

    export_scene(scene, &settings)
}
```

The image is written under `rendered_output/square_mark`. The transparent logo example uses the same square frame and capture path:

```bash
cargo run --release --example murali_logo_transparent
```

## Layout And Camera Behavior

Frame-relative layout is available during scene construction:

```rust
let mut scene = Scene::new().with_frame(Frame::portrait());

let title = scene.add_tattva(title_label, Vec3::ZERO);
scene.to_edge(title, Direction::Up, 0.6);
```

In portrait mode, `Direction::Up` targets Y `8`; in landscape mode it targets Y `4.5`. The margin and tattva bounds are then applied normally.

`set_view_width` changes camera zoom while retaining the selected aspect:

```rust
scene.camera_mut().set_view_width(4.5);
```

For a portrait scene this produces a `4.5x8` visible camera area. Perspective cameras also use the scene frame's aspect ratio during rendering.

## Common Mistakes

### Changing only export width

```toml
[export]
width = 1080
```

This does not select portrait mode. Use `Frame::portrait()` in the scene.

### Selecting the frame after layout

```rust
scene.to_edge(title, Direction::Up, 0.5);
scene.set_frame(Frame::portrait()); // Too late for the earlier placement.
```

Select the frame when constructing the scene:

```rust
let mut scene = Scene::new().with_frame(Frame::portrait());
```

### Expecting automatic responsive composition

Murali intentionally does not move, resize, or reflow tattvas when the frame changes. Landscape and portrait versions can share components and animation logic, but their composition should be authored explicitly.

## Migration From 0.1

Export height is no longer configured directly.

Before:

```toml
[export]
width = 1920
height = 1080
```

After:

```toml
[export]
width = 1920
```

Rust export settings follow the same rule:

```rust
let settings = ExportSettings {
    width: 1920,
    ..ExportSettings::default()
};
```

`RenderOptions::resolution` is replaced by literal output width:

```rust
let options = RenderOptions {
    width: Some(1080),
    ..RenderOptions::default()
};
```
