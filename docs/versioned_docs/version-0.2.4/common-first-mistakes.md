---
sidebar_position: 8
---

# Common First Mistakes

Learning Murali is straightforward, but there are a few common pitfalls that trip up newcomers. This guide helps you avoid them.

## 1. Forgetting `.spawn()`

### The Mistake

```rust
// This does NOTHING!
timeline
    .animate(circle_id)
    .at(0.0)
    .for_duration(2.0)
    .move_to(Vec3::new(3.0, 0.0, 0.0));
// Missing .spawn()
```

### Why It Happens

The animation builder uses a fluent API. Without `.spawn()`, the animation is built but never added to the timeline.

### The Fix

```rust
timeline
    .animate(circle_id)
    .at(0.0)
    .for_duration(2.0)
    .move_to(Vec3::new(3.0, 0.0, 0.0))
    .spawn();  // ✅ Don't forget this!
```

### How to Remember

Think of `.spawn()` as "commit this animation to the timeline." Every animation needs it.

:::tip Learn More
See the [Animations](./animations) guide for all available animation verbs and their usage.
:::

---

## 2. World Space vs Pixels

### The Mistake

```rust
// Trying to use pixel coordinates
let circle = Circle::new(100.0, 48, color);  // ❌ 100 pixels?
scene.add_tattva(circle, Vec3::new(640.0, 360.0, 0.0));  // ❌ Screen center?
```

### Why It Happens

Coming from other tools that use pixel coordinates.

### The Fix

```rust
// Use world units
let circle = Circle::new(1.5, 48, color);  // ✅ 1.5 world units
scene.add_tattva(circle, Vec3::new(0.0, 0.0, 0.0));  // ✅ World origin

// Set camera view width to control scale
scene.camera_mut().set_view_width(16.0);  // 16 units fit horizontally
```

### Key Insight

- **World space** = Mathematical units (1.0 means 1.0)
- **Pixels** = Screen resolution (handled by camera)
- The camera maps world space to pixels

**Rule of thumb:** If your numbers are in the hundreds or thousands, you're probably thinking in pixels.

:::tip Learn More
See [Coordinate System](./coordinate-system) for a complete explanation of world space.
:::

---

## 3. Camera Position and Visibility

### The Mistake

```rust
let mut scene = Scene::new();

// Add a circle at origin
let circle_id = scene.add_tattva(
    Circle::new(1.0, 48, color),
    Vec3::ZERO,
);

// Forgot to position camera!
// Default camera is at origin, can't see anything
```

### Why It Happens

Forgetting that the camera needs to be positioned to see objects.

### The Fix

```rust
let mut scene = Scene::new();

let circle_id = scene.add_tattva(
    Circle::new(1.0, 48, color),
    Vec3::ZERO,
);

// Position camera away from objects
scene.camera_mut().position = Vec3::new(0.0, 0.0, 10.0);  // ✅ 10 units back
scene.camera_mut().set_view_width(16.0);
```

### Standard 2D Setup

```rust
// This works for most 2D scenes
scene.camera_mut().position = Vec3::new(0.0, 0.0, 10.0);
scene.camera_mut().set_view_width(16.0);
```

:::tip Learn More
See the [Camera](./camera) guide for advanced camera control and 3D scenes.
:::

---

## 4. Opacity vs Visibility Confusion

### The Mistake

```rust
// Trying to hide an object
scene.set_opacity(id, 0.0);  // ❌ Still in render pipeline

// Or trying to fade in without staging
timeline.animate(id).at(0.0).for_duration(1.0).appear().spawn();
// ❌ Object is already visible, appear() does nothing
```

### Why It Happens

Confusion between opacity (transparency) and visibility (render state).

### The Fix

```rust
// To hide instantly
scene.hide(id);  // ✅ Removes from render pipeline

// To fade in, stage first
scene.hide(id);  // Stage it
timeline
    .animate(id)
    .at(0.0)
    .for_duration(1.0)
    .appear()  // ✅ Now it fades in
    .spawn();
```

### The Difference

- **Opacity** = How transparent (0.0 to 1.0)
- **Visibility** = Whether it's rendered at all (hidden/shown)
- **Staging** = Hiding before reveal animation

---

## 5. Using `.draw()` on Filled Shapes

### The Mistake

```rust
let circle_id = scene.add_tattva(
    Circle::new(1.0, 48, color),
    Vec3::ZERO,
);

// Trying to "draw" a filled circle
timeline
    .animate(circle_id)
    .at(0.0)
    .for_duration(2.0)
    .draw()  // ❌ Circles don't have a natural "draw" progression
    .spawn();
```

### Why It Happens

Thinking all objects can be "drawn" progressively.

### The Fix

```rust
// For filled shapes, use appear()
timeline
    .animate(circle_id)
    .at(0.0)
    .for_duration(1.0)
    .appear()  // ✅ Fade in
    .spawn();

// .draw() is for paths, lines, arrows
let line_id = scene.add_tattva(Line::new(...), Vec3::ZERO);
timeline
    .animate(line_id)
    .at(0.0)
    .for_duration(2.0)
    .draw()  // ✅ Lines can be drawn
    .spawn();
```

### When to Use What

- **`.draw()`** → Lines, arrows, paths, curves
- **`.appear()`** → Circles, squares, filled shapes, text

---

## 6. Not Calling `scene.play(timeline)`

### The Mistake

```rust
let mut scene = Scene::new();
let mut timeline = Timeline::new();

// Add tattvas...
// Add animations to timeline...

// Forgot to play the timeline!
App::new()?.with_scene(scene).run_app()  // ❌ Nothing animates
```

### Why It Happens

Building the timeline but forgetting to attach it to the scene.

### The Fix

```rust
let mut scene = Scene::new();
let mut timeline = Timeline::new();

// Add tattvas...
// Add animations to timeline...

scene.play(timeline)?;  // ✅ Don't forget this!

App::new()?.with_scene(scene).run_app()
```

---

## 7. Animation Duration = 0

### The Mistake

```rust
timeline
    .animate(id)
    .at(0.0)
    .for_duration(0.0)  // ❌ Instant, no animation
    .move_to(Vec3::new(3.0, 0.0, 0.0))
    .spawn();
```

### Why It Happens

Typo or misunderstanding of duration.

### The Fix

```rust
timeline
    .animate(id)
    .at(0.0)
    .for_duration(2.0)  // ✅ Takes 2 seconds
    .move_to(Vec3::new(3.0, 0.0, 0.0))
    .spawn();
```

### Typical Durations

- **0.5-1.0s** → Quick transitions
- **1.0-2.0s** → Standard animations
- **2.0-3.0s** → Slow, dramatic effects

---

## 8. Animating Non-Existent Tattvas

### The Mistake

```rust
let circle_id = scene.add_tattva(Circle::new(1.0, 48, color), Vec3::ZERO);

// Typo in variable name
timeline
    .animate(circl_id)  // ❌ Wrong variable
    .at(0.0)
    .for_duration(2.0)
    .move_to(Vec3::new(3.0, 0.0, 0.0))
    .spawn();
```

### Why It Happens

Typos, copy-paste errors, or using IDs from removed tattvas.

### The Fix

```rust
// Save IDs in clear variable names
let circle_id = scene.add_tattva(Circle::new(1.0, 48, color), Vec3::ZERO);
let square_id = scene.add_tattva(Square::new(1.2, color), Vec3::new(2.0, 0.0, 0.0));

// Use the correct ID
timeline
    .animate(circle_id)  // ✅ Correct variable
    .at(0.0)
    .for_duration(2.0)
    .move_to(Vec3::new(3.0, 0.0, 0.0))
    .spawn();
```

---

## 9. Expecting Animations to Stack

### The Mistake

```rust
// Trying to move an object twice
timeline
    .animate(id)
    .at(0.0)
    .for_duration(2.0)
    .move_to(Vec3::new(2.0, 0.0, 0.0))
    .spawn();

timeline
    .animate(id)
    .at(0.0)  // ❌ Same time, conflicts!
    .for_duration(2.0)
    .move_to(Vec3::new(0.0, 2.0, 0.0))
    .spawn();
```

### Why It Happens

Expecting animations to combine or add together.

### What Actually Happens

The last animation to apply wins. Animations don't stack—they overwrite.

### The Fix

```rust
// Sequential animations
timeline
    .animate(id)
    .at(0.0)
    .for_duration(2.0)
    .move_to(Vec3::new(2.0, 0.0, 0.0))
    .spawn();

timeline
    .animate(id)
    .at(2.0)  // ✅ Starts after first ends
    .for_duration(2.0)
    .move_to(Vec3::new(0.0, 2.0, 0.0))
    .spawn();

// Or animate different properties
timeline
    .animate(id)
    .at(0.0)
    .for_duration(2.0)
    .move_to(Vec3::new(2.0, 0.0, 0.0))  // Position
    .spawn();

timeline
    .animate(id)
    .at(0.0)
    .for_duration(2.0)
    .scale_to(Vec3::splat(2.0))  // Scale (different property)
    .spawn();
```

---

## 10. Colors with Alpha = 0

### The Mistake

```rust
// Creating an invisible object
let circle = Circle::new(
    1.0,
    48,
    Vec4::new(1.0, 0.0, 0.0, 0.0)  // ❌ Alpha = 0, invisible!
);
```

### Why It Happens

Forgetting that the fourth component is alpha (transparency).

### The Fix

```rust
// Fully opaque color
let circle = Circle::new(
    1.0,
    48,
    Vec4::new(1.0, 0.0, 0.0, 1.0)  // ✅ Alpha = 1.0, fully visible
);
```

### Color Format

```rust
Vec4::new(r, g, b, a)
// r, g, b = 0.0 to 1.0 (color channels)
// a = 0.0 to 1.0 (alpha: 0.0 = transparent, 1.0 = opaque)
```

---

## 11. Running in Debug Mode

### The Mistake

```bash
# Running without --release
cargo run --example my_scene
```

### Why It Happens

Not knowing that debug builds are much slower.

### The Fix

```bash
# Always use --release for preview
cargo run --example my_scene --release
```

### Performance Difference

- **Debug mode** → 5-10 FPS (unusable)
- **Release mode** → 60+ FPS (smooth)

**Rule:** Always use `--release` when previewing or exporting.

---

## 12. Morphing Without Hiding Target

### The Mistake

```rust
// Both shapes are visible
let square_id = scene.add_tattva(Square::new(1.0, color), Vec3::new(-2.0, 0.0, 0.0));
let circle_id = scene.add_tattva(Circle::new(1.0, 48, color), Vec3::new(2.0, 0.0, 0.0));

// Morph square into circle
timeline
    .animate(circle_id)
    .at(0.0)
    .for_duration(2.0)
    .morph_from(square_id)
    .spawn();
// ❌ Circle is visible at final position, looks wrong
```

### Why It Happens

Not understanding that morph reveals the target.

### The Fix

```rust
let square_id = scene.add_tattva(Square::new(1.0, color), Vec3::new(-2.0, 0.0, 0.0));
let circle_id = scene.add_tattva(Circle::new(1.0, 48, color), Vec3::new(2.0, 0.0, 0.0));

// Hide the target first
scene.hide(circle_id);  // ✅ Stage it

timeline
    .animate(circle_id)
    .at(0.0)
    .for_duration(2.0)
    .morph_from(square_id)
    .spawn();
```

---

## 13. Confusing Preview and Export Flags

### The Mistake

```bash
# Expecting preview but getting export
cargo run --example my_scene --release
# (Default is export mode)
```

### Why It Happens

Not understanding the default behavior.

### The Fix

```bash
# For preview (interactive window)
cargo run --example my_scene --release -- --preview

# For export (video)
cargo run --example my_scene --release -- --export
```

### Remember

- **No flag** → Export (default)
- **`--preview`** → Interactive window
- **`--export`** → Explicit export

---

## 14. Expecting Frame-Based Animation

### The Mistake

```rust
// Thinking in frames
timeline
    .animate(id)
    .at(60.0)  // ❌ Thinking "frame 60"
    .for_duration(120.0)  // ❌ Thinking "120 frames"
    .move_to(Vec3::new(3.0, 0.0, 0.0))
    .spawn();
```

### Why It Happens

Coming from frame-based animation tools.

### The Fix

```rust
// Think in seconds
timeline
    .animate(id)
    .at(1.0)  // ✅ 1 second
    .for_duration(2.0)  // ✅ 2 seconds
    .move_to(Vec3::new(3.0, 0.0, 0.0))
    .spawn();
```

### Key Insight

Murali is **time-based**, not frame-based:
- `.at(1.0)` = 1 second, not frame 1
- `.for_duration(2.0)` = 2 seconds, not 2 frames

---

## 15. Not Checking Tattva Types

### The Mistake

```rust
// Trying to access Circle properties on a Square
if let Some(circle) = scene.get_tattva_typed_mut::<Circle>(square_id) {
    circle.radius = 2.0;  // ❌ square_id is not a Circle!
}
```

### Why It Happens

Mixing up IDs or not tracking tattva types.

### The Fix

```rust
// Keep track of what type each ID is
let circle_id = scene.add_tattva(Circle::new(1.0, 48, color), Vec3::ZERO);
let square_id = scene.add_tattva(Square::new(1.2, color), Vec3::new(2.0, 0.0, 0.0));

// Use the correct type
if let Some(circle) = scene.get_tattva_typed_mut::<Circle>(circle_id) {
    circle.radius = 2.0;  // ✅ Correct type
}
```

---

## Quick Checklist

Before running your scene, check:

- [ ] All animations end with `.spawn()`
- [ ] Camera is positioned (usually `Vec3::new(0.0, 0.0, 10.0)`)
- [ ] Using world units, not pixels
- [ ] Called `scene.play(timeline)`
- [ ] Animation durations > 0
- [ ] Colors have alpha = 1.0 (unless intentionally transparent)
- [ ] Running with `--release` flag
- [ ] Using `--preview` flag for interactive window
- [ ] Staged objects (hidden) before `.appear()` animations
- [ ] Using `.draw()` only on paths/lines, not filled shapes

## What's Next?

- **[Mental Model](./mental-model)** - Understand core concepts
- **[Which API Should I Use?](./which-api-should-i-use)** - Choose the right API
- **[First Scene](./first-scene)** - Step-by-step tutorial
- **[Animations](./animations)** - Learn all animation verbs
