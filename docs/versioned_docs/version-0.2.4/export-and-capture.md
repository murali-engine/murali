---
sidebar_position: 11
---

# Export And Capture

Murali supports two main workflows:

- preview a scene interactively while authoring
- export video, screenshots, GIF captures, and optional raw frames for deliverables

The most important thing to know is that preview and export are different modes. Preview opens a window for interactive inspection. Export renders deterministically from `scene_time` and writes output to disk.

## Preview Vs Export

Preview is for authoring:

```rust
App::new()?
    .with_scene(scene)
    .with_preview()
    .run_app()?;
```

or from the CLI:

```bash
cargo run --example murali_logo_transparent -- --preview
```

Export is the default behavior if you do not opt into preview:

```rust
App::new()?
    .with_scene(scene)
    .run_app()?;
```

or:

```bash
cargo run --example murali_logo_transparent -- --export
```

If you want a capture-only export with no main frame sequence and no MP4, use:

```bash
cargo run --example murali_logo_transparent -- --export --no-video
```

Use preview when you are iterating on timing, layout, or camera choices. Use export when you want reproducible assets.

## Basic Export Controls

`App` exposes a few high-level helpers:

```rust
App::new()?
    .with_scene(scene)
    .with_video_export()
    .with_frames_export(true)
    .run_app()?;
```

What these mean:

- `.with_preview()` switches into interactive preview mode
- `.with_video_export()` ensures video export is enabled
- `.with_frames_export(true)` preserves the raw PNG frame sequence after export

By default, Murali exports through an internal frame sequence and then cleans up that raw frame directory after successful video assembly. Use `.with_frames_export(true)` when you want to keep those PNG frames on disk.

There is currently no separate CLI flag named `preserve_frame_exports`. The public switch is:

- Rust API: `.with_frames_export(true)`
- project config: `[export] preserve_frame_exports = true`

If you want authored screenshots and GIF captures without producing the main frame sequence or MP4, use:

- Rust API: `ExportSettings { video_enabled: false, ..Default::default() }`
- project config: `[export] video_enabled = false`
- CLI: `--no-video`

## Render Options

Use `RenderOptions` when you want to control output behavior without building a full `ExportSettings` value yourself.

```rust
use murali::engine::render::RenderOptions;

let options = RenderOptions {
    video: Some(true),
    frames: Some(true),
    fps: Some(60),
    width: Some(1920),
};

App::new()?
    .with_scene(scene)
    .with_render_options(options)
    .run_app()?;
```

Use this path when you mainly care about:

- output width
- frame rate
- whether raw frames should be preserved in addition to final media

The scene frame owns the aspect ratio. Export specifies only the final pixel width, and Murali derives the height:

| Scene frame | Export width | Output dimensions |
| --- | ---: | ---: |
| `Frame::landscape()` | 1920 | 1920×1080 |
| `Frame::portrait()` | 1080 | 1080×1920 |
| `Frame::square()` | 1200 | 1200×1200 |

Changing export width changes image quality, never scene composition.

See [Video Formats](./video-formats.md) for complete Shorts, landscape, and square examples.

## ExportSettings

Use `ExportSettings` when you want more direct control over the export pipeline:

```rust
use glam::Vec4;
use murali::engine::export::{ExportSettings, PngCompressionMode, export_scene};

let settings = ExportSettings {
    artifact_dir: "demo".into(),
    width: 1920,
    fps: 60,
    duration_seconds: 6.0,
    video_enabled: true,
    preserve_frame_exports: true,
    clear_color: Vec4::new(0.05, 0.10, 0.15, 1.0),
    png_compression: PngCompressionMode::Balanced,
};

export_scene(scene, &settings)?;
```

Reach for `ExportSettings` when you want to control:

- artifact root
- exact duration
- whether video should be assembled
- whether raw frame exports should be kept
- clear color
- PNG compression mode

`video_enabled` defaults to `true`, so you only need to set it when you want a capture-only export.

`rendered_output` is always the top-level export root.

`artifact_dir` means the folder inside `rendered_output`. So with:

```rust
artifact_dir: "demo".into()
```

Murali writes to:

- `rendered_output/demo/frames`
- `rendered_output/demo/captures`
- `rendered_output/demo/gifs`
- `rendered_output/demo/demo.mp4`

If video is disabled, Murali skips the main frame sequence and MP4, but still honors explicit screenshot captures and authored GIF capture groups.

### PNG compression modes

PNG export is lossless, but you can choose the tradeoff between export speed and frame size.

Available modes:

- `PngCompressionMode::Fast` — quickest frame writes, larger PNG files
- `PngCompressionMode::Balanced` — middle ground between speed and file size
- `PngCompressionMode::Smallest` — smaller PNG files, slower frame writes

Example:

```rust
use murali::engine::export::{ExportSettings, PngCompressionMode};

let settings = ExportSettings {
    png_compression: PngCompressionMode::Balanced,
    ..Default::default()
};
```

Or in `murali.toml`:

```toml
[export]
png_compression = "balanced"
```

Accepted values are:

- `fast`
- `balanced`
- `smallest`

You can also preserve the raw frame directory from `murali.toml`:

```toml
[export]
preserve_frame_exports = true
png_compression = "balanced"
```

### Transparent PNG backgrounds

Murali can export PNG frames and screenshots with a transparent background.

Set the export clear color alpha to `0.0`:

```rust
use glam::Vec4;
use murali::engine::export::ExportSettings;

let settings = ExportSettings {
    clear_color: Vec4::new(0.0, 0.0, 0.0, 0.0),
    ..Default::default()
};
```

This is especially useful for:

- logos
- overlays
- slide assets
- compositing in video editors or design tools

PNG preserves this transparency. MP4 export does not preserve transparent backgrounds in this pipeline, and GIF transparency is more limited.

## Screenshots At Authored Times

If you want screenshots from specific authored moments, schedule them on the `Scene`:

```rust
scene.capture_screenshots([0.5, 1.0, 2.0]);
```

If you want named output files:

```rust
scene.capture_screenshots_named([
    (0.5, Some("captures/intro.png")),
    (1.0, None),
    (2.0, Some("captures/final.png")),
]);
```

This is usually the nicest authoring model for tutorials, blog posts, and docs assets because the capture schedule lives with the scene itself.

## GIF Capture Groups

You can also schedule small GIF capture groups from authored times:

```rust
scene.capture_gif("overview", [0.5, 1.0, 2.0]);
scene.capture_gif("focus", [2.5, 2.7, 2.9]);
```

Those captures are assembled into GIFs under `artifact_dir/gifs`.

This is useful when you want:

- short explainer GIFs from a longer scene
- multiple derived assets from one authored scene
- social/documentation snippets without reauthoring the animation

## GIF Outputs

Murali supports authored GIF capture groups through `scene.capture_gif(...)`.

So if you want several GIFs from one scene, the intended path is:

```rust
scene.capture_gif("overview", [0.5, 1.0, 2.0]);
scene.capture_gif("focus", [2.5, 2.7, 2.9]);
```

That will create one GIF per capture group inside `artifact_dir/gifs`.

## ScreenshotMarker

`ScreenshotMarker` is a utility tattva for capture-oriented scenes, but most authored capture workflows should prefer the scene-level helpers above first:

- `scene.capture_screenshots(...)`
- `scene.capture_screenshots_named(...)`
- `scene.capture_gif(...)`

Use `ScreenshotMarker` when you explicitly want capture intent represented as a tattva in the scene graph.

## Deterministic Export

Export advances the scene from authored time rather than relying on interactive preview timing. Frames normally land on the configured FPS grid. When the requested duration is not frame-aligned, Murali adds a final sample at the exact terminal time so the export cannot stop before the timeline's authored final state.

That makes export the right mode for:

- final deliverables
- reproducible screenshots
- asset generation in CI or scripted workflows
- checking whether authored timing really lands as expected

## Practical Advice

- use preview for iteration speed
- use export when checking the final timing and framing
- use scene-level capture helpers for doc/blog assets
- use `RenderOptions` for lightweight output control
- use `ExportSettings` for production-style exports

## Related Docs

- [Scene and App](./scene-and-app)
- [Examples](./examples/showcase)
- [Utility Tattvas](./tattvas/utility)
