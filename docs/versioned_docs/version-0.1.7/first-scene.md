---
sidebar_position: 4
---

# Your First Scene

This guide walks you through creating a minimal Murali scene from scratch. By the end, you'll have a working animation with shapes, text, and motion.

## What You'll Build

A simple scene with:
- A title label
- Two shapes (a square and a circle)
- Smooth animated motion between positions

## Prerequisites

Make sure you have Murali installed. If not, see the [Introduction](./intro.mdx) for setup instructions.

## Step 1: Create a New File

Create a new file in your project's `examples/` directory or in a new binary crate:

```bash
# If using the Murali repo
touch examples/my_first_scene.rs

# Or create a new binary project
cargo new --bin my_first_scene
cd my_first_scene
# Add murali to Cargo.toml dependencies
```

## Step 2: Import What You Need

Start with the essential imports:

```rust
use glam::{Vec3, Vec4};
use murali::App;
use murali::colors::*;
use murali::positions::*;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::animation::Ease;
use murali::frontend::collection::primitives::{circle::Circle, square::Square};
use murali::frontend::collection::text::label::Label;

fn main() -> anyhow::Result<()> {
    // We'll build the scene here
    Ok(())
}
```

## Step 3: Create a Scene

The `Scene` is the container for all your objects:

```rust
fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    // More code will go here

    Ok(())
}
```

## Step 4: Add a Title

Add a text label at the top of your scene:

```rust
fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    // Add a title
    scene.add_tattva(
        Label::new("My First Scene", 0.32)
            .with_color(WHITE),
        3.0 * UP,
    );

    Ok(())
}
```

**What's happening here:**
- `Label::new("My First Scene", 0.32)` creates a text label with font size 0.32
- `.with_color(...)` sets the text color (RGBA values from 0.0 to 1.0)
- `3.0 * UP` positions the label at coordinates (x=0, y=3, z=0)

## Step 5: Add Shapes

Now add two shapes that we'll animate:

```rust
fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    // Title
    scene.add_tattva(
        Label::new("My First Scene", 0.32)
            .with_color(WHITE),
        3.0 * UP,
    );

    // Add a red square on the left
    let square_id = scene.add_tattva(
        Square::new(1.2, RED_B),
        4.0 * LEFT,
    );

    // Add a green circle on the right
    let circle_id = scene.add_tattva(
        Circle::new(0.65, 48, GREEN_D),
        4.0 * RIGHT,
    );

    Ok(())
}
```

**Important:** Notice we're saving the IDs returned by `add_tattva`. We'll need these to animate the shapes.

**Parameters explained:**
- `Square::new(1.2, color)` - size of 1.2 units, with a red color
- `Circle::new(0.65, 48, color)` - radius of 0.65, 48 segments for smoothness, green color

## Step 6: Set Up the Camera

Configure the camera to frame your scene properly:

```rust
fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    // ... (previous code for title and shapes)

    // Configure camera
    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    scene.camera_mut().set_view_width(16.0);

    Ok(())
}
```

The camera is positioned at z=10 (looking toward the origin) with a view width of 16 units.

## Step 7: Create a Timeline and Add Animations

Now for the fun part - making things move:

```rust
fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    // ... (previous code)

    // Create a timeline
    let mut timeline = Timeline::new();

    // Animate the square
    timeline
        .animate(square_id)
        .at(0.0)                              // Start at time 0
        .for_duration(2.0)                    // Take 2 seconds
        .ease(Ease::InOutQuad)                // Smooth easing
        .move_to(2.0 * RIGHT)    // Move to new position
        .spawn();

    // Animate the circle
    timeline
        .animate(circle_id)
        .at(0.5)                              // Start at time 0.5
        .for_duration(2.0)                    // Take 2 seconds
        .ease(Ease::OutQuad)                  // Different easing
        .move_to(2.0 * LEFT)   // Move to new position
        .spawn();

    Ok(())
}
```

**What's happening:**
- We create animations using the builder pattern
- `.animate(id)` targets a specific tattva by its ID
- `.at(time)` sets when the animation starts
- `.for_duration(seconds)` sets how long it takes
- `.ease(...)` controls the motion curve (smooth vs linear)
- `.move_to(position)` is the animation verb - what actually changes
- `.spawn()` adds the animation to the timeline

## Step 8: Play the Timeline

Tell the scene to use this timeline:

```rust
fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    // ... (all previous code)

    // Play the timeline
    scene.play(timeline)?;

    Ok(())
}
```

## Step 9: Run the App

Finally, create and run the app:

```rust
fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    // ... (all previous code)

    // Run the app
    App::new()?.with_scene(scene).run_app()
}
```

## Complete Code

Here's the full example:

```rust
use glam::{Vec3, Vec4};
use murali::App;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::animation::Ease;
use murali::frontend::collection::primitives::{circle::Circle, square::Square};
use murali::frontend::collection::text::label::Label;

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();

    // Title
    scene.add_tattva(
        Label::new("My First Scene", 0.32)
            .with_color(WHITE),
        3.0 * UP,
    );

    // Shapes
    let square_id = scene.add_tattva(
        Square::new(1.2, RED_B),
        4.0 * LEFT,
    );

    let circle_id = scene.add_tattva(
        Circle::new(0.65, 48, GREEN_D),
        4.0 * RIGHT,
    );

    // Camera
    scene.camera_mut().position = CAMERA_DEFAULT_POS;
    scene.camera_mut().set_view_width(16.0);

    // Timeline
    let mut timeline = Timeline::new();

    timeline
        .animate(square_id)
        .at(0.0)
        .for_duration(2.0)
        .ease(Ease::InOutQuad)
        .move_to(2.0 * RIGHT)
        .spawn();

    timeline
        .animate(circle_id)
        .at(0.5)
        .for_duration(2.0)
        .ease(Ease::OutQuad)
        .move_to(2.0 * LEFT)
        .spawn();

    scene.play(timeline)?;

    // Run
    App::new()?.with_scene(scene).run_app()
}
```

## Running Your Scene

### Preview Mode

To see your animation in a window:

```bash
cargo run --example my_first_scene --release -- --preview
```

Without `--preview`, Murali exports by default rather than opening an interactive window.

For explicit export:

```bash
cargo run --example my_first_scene --release -- --export
```

**Controls:**
- **[O]** - Switch to orbit camera mode
- **[P]** - Switch to pan/zoom camera mode
- **[Drag]** - Move the camera
- **[Scroll]** - Zoom in/out
- **[Esc]** - Exit

### Export Mode

To export your animation as a video:

```bash
cargo run --example my_first_scene --release
```

Or explicitly:

```bash
cargo run --example my_first_scene --release -- --export
```

This will create an MP4 file in your output directory.

## Understanding the Coordinate System

Murali uses a right-handed coordinate system:
- **X-axis**: Left (negative) to right (positive)
- **Y-axis**: Down (negative) to up (positive)
- **Z-axis**: Into screen (negative) to out of screen (positive)

The origin (0, 0, 0) is at the center of the frame.

## Key Concepts

### Scene
The container for all objects, the global timeline, and state. It's the source of truth for your animation.

### Tattva
Any visual object in Murali (shapes, text, graphs, etc.). Each tattva has:
- A unique ID
- A position in 3D space
- Properties like color, scale, rotation, opacity

### Timeline
The scene's global time axis. It schedules when animations happen and how long they take.

### Clip
A reusable animation section authored from its own local time `0.0`, then appended, overlaid, or explicitly placed on the timeline.

### Animation Verbs
Methods that change tattva properties over time:
- `.move_to(position)` - Change position
- `.scale_to(scale)` - Change size
- `.rotate_to(quat)` - Change rotation
- `.fade_to(opacity)` - Change opacity
- `.appear()` - Fade in from invisible
- And many more...

## What's Next?

Now that you have a working scene, here's your recommended learning path:

**Core Concepts (Start Here):**
1. **[Mental Model](./mental-model)** - Understand how Scene, Tattva, and Timeline work together
2. **[Which API Should I Use?](./which-api-should-i-use)** - Decision guide for common tasks
3. **[Common First Mistakes](./common-first-mistakes)** - Avoid common pitfalls

**Expand Your Skills:**
- **[Animations](./animations)** - Complete reference for all animation verbs
- **[Tattvas](./tattvas/)** - Discover all shapes, text, graphs, and components
- **[Scene and App](./scene-and-app)** - Advanced scene management
- **[Camera](./camera)** - Control camera movement and framing

**By Use Case:**
- Teaching math → [Text](./tattvas/text), [Math](./tattvas/math), [Graphs](./tattvas/graphs)
- Data visualization → [Graphs](./tattvas/graphs), [Tables](./tattvas/tables)
- AI/ML explanations → [AI Tattvas](./tattvas/ai), [Storytelling](./tattvas/storytelling)

## Common First Mistakes

### Shapes Not Visible
- Check that your camera is positioned correctly (usually at positive Z)
- Verify your shapes are within the camera's view width
- Make sure colors have alpha = 1.0 (fully opaque)

### Animations Not Playing
- Did you call `scene.play(timeline)` before running the app?
- Check that animation start times (`.at(...)`) are reasonable
- Verify you're using the correct tattva IDs

### Colors Look Wrong
- Colors use RGBA format with values from 0.0 to 1.0
- Example: `Vec4::new(1.0, 0.0, 0.0, 1.0)` is bright red
- The fourth value is alpha (transparency)

## Troubleshooting

**"Cannot find tattva with ID"**
- You're trying to animate a tattva that doesn't exist
- Make sure you saved the ID from `add_tattva` and used it correctly

**Window opens but nothing appears**
- Your objects might be outside the camera view
- Try adjusting camera position or view width
- Check that your positions make sense (not too far from origin)

**Animation happens instantly**
- Check your `.for_duration(...)` value - it should be > 0
- Verify your timeline is actually being played with `scene.play(timeline)`
