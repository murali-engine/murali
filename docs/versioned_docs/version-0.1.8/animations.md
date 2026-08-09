---
sidebar_position: 4
---

# Animations

Murali uses a `Timeline` to schedule animations. Animations are **time-driven** — they depend on elapsed time, not frame count. This makes them deterministic, reproducible, and easy to reason about.

Prefer concrete animation verbs first. Reach for callbacks like `call_at(...)` and `call_during(...)` only when the built-in authored verbs do not express the motion or state change you need.

## Quick Decision Guide

**Choose your animation based on what you want to achieve:**

| What do you want? | Use this | When NOT to use it |
|---|---|---|
| Move an object | `.move_to(position)` | Object needs complex path (use updater or `call_during`) |
| Change size | `.scale_to(scale)` | Non-uniform scaling on specific tattva types |
| Rotate an object | `.rotate_to(quaternion)` | For 2D rotation, use `Quat::from_rotation_z(angle)` |
| Fade in/out | `.appear()` / `.fade_to(opacity)` | You want an immediate visibility change (use scene helpers like `hide` / `show`) |
| Draw a path progressively | `.draw()` / `.undraw()` | Not a path-like tattva (circles, squares don't draw) |
| Type text character-by-character | `.typewrite_text()` | Text should appear all at once (use `.appear()`) |
| Reveal text with shift effect | `.reveal_text()` | Simple fade is enough (use `.appear()`) |
| Build a table row-by-row | `.write_table()` | Table should appear instantly |
| Morph one shape into another | `.morph_from(source_id)` | Shapes are very different (may look strange) |
| Match another object's transform | `.match_transform(id)` | You want to animate to a specific position |
| Run custom logic | `.call_at()` / `.call_during()` | Simple property change (use animation verb) |

## Basic Setup

```rust
use murali::engine::timeline::Timeline;
use murali::frontend::animation::Ease;

let mut timeline = Timeline::new();

timeline
    .animate(square_id)
    .at(0.0)                              // Start time in seconds
    .for_duration(2.0)                    // Duration in seconds
    .ease(Ease::InOutQuad)                // Easing function
    .move_to(Vec3::new(3.0, 0.0, 0.0))    // Animation verb
    .spawn();                             // Add to timeline

scene.play(timeline)?;
```

**Key concepts:**
- `.at(time)` - When the animation starts (in seconds)
- `.for_duration(seconds)` - How long it takes
- `.ease(...)` - How the motion feels (linear, smooth, bouncy, etc.)
- `.animation_verb()` - What actually changes
- `.spawn()` - Adds the animation to the timeline

For larger scenes, author related animations in a `Clip`. A clip's `.at(...)` values are local to that clip, beginning at `0.0`; `Timeline::append`, `overlay`, or `place_at` converts them to absolute scene times.

## Animation Verbs

### Transform Animations

#### move_to

Moves a tattva to a world-space position.

```rust
timeline
    .animate(circle_id)
    .at(0.0)
    .for_duration(2.0)
    .ease(Ease::OutCubic)
    .move_to(Vec3::new(2.0, 1.0, 0.0))
    .spawn();
```

Optionally specify a starting position (useful for objects that should move from somewhere other than their current position):

```rust
timeline
    .animate(circle_id)
    .at(0.0)
    .for_duration(2.0)
    .ease(Ease::OutCubic)
    .move_to(Vec3::new(2.0, 1.0, 0.0))
    .from_vec3(Vec3::new(-2.0, 0.0, 0.0))
    .spawn();
```

**When to use:** Moving objects between positions, creating motion paths, layout transitions.

**When NOT to use:** Complex curved paths (use `call_during` with custom logic instead).

#### scale_to

Scales a tattva uniformly or non-uniformly.

```rust
// Uniform scaling
timeline
    .animate(square_id)
    .at(0.0)
    .for_duration(1.5)
    .ease(Ease::OutBack)
    .scale_to(Vec3::splat(2.0))  // 2x in all directions
    .spawn();

// Non-uniform scaling
timeline
    .animate(rectangle_id)
    .at(0.0)
    .for_duration(1.5)
    .ease(Ease::InOutQuad)
    .scale_to(Vec3::new(3.0, 1.0, 1.0))  // Stretch horizontally
    .spawn();
```

**When to use:** Growing/shrinking objects, emphasis effects, size transitions.

**When NOT to use:** Some tattvas may have specific sizing APIs that are more appropriate.

#### rotate_to

Rotates a tattva using quaternions.

```rust
use glam::Quat;

timeline
    .animate(arrow_id)
    .at(0.0)
    .for_duration(2.0)
    .ease(Ease::InOutQuad)
    .rotate_to(Quat::from_rotation_z(std::f32::consts::PI))
    .spawn();
```

**When to use:** Rotating objects in 3D space, orientation changes.

**Tip:** For simple 2D rotation around Z-axis, use:
```rust
.rotate_to(Quat::from_rotation_z(angle_in_radians))
```

### Visibility Animations

#### appear

Reveals a tattva by animating its opacity from 0 to 1.

```rust
timeline
    .animate(circle_id)
    .at(0.0)
    .for_duration(1.0)
    .ease(Ease::OutCubic)
    .appear()
    .spawn();
```

**When to use:** Simple fade-in effects, revealing objects smoothly.

**Note:** The tattva should be staged (hidden initially) for this to work as expected. Use `scene.hide(id)` before the animation if needed.

**Legacy alias:** `.create()` still works but `.appear()` is clearer.

#### fade_to

Animates opacity to a specific value.

```rust
// Fade out
timeline
    .animate(label_id)
    .at(2.0)
    .for_duration(1.0)
    .ease(Ease::InCubic)
    .fade_to(0.0)
    .spawn();

// Fade to semi-transparent
timeline
    .animate(background_id)
    .at(0.0)
    .for_duration(0.5)
    .ease(Ease::Linear)
    .fade_to(0.3)
    .spawn();
```

**When to use:** Fade in/out effects, transparency transitions, layering effects.

**When NOT to use:** If you just want to hide something immediately, use `scene.hide(id)` instead.

### Path Animations

#### draw / undraw

Progressively reveals or hides path-like geometry.

```rust
// Draw a line from start to end
timeline
    .animate(line_id)
    .at(0.0)
    .for_duration(2.0)
    .ease(Ease::OutCubic)
    .draw()
    .spawn();

// Undraw it later
timeline
    .animate(line_id)
    .at(4.0)
    .for_duration(1.5)
    .ease(Ease::InCubic)
    .undraw()
    .spawn();
```

**When to use:** Lines, arrows, paths, curves, vector graphics.

**When NOT to use:** Filled shapes like circles or squares (they don't have a natural "draw" progression).

**Legacy aliases:** `.write()` / `.unwrite()` still work.

### Text Animations

#### typewrite_text / untypewrite_text

Reveals text character by character, like typing.

```rust
timeline
    .animate(label_id)
    .at(0.0)
    .for_duration(2.0)
    .ease(Ease::Linear)  // Usually linear for typing effect
    .typewrite_text()
    .spawn();
```

**When to use:** Code snippets, terminal output, dramatic text reveals.

**When NOT to use:** Math equations (characters may not align properly), short labels (too slow).

#### reveal_text / hide_text

Reveals text with a centered shifting effect.

```rust
timeline
    .animate(title_id)
    .at(0.0)
    .for_duration(1.5)
    .ease(Ease::OutCubic)
    .reveal_text()
    .spawn();
```

**When to use:** Titles, headings, emphasis text.

**When NOT to use:** Long paragraphs (effect may be too busy), code blocks.

### Structured Object Animations

#### write_table / unwrite_table

Builds a table row by row or cell by cell.

```rust
timeline
    .animate(table_id)
    .at(0.0)
    .for_duration(3.0)
    .ease(Ease::OutCubic)
    .write_table()
    .spawn();
```

**When to use:** Data tables, matrices, structured data reveals.

#### write_surface / unwrite_surface

Progressively reveals a parametric surface.

```rust
timeline
    .animate(surface_id)
    .at(0.0)
    .for_duration(4.0)
    .ease(Ease::InOutQuad)
    .write_surface()
    .spawn();
```

**When to use:** 3D surfaces, mathematical visualizations, terrain reveals.

### Morphing Animations

#### morph_from

Morphs one tattva's shape into another.

```rust
// Hide the target initially
scene.hide(circle_id);

// Morph square into circle
timeline
    .animate(circle_id)
    .at(0.5)
    .for_duration(2.0)
    .ease(Ease::InOutQuad)
    .morph_from(square_id)
    .spawn();
```

**When to use:** Shape transitions, transformations, visual metaphors.

**When NOT to use:**
- Shapes are very different (may look strange)
- You need precise control over intermediate states
- Shapes have different numbers of vertices (may need manual vertex matching)

**Important:** Always hide the target tattva before morphing, or it will be visible at its final state.

#### morph_matching_staged

Morphs one group of tattvas into another while automatically staging the target group.

```rust
timeline.morph_matching_staged(
    source_ids,
    target_ids,
    &mut scene,
    1.0,        // start time
    3.0,        // duration
    Ease::InOutCubic,
);
```

**When to use:** Transitioning between multiple objects, equation transformations, diagram transitions.

**Note:** This is a helper that handles staging for you. Use raw `morph_matching(...)` only when you need manual control.

#### match_transform

Snaps a tattva's transform to match another tattva's.

```rust
timeline
    .animate(copy_id)
    .at(0.0)
    .for_duration(1.0)
    .ease(Ease::OutCubic)
    .match_transform(original_id)
    .spawn();
```

**When to use:** Synchronizing positions, creating copies, alignment animations.

**When NOT to use:** You know the exact target position (use `.move_to()` instead).

## Easing Functions

Easing functions control how an animation progresses over time. They affect the "feel" of motion.

| Easing | Feel | Best for |
|---|---|---|
| `Ease::Linear` | Constant speed | Mechanical motion, technical diagrams |
| `Ease::InQuad` | Slow start, fast end | Falling objects, gravity |
| `Ease::OutQuad` | Fast start, slow end | Coming to rest, deceleration |
| `Ease::InOutQuad` | Slow start and end | Natural motion, smooth transitions |
| `Ease::InCubic` | Stronger slow start | Dramatic acceleration |
| `Ease::OutCubic` | Stronger slow end | Smooth landing |
| `Ease::InOutCubic` | Stronger smooth both ends | Elegant motion |
| `Ease::InOutSmooth` | Smoothstep (C1 continuous) | Very smooth, organic motion |

**Choosing an easing:**
- **OutCubic** - Default choice for most animations
- **InOutQuad** - When you want symmetrical smoothness
- **Linear** - Typing effects, technical motion
- **OutQuad** - Quick but gentle stops

```rust
// Smooth, natural motion
.ease(Ease::OutCubic)

// Symmetrical, elegant
.ease(Ease::InOutQuad)

// Mechanical, constant speed
.ease(Ease::Linear)
```

## Callbacks

Sometimes you need to run custom code at specific times or over a duration.

### call_at

Run code once at a specific time:

```rust
timeline.call_at(2.0, |scene| {
    // This runs once when scene_time reaches 2.0
    println!("Halfway through!");

    // You can modify the scene here
    scene.hide(some_id);
});
```

**When to use:** Discrete events, state changes, logging, cleanup.

### call_during

Run code continuously over a duration:

```rust
timeline.call_during(1.0, 2.0, |scene, t| {
    // This runs every frame between t=1.0 and t=3.0
    // t is normalized from 0.0 to 1.0

    // Example: custom motion path
    let angle = t * std::f32::consts::TAU;
    let position = Vec3::new(
        angle.cos() * 3.0,
        angle.sin() * 3.0,
        0.0
    );
    scene.set_position_3d(circle_id, position);
});
```

**When to use:**
- Complex motion paths (circles, spirals, custom curves)
- Dependent motion (one object following another)
- Custom interpolation logic
- Procedural animations

**When NOT to use:** Simple property changes (use animation verbs instead).

## Camera Animations

Animate the camera just like any other object:

```rust
use murali::frontend::animation::camera_animation_builder::CameraAnimationBuilder;

timeline
    .animate_camera()
    .at(0.0)
    .for_duration(2.0)
    .ease(Ease::InOutQuad)
    .move_to(Vec3::new(0.0, 0.0, 5.0))
    .spawn();
```

**Available camera animations:**
- `.move_to(position)` - Move camera position
- `.look_at(target)` - Point camera at target
- `.set_view_width(width)` - Zoom in/out

## Sequencing Animations

### Sequential (one after another)

```rust
let mut timeline = Timeline::new();

// First animation: 0.0 to 2.0
timeline.animate(id1).at(0.0).for_duration(2.0).move_to(...).spawn();

// Second animation: 2.0 to 4.0 (starts when first ends)
timeline.animate(id2).at(2.0).for_duration(2.0).move_to(...).spawn();

// Third animation: 4.0 to 5.5
timeline.animate(id3).at(4.0).for_duration(1.5).move_to(...).spawn();
```

### Parallel (at the same time)

```rust
let mut timeline = Timeline::new();

// All start at t=0.0
timeline.animate(id1).at(0.0).for_duration(2.0).move_to(...).spawn();
timeline.animate(id2).at(0.0).for_duration(2.0).scale_to(...).spawn();
timeline.animate(id3).at(0.0).for_duration(2.0).fade_to(...).spawn();
```

### Staggered (overlapping)

```rust
let mut timeline = Timeline::new();
let stagger = 0.2;  // 0.2 second delay between each

for (i, id) in tattva_ids.iter().enumerate() {
    timeline
        .animate(*id)
        .at(i as f32 * stagger)
        .for_duration(1.0)
        .ease(Ease::OutCubic)
        .appear()
        .spawn();
}
```

## Common Patterns

### Fade In, Wait, Fade Out

```rust
let mut timeline = Timeline::new();

// Fade in
timeline.animate(id).at(0.0).for_duration(1.0).appear().spawn();

// (Visible from t=1.0 to t=4.0)

// Fade out
timeline.animate(id).at(4.0).for_duration(1.0).fade_to(0.0).spawn();
```

### Move and Scale Together

```rust
timeline
    .animate(id)
    .at(0.0)
    .for_duration(2.0)
    .ease(Ease::OutCubic)
    .move_to(Vec3::new(3.0, 0.0, 0.0))
    .spawn();

timeline
    .animate(id)
    .at(0.0)
    .for_duration(2.0)
    .ease(Ease::OutCubic)
    .scale_to(Vec3::splat(2.0))
    .spawn();
```

### Reveal Text Then Draw Arrow

```rust
// Text appears first
timeline
    .animate(label_id)
    .at(0.0)
    .for_duration(1.0)
    .ease(Ease::OutCubic)
    .appear()
    .spawn();

// Arrow draws after text is visible
timeline
    .animate(arrow_id)
    .at(1.5)
    .for_duration(1.5)
    .ease(Ease::OutCubic)
    .draw()
    .spawn();
```

## Best Practices

### Do's
- ✅ Use `.appear()` for simple fade-ins
- ✅ Use `OutCubic` as your default easing
- ✅ Keep animation durations between 0.5 and 2.0 seconds for most effects
- ✅ Use `.call_during()` for complex motion paths
- ✅ Stagger animations for visual interest
- ✅ Hide target tattvas before morphing

### Don'ts
- ❌ Don't use `.draw()` on filled shapes (circles, squares)
- ❌ Don't make animations too fast (< 0.3 seconds) or too slow (> 3 seconds)
- ❌ Don't use `Linear` easing for organic motion
- ❌ Don't forget to call `.spawn()` at the end
- ❌ Don't animate properties that don't exist on a tattva type

## Troubleshooting

**Animation doesn't play:**
- Did you call `.spawn()` at the end?
- Did you call `scene.play(timeline)`?
- Is the start time (`.at(...)`) reasonable?

**Animation happens instantly:**
- Check `.for_duration(...)` is > 0
- Verify the timeline is actually being played

**Object doesn't appear:**
- For `.appear()`, make sure the object is staged (hidden initially)
- Check that the object is within camera view
- Verify opacity is not 0.0

**Morph looks weird:**
- Source and target shapes may be too different
- Try hiding the target before morphing
- Consider using `.fade_to()` transitions instead

## What's Next?

- **[Scene and App](./scene-and-app)** - Learn more about scene management
- **[Tattvas](./tattvas/)** - Explore all available objects to animate
- **[Camera](./camera)** - Control camera movement and framing
- **[Updaters](./updaters)** - For frame-by-frame custom logic
