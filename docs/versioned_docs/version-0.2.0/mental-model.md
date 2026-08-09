---
sidebar_position: 5
---

# Mental Model

Understanding Murali's core concepts will help you build animations more effectively. This page explains the fundamental building blocks and how they work together.

## The Big Picture

Murali animations follow a simple flow:

```
Scene → Tattvas → Timeline → Animations → App
```

1. You create a **Scene** (the container)
2. You add **Tattvas** (visual objects) to the scene
3. You build a **Timeline** (schedule of changes)
4. You add **Animations** to the timeline (what changes and when)
5. You run the **App** (preview or export)

## Scene

The `Scene` is the authoritative source of truth for your animation.

**What it owns:**
- All tattvas (visual objects)
- The global timeline
- The camera
- The logical landscape, portrait, or square frame
- Scene time (current playback position)
- Updaters (frame-by-frame callbacks)

**Key insight:** The scene is not just a container—it's the single source of truth. Everything that appears on screen is derived from the scene's current state.

```rust
let mut scene = Scene::new();

// The scene starts at time 0.0
// As time advances, animations mutate the scene state
// The renderer draws what the scene currently contains
```

Choose a non-landscape frame before placing objects or using frame-relative layout:

```rust
let mut scene = Scene::new().with_frame(Frame::portrait());
```

The frame defines composition; export width only determines pixel quality. See [Video Formats](./video-formats.md).

**Think of it like:** A stage in a theater. The stage holds all the actors (tattvas), knows what time it is in the performance (scene_time), and follows a script (timeline).

## Tattva

A **tattva** is any visual object in your scene. The word comes from Sanskrit, meaning "element" or "essence."

**Examples of tattvas:**
- Shapes: `Circle`, `Square`, `Line`, `Arrow`
- Text: `Label`, `Latex`, `Typst`, `CodeBlock`
- Math: `Equation`, `VectorFormula`
- Graphs: `ParametricCurve`, `ParametricSurface`
- Composite: `Table`, `NeuralNetworkDiagram`
- Utility: `TracedPath`, `ScreenshotMarker`

**Every tattva has:**
- A unique `TattvaId` (returned when you add it to the scene)
- A position in 3D space
- Common properties: scale, rotation, opacity, visibility
- Type-specific state: geometry, text content, colors, etc.

```rust
// Adding a tattva returns its ID
let circle_id = scene.add_tattva(
    Circle::new(1.0, 48, RED),
    ORIGIN,  // position
);

// You use the ID to animate or modify the tattva later
timeline.animate(circle_id).at(0.0).for_duration(2.0).move_to(...).spawn();
```

**Think of it like:** An actor on stage. Each actor has a position, appearance, and can be directed to move or change.

## Timeline

A **timeline** schedules when things happen and for how long.

**What it does:**
- Schedules animations at specific times
- Manages animation duration and easing
- Allows callbacks at specific moments
- Progresses as scene time advances

```rust
let mut timeline = Timeline::new();

// Animation 1: starts at t=0.0, lasts 2.0 seconds
timeline
    .animate(circle_id)
    .at(0.0)
    .for_duration(2.0)
    .ease(Ease::InOutQuad)
    .move_to(RIGHT * 3)
    .spawn();

// Animation 2: starts at t=2.5, lasts 1.5 seconds
timeline
    .animate(square_id)
    .at(2.5)
    .for_duration(1.5)
    .ease(Ease::OutCubic)
    .scale_to(Vec3::splat(2.0))
    .spawn();

// Play the timeline
scene.play(timeline)?;
```

**Key insight:** Animations are scheduled relative to scene time, not frame numbers. This makes them deterministic and reproducible.

**Think of it like:** A musical score. It tells each instrument (tattva) when to play (start time), for how long (duration), and how (animation verb).

## Animation Verbs

**Animation verbs** are the methods that actually change tattva properties over time.

**Common verbs:**
- `.move_to(position)` - Change position
- `.scale_to(scale)` - Change size
- `.rotate_to(quat)` - Change rotation
- `.fade_to(opacity)` - Change transparency
- `.appear()` - Fade in from invisible
- `.draw()` - Progressively reveal (for paths)
- `.typewrite_text()` - Reveal text character by character

```rust
timeline
    .animate(tattva_id)
    .at(start_time)
    .for_duration(duration)
    .ease(easing_function)
    .animation_verb()  // ← This is what changes
    .spawn();
```

**Think of it like:** Stage directions. "Move to stage left," "grow larger," "fade out."

## App

The `App` is the runtime that brings everything together.

**What it does:**
- Creates a window (in preview mode)
- Advances scene time each frame
- Triggers the render pipeline
- Handles export (in export mode)

```rust
App::new()?
    .with_scene(scene)
    .run_app()
```

**Two modes:**
1. **Preview mode** (`--preview` flag or `.with_preview()`) - Opens a window, runs interactively
2. **Export mode** (default, or `--export` flag) - Renders frames to video/images

**Think of it like:** The theater's lighting and sound crew. They make sure the performance runs smoothly, frame by frame.

## How They Work Together

Here's what happens when you run a Murali animation:

### 1. Setup Phase
```rust
let mut scene = Scene::new();
let circle_id = scene.add_tattva(...);
let mut timeline = Timeline::new();
timeline.animate(circle_id).at(0.0).for_duration(2.0).move_to(...).spawn();
scene.play(timeline)?;
```

At this point:
- Scene contains the circle at its initial position
- Timeline knows it should animate the circle from t=0.0 to t=2.0
- Nothing has moved yet

### 2. Runtime Phase
```rust
App::new()?.with_scene(scene).run_app()
```

Each frame:
1. **Time advances** - `scene_time` increases by `dt` (e.g., 1/60 second)
2. **Timeline ticks** - Checks which animations should be active at current time
3. **Animations apply** - Active animations mutate tattva properties
4. **Sync happens** - Changed tattvas are marked dirty and synced to GPU
5. **Render** - The current scene state is drawn to screen

### 3. The Flow

```
Frame N:
  scene_time = 0.5
  → Timeline checks: "circle animation is active (0.0 to 2.0)"
  → Animation applies: "circle should be 25% of the way to target"
  → Scene updates: circle.position = lerp(start, end, 0.25)
  → Renderer draws: circle at its new position

Frame N+1:
  scene_time = 0.517
  → Timeline checks: "circle animation still active"
  → Animation applies: "circle should be 25.85% of the way"
  → Scene updates: circle.position = lerp(start, end, 0.2585)
  → Renderer draws: circle at its new position
```

## Authored State vs Rendered Output

This is a crucial distinction:

**Authored state** (Frontend):
- Lives in the `Scene`
- Semantic and meaningful
- Example: "A circle with radius 1.0 at position (2, 3, 0)"

**Rendered output** (Backend):
- Lives in the GPU
- Optimized for drawing
- Example: "A mesh with 48 vertices, uploaded to buffer #42"

The **sync boundary** translates authored state into rendered output only when needed. This keeps the architecture clean and allows optimizations like:
- Transform-only changes don't rebuild geometry
- Invisible objects don't get synced
- Dirty flags track what actually changed

## World Space vs Pixels

Murali uses **world space coordinates**, not pixels.

```rust
// This circle has radius 1.0 in world units
let circle = Circle::new(1.0, 48, color);

// Position is also in world units
scene.add_tattva(circle, Vec3::new(2.0, 3.0, 0.0));

//or
// scene.add_tattva(circle, RIGHT * 2 + UP * 3);
```

The camera determines how world space maps to screen pixels:

```rust
scene.camera_mut().set_view_width(16.0);
// Now 16 world units fit horizontally on screen
```

**Why world space?**
- Mathematical precision (1.0 means 1.0, not "approximately 100 pixels")
- Resolution independence (same scene works at any resolution)
- Easier reasoning about geometry and layout

## One Timeline and Local-Time Clips

Each scene plays one global timeline:

```rust
let mut timeline = Timeline::new();
// ... add all animations
scene.play(timeline)?;
```

Complex sections are authored as clips, each beginning at local time `0.0`:

```rust
let mut intro = Clip::new();
let mut background = Clip::new();

// Add animations using times local to each clip.

let mut timeline = Timeline::new();
timeline.append(intro).overlay(background);
scene.play(timeline)?;
```

Composition converts clip-local times to absolute scene times before playback. There is still only one runtime clock and one deterministic schedule.

:::tip Learn More
For a deeper dive into timelines, callbacks, and advanced scheduling, see the [Timelines](./timelines) guide.
:::

## Key Takeaways

1. **Scene is the source of truth** - Everything visible comes from scene state
2. **Tattvas are semantic objects** - They represent what you mean, not how to draw it
3. **The timeline schedules changes** - Clips provide local authoring frames without adding clocks
4. **Animations are time-driven** - Based on time, not frames
5. **World space is mathematical** - Coordinates are precise, not pixel-based
6. **The flow is one-way** - Scene → Projection → Backend → Renderer

## What's Next?

Now that you understand the mental model:

- **[Animations](./animations)** - Learn all the animation verbs
- **[Scene and App](./scene-and-app)** - Deeper dive into scene management
- **[Tattvas](./tattvas/)** - Explore all available visual objects
- **[Camera](./camera)** - Control framing and perspective
- **[Architecture Overview](./architecture/overview)** - How it works under the hood
