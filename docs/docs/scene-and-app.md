---
sidebar_position: 5
---

# Scene & App

This guide covers how to work with `Scene` (the container for your animation) and `App` (the runtime that brings it to life).

For most authored code, prefer the higher-level APIs first:

- `scene.add_tattva(...)` instead of `scene.add(...)`
- scene intent helpers like `hide`, `show`, `set_position_2d`, `set_position_3d`, `set_scale`, and `set_rotation`
- `scene.play(...)` to install the global timeline

Use lower-level access only when you need type-specific mutation or generic scene tooling.

## Scene

`Scene` is the authoritative source of truth for your animation. It owns all tattvas, the global timeline, camera state, and scene time.

```rust
use murali::engine::scene::Scene;

let mut scene = Scene::new();
```

**What the scene owns:**
- All tattvas (visual objects)
- The global timeline
- Scene time (current playback position)
- Camera configuration
- Updaters (frame-by-frame callbacks)

## Adding Tattvas

The primary way to add objects to your scene is `add_tattva`:

```rust
use murali::frontend::collection::primitives::circle::Circle;
use glam::{Vec3, Vec4};

// Returns a TattvaId for later reference
let circle_id = scene.add_tattva(
    Circle::new(1.5, 48, Vec4::new(0.2, 0.6, 0.3, 1.0)),
    Vec3::new(0.0, 0.0, 0.0),  // position
);
```

**Key points:**
- `add_tattva(state, position)` is the preferred API for all user code
- It returns a `TattvaId` that you use to reference the tattva later
- The position is in world-space coordinates (not pixels)

### Multiple Tattvas

```rust
let title_id = scene.add_tattva(
    Label::new("My Animation", 0.32),
    Vec3::new(0.0, 3.0, 0.0),
);

let square_id = scene.add_tattva(
    Square::new(1.2, Vec4::new(0.9, 0.3, 0.3, 1.0)),
    Vec3::new(-2.0, 0.0, 0.0),
);

let circle_id = scene.add_tattva(
    Circle::new(0.8, 48, Vec4::new(0.2, 0.6, 0.3, 1.0)),
    Vec3::new(2.0, 0.0, 0.0),
);
```

### Advanced: scene.add()

`scene.add(...)` is a lower-level insertion API for already-wrapped tattva types. It's used internally and in advanced scenarios, but **you should prefer `add_tattva()` in normal code**.

```rust
// Advanced usage - not recommended for typical scenes
let tattva = state.into_tattva();
let id = scene.add(tattva);
```

## Modifying Tattvas

### Preferred: Intent Helpers

For common operations, use the scene's intent helpers. These are clear, type-safe, and work across all tattva types:

```rust
// Visibility
scene.hide(id);
scene.show(id);

// Opacity
scene.set_opacity(id, 0.5);

// Position
scene.set_position_2d(id, glam::Vec2::new(1.0, 2.0));
scene.set_position_3d(id, glam::Vec3::new(1.0, 2.0, 0.5));

// Transform
scene.set_scale(id, glam::Vec3::splat(1.25));
scene.set_rotation(id, glam::Quat::from_rotation_z(0.4));
```

**When to use:** Anytime you need to change position, visibility, opacity, scale, or rotation.

**Why these are better:** They're explicit about intent, work on any tattva type, and are easier to read than low-level access.

### Low-Level Access

When you need to modify type-specific properties (like a circle's radius or a label's text), use typed access:

```rust
// Typed mutable access
if let Some(circle) = scene.get_tattva_typed_mut::<Circle>(circle_id) {
    circle.state.radius = 2.0;
    circle.state.segments = 64;
}

// Typed immutable access
if let Some(label) = scene.get_tattva_typed::<Label>(label_id) {
    println!("Label text: {}", label.state.text);
}
```

**When to use:** When you need to access or modify properties specific to a tattva type.

Successful typed mutable access automatically marks the tattva for a full backend rebuild. You do
not need to call `mark_dirty` after changing `state` through this API.

### Untyped Access

For advanced scenarios where you don't know the concrete type:

```rust
if let Some(tattva) = scene.get_tattva_any(id) {
    println!("Tattva ID: {}", tattva.id());
}
```

**When to use:** Generic read-only code that works with any tattva type. Raw untyped mutation is
reserved for engine internals so authored state changes cannot bypass dirty tracking.

## Removing Tattvas

```rust
scene.remove_tattva(id);
```

The tattva is marked for removal and will be cleaned up during the next sync pass.

## Camera

The camera determines what part of the world is visible and how it's projected to the screen.

```rust
// Access the camera
scene.camera_mut().position = Vec3::new(0.0, 0.0, 10.0);
scene.camera_mut().set_view_width(16.0);
```

**Common camera setup for 2D scenes:**

```rust
// Position camera 10 units away, looking at origin
scene.camera_mut().position = Vec3::new(0.0, 0.0, 10.0);

// Set view width (how many world units fit horizontally)
scene.camera_mut().set_view_width(16.0);
```

**Key concepts:**
- **Position** - Where the camera is in world space
- **View width** - How many world units fit horizontally on screen
- **Look at** - What point the camera is aimed at (default: origin)

For more details, see the [Camera](./camera) guide.

## Layout Helpers

Murali provides helpers for common layout tasks:

### to_edge

Push a tattva to the edge of the frame with padding:

```rust
use murali::frontend::layout::Direction;

// Push title to top with 0.35 units of padding
scene.to_edge(title_id, Direction::Up, 0.35);

// Push footer to bottom
scene.to_edge(footer_id, Direction::Down, 0.5);

// Push to left or right
scene.to_edge(sidebar_id, Direction::Left, 0.3);
scene.to_edge(menu_id, Direction::Right, 0.3);
```

### next_to

Position one tattva relative to another:

```rust
// Place label next to circle, on the right side
scene.next_to(label_id, circle_id, Direction::Right, 0.5);
```

### align_to

Align one tattva to another:

```rust
use murali::frontend::layout::Anchor;

// Align label to circle's center
scene.align_to(label_id, circle_id, Anchor::Center);
```

For more layout patterns, see the Layout and Composition guide (coming soon).

## Timeline and Clips

The scene plays one global timeline. Clips organize reusable animation sections with their own local time starting at `0.0`.

### Direct Timeline Authoring

Most scenes use a single timeline:

```rust
let mut timeline = Timeline::new();

timeline
    .animate(circle_id)
    .at(0.0)
    .for_duration(2.0)
    .ease(Ease::OutCubic)
    .move_to(Vec3::new(3.0, 0.0, 0.0))
    .spawn();

scene.play(timeline)?;
```

Direct `.at(...)` values on a timeline are absolute scene times. This remains convenient for small scenes.

### Composing Local-Time Clips

Use clips when sections should be authored independently or rearranged without rewriting their internal timestamps:

```rust
let mut intro = Clip::new();
let mut explanation = Clip::new();

intro.animate(title_id).at(0.0).for_duration(1.0).appear().spawn();
explanation.animate(graph_id).at(0.0).for_duration(3.0).draw().spawn();

let mut timeline = Timeline::new();
timeline.append(intro).append(explanation);
scene.play(timeline)?;
```

Both clips use local time `0.0`; `append` converts those local times into one flat absolute schedule. Use `overlay` for concurrent clips and `place_at` for explicit global placement.

For composition semantics and examples, see the [Timelines and Clips](./timelines) guide.

## Capture Helpers

For exporting screenshots or GIFs at specific times:

### Screenshots

```rust
// Capture at specific times
scene.capture_screenshots([0.5, 1.0, 2.0]);

// Capture with custom filenames
scene.capture_screenshots_named([
    (0.5, Some("captures/intro.png")),
    (1.0, None::<&str>),  // Auto-generated name
    (2.0, Some("captures/final.png")),
]);
```

### GIFs

```rust
// Capture a GIF from multiple frames
scene.capture_gif("overview", [0.5, 1.0, 2.0]);
```

**When to use:** Creating documentation, generating thumbnails, debugging specific frames.

## Common Scene Patterns

### Title and Content

```rust
// Title at top
let title_id = scene.add_tattva(
    Label::new("My Topic", 0.36),
    Vec3::ZERO,
);
scene.to_edge(title_id, Direction::Up, 0.35);

// Subtitle below title
let subtitle_id = scene.add_tattva(
    Label::new("A detailed explanation", 0.18),
    Vec3::new(0.0, 2.5, 0.0),
);

// Main content in center
let content_id = scene.add_tattva(
    Circle::new(2.0, 48, color),
    Vec3::ZERO,
);
```

### Grid Layout

```rust
let spacing = 2.5;
let mut ids = Vec::new();

for row in 0..3 {
    for col in 0..3 {
        let x = (col as f32 - 1.0) * spacing;
        let y = (row as f32 - 1.0) * spacing;
        
        let id = scene.add_tattva(
            Square::new(0.8, color),
            Vec3::new(x, y, 0.0),
        );
        ids.push(id);
    }
}
```

### Staged Objects (Hidden Initially)

```rust
// Add object but keep it hidden
let circle_id = scene.add_tattva(
    Circle::new(1.0, 48, color),
    Vec3::ZERO,
);
scene.hide(circle_id);

// Later, reveal it with animation
timeline
    .animate(circle_id)
    .at(1.0)
    .for_duration(1.0)
    .ease(Ease::OutCubic)
    .appear()
    .spawn();
```

---

## App

`App` is the runtime that executes your scene — either as a live preview window or as a video export.

```rust
use murali::App;

App::new()?
    .with_scene(scene)
    .run_app()
```

## Preview vs Export

By default, `run_app()` checks command-line arguments to decide the mode:

- **No flags** → Export mode (renders video)
- **`--preview`** → Preview mode (opens window)
- **`--export`** → Export mode (explicit)

### Running from CLI

```bash
# Preview mode (interactive window)
cargo run --example my_scene --release -- --preview

# Export mode (renders video)
cargo run --example my_scene --release -- --export

# Default (export)
cargo run --example my_scene --release
```

### Forcing a Mode in Code

You can override the default behavior:

```rust
// Always open preview window
App::new()?
    .with_scene(scene)
    .with_preview()
    .run_app()

// Always export video
App::new()?
    .with_scene(scene)
    .with_video_export()
    .run_app()

// Export frames in addition to video export
App::new()?
    .with_scene(scene)
    .with_frames_export(true)
    .run_app()
```

## Preview Mode

Preview mode opens an interactive window where you can watch your animation play.

**Characteristics:**
- Advances one frame at a time
- Tries to maintain target FPS (usually 60)
- Slows down if your machine can't keep up (doesn't skip frames)
- Allows camera control with mouse/keyboard

**Why it doesn't skip frames:** Unlike games or video players, Murali preview prioritizes showing every frame so you can inspect motion, easing, and visual changes. If your machine is slow, preview will run slower than real-time, but you'll see every frame in order.

### Preview Controls

| Input | Action |
|---|---|
| **[O]** | Switch to orbit camera mode |
| **[P]** | Switch to pan/zoom camera mode |
| **Left drag** | Move camera (orbit or pan depending on mode) |
| **Scroll** | Zoom in/out |
| **[Esc]** | Exit preview |

**Camera modes:**
- **Orbit** - Rotate around a center point (good for 3D scenes)
- **Pan/Zoom** - Move camera in 2D plane (good for 2D scenes)

## Export Mode

Export mode renders your animation to video or image frames.

**Characteristics:**
- Renders at exact target FPS (deterministic)
- No window (headless rendering)
- Outputs to configured directory
- Can export video (MP4) or frames (PNG)

**Output location:** Check your project's export configuration or `RenderConfig` for output paths.

### Export Settings

```rust
use murali::engine::export::ExportSettings;

let settings = ExportSettings {
    artifact_dir: "my_animation".into(),
    fps: 60,
    duration_seconds: 10.0,
    // ... other settings
};

App::new()?
    .with_scene(scene)
    .with_export_settings(settings)
    .run_app()
```

## Render Options

Customize rendering behavior:

```rust
use murali::engine::render::RenderOptions;

let options = RenderOptions {
    video: Some(true),   // Enable video export
    frames: Some(true),  // Also export individual frames
    // ... other options
};

App::new()?
    .with_scene(scene)
    .with_render_options(options)
    .run_app()
```

## Common App Patterns

### Simple Preview

```rust
fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();
    // ... build scene
    
    App::new()?.with_scene(scene).run_app()
}
```

Run with: `cargo run --example my_scene -- --preview`

### Always Export

```rust
fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();
    // ... build scene
    
    App::new()?
        .with_scene(scene)
        .with_video_export()
        .run_app()
}
```

### Custom Export Settings

```rust
fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();
    // ... build scene
    
let settings = ExportSettings {
    artifact_dir: "tutorial".into(),
    fps: 60,
    duration_seconds: 15.0,
    ..Default::default()
};
    
    App::new()?
        .with_scene(scene)
        .with_export_settings(settings)
        .run_app()
}
```

## Best Practices

### Scene Management

✅ **Do:**
- Use `add_tattva()` for adding objects
- Use intent helpers (`set_position_2d`, `set_position_3d`, `hide`, `show`) for common operations
- Keep tattva IDs in variables for later reference
- Use `scene.play(timeline)` to install the scene's global timeline
- Stage objects (hide them) before reveal animations

❌ **Don't:**
- Use `scene.add()` unless you have a specific advanced need
- Forget to save tattva IDs returned by `add_tattva()`
- Try to animate tattvas that don't exist
- Modify tattva properties during animation (let timeline handle it)

### App Usage

✅ **Do:**
- Use `--preview` flag during development
- Use `--export` for final renders
- Test in preview mode before exporting
- Use release builds for smooth preview (`--release`)

❌ **Don't:**
- Export without previewing first
- Run preview in debug mode (it's slow)
- Forget to check export output directory
- Mix preview and export logic in the same run

## What's Next?

- **[Animations](./animations)** - Learn all animation verbs and patterns
- **[Camera](./camera)** - Control camera movement and framing
- **[Tattvas](./tattvas/)** - Explore all available visual objects
