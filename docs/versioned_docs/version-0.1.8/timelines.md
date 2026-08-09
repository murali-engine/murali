---
sidebar_position: 6
---

# Timelines and Clips

A **Timeline** is the scene's single global time axis. A **Clip** is a reusable group of animations authored from its own local time `0.0` and placed onto that timeline.

## What is a Timeline?

Think of the timeline as the editing surface for the complete animation. Clips are the sections arranged on it: an introduction, a model explanation, a background motion sequence, or an outro.

Murali has one runtime clock and one global timeline per scene. Clips do not introduce independent runtime clocks. Murali converts every clip-local start time into an absolute scene time when the clip is composed.

```text
absolute start = clip placement + clip-local start
```

**Key insight:** Timelines and clips schedule mutations; the scene holds the actual visual state.

## Creating a Timeline

```rust
use murali::{Clip, Timeline};

let mut timeline = Timeline::new();
```

Directly authored timeline times are absolute scene times. Use a clip when a section should have its own local time frame or be reusable.

```rust
let mut intro = Clip::new();

intro
    .animate(title_id)
    .at(0.0) // Local to `intro`
    .for_duration(1.0)
    .appear()
    .spawn();
```

## Adding Animations

The most common way to add animations is with the fluent builder API:

```rust
timeline
    .animate(tattva_id)           // What to animate
    .at(0.0)                      // When to start (seconds)
    .for_duration(2.0)            // How long it takes
    .ease(Ease::OutCubic)         // How it moves
    .move_to(Vec3::new(3.0, 0.0, 0.0))  // What changes
    .spawn();                     // Add to timeline
```

**Important:** Don't forget `.spawn()` at the end! Without it, the animation isn't added to the timeline.

## Timeline Lifecycle

### 1. Build Phase

You can add animations directly to the global timeline or author them inside local-time clips:

```rust
let mut timeline = Timeline::new();
let mut intro = Clip::new();
let mut explanation = Clip::new();

intro.animate(id1).at(0.0).for_duration(2.0).move_to(...).spawn();
explanation.animate(id2).at(0.0).for_duration(1.5).scale_to(...).spawn();
explanation.animate(id3).at(0.5).for_duration(1.0).fade_to(...).spawn();

timeline.append(intro);
timeline.append(explanation);
```

Both clips begin at local `0.0`. `append` places `explanation` after `intro` and flattens both schedules into absolute scene times.

### 2. Play Phase

You give the timeline to the scene:

```rust
scene.play(timeline)?;
```

Timeline start times and durations must be finite. Fluent builders reject invalid entries and
retain a structured `ValidationError`; `scene.play(timeline)?` surfaces that error instead of
installing a partial schedule. Low-level authors can use `timeline.try_add_animation(...)` when
they need the error immediately. Clip placement and `wait_until` follow the same contract.

### 3. Runtime Phase

Each frame:
1. Scene time advances by `dt` (e.g., 1/60 second)
2. The timeline checks which animations should be active
3. Active animations apply their changes to tattvas
4. Scene state is updated
5. Renderer draws the new state

## Seeking And Rewinding

Use an absolute scene time when reconstructing a timeline-authored frame:

```rust
scene.seek_to(2.5)?;
```

When an `Engine` already owns the scene and renderer, use `engine.seek_to(2.5)?` instead. It reconstructs the frontend state and immediately synchronizes the result to the rendering backend.

Before its first evaluation, a timeline captures the scene's drawable properties and camera as its baseline. Seeking restores that baseline, resets animation-specific state in reverse order, and then evaluates the authored schedule forward to the requested time. Repeated seeks to the same time therefore produce the same property and camera state. A negative `scene.update(dt)?` also uses this reconstruction path and clamps at time `0.0`.

Seeking is fail-closed. Murali returns a `SeekError` before changing scene state when reconstruction would cross a non-reversible callback, or when the scene contains frame-dependent updaters or history-dependent traced paths. A seek therefore either produces a reconstructed frame or reports why that frame cannot be reconstructed; it never silently returns a partial approximation.

## One Timeline, Many Clips

Every scene has at most one installed timeline:

```rust
let mut timeline = Timeline::new();

timeline.append(intro);
timeline.append(explanation);
timeline.overlay(background_motion);
timeline.place_at(12.0, outro);

scene.play(timeline)?;
```

The composition methods have distinct placement behavior:

- `append(clip)` places the clip at the composition cursor and advances the cursor by its duration.
- `overlay(clip)` places the clip at the start of the most recently appended composition group. If it is longer, it extends the cursor.
- `place_at(time, clip)` places the clip at an explicit absolute scene time without moving the composition cursor.
- `cursor()` returns where the next appended clip will begin.

After composition, clips are consumed. Playback evaluates a flat, deterministically ordered animation schedule.

### Migrating from Named Timelines

`Scene::play_named` and named `Scene::set_timeline` have been removed. Convert each former scheduling lane into a clip, then compose those clips onto one timeline:

```rust
let mut content = Clip::new();
let mut background = Clip::new();

// Author both clips from local time 0.0.

let mut timeline = Timeline::new();
timeline.append(content).overlay(background);
scene.play(timeline)?;
```

Use `append` when former sections should run sequentially, `overlay` when they should share a composition origin, and `place_at` when the global start time is intentional.

## Global Time vs Clip-Local Time

**Scene time** is the authoritative clock. It's a single `f32` that represents "where we are" in the animation.

Animations authored directly on a timeline use absolute scene time:

```rust
// At scene_time = 1.5:
timeline.animate(id).at(0.0).for_duration(2.0).move_to(...).spawn();
// This animation is active (started at 0.0, ends at 2.0)

timeline.animate(id).at(2.0).for_duration(1.0).scale_to(...).spawn();
// This animation hasn't started yet (starts at 2.0)
```

A clip uses local time:

```rust
let mut clip = Clip::new();
clip.animate(id).at(2.0).for_duration(1.0).appear().spawn();

timeline.place_at(10.0, clip);
// The animation starts at absolute scene time 12.0.
```

Clip-local time is an authoring reference frame, not a clock that can drift, pause, or advance independently.

## Callbacks

Sometimes you need to run custom code at specific times.

### call_at

Run a one-shot effect once at a specific time during forward playback:

```rust
timeline.call_at(2.0, |scene| {
    println!("Reached t=2.0!");
    scene.hide(some_id);
});
```

**Use cases:**
- Logging or debugging
- External notifications
- Effects that are intentionally not seekable

`call_at` is non-reversible and blocks seeking once the requested or previously played interval
crosses it. For scene mutations, provide an explicit inverse:

```rust
timeline.call_at_reversible(
    2.0,
    move |scene| scene.hide(some_id),
    move |scene| scene.show(some_id),
);
```

### call_during

Run non-seekable procedural code continuously over a duration:

```rust
timeline.call_during(1.0, 2.0, |scene, t| {
    // t goes from 0.0 to 1.0 over the duration
    // This runs every frame between scene_time 1.0 and 3.0

    let angle = t * std::f32::consts::TAU;
    let position = Vec3::new(
        angle.cos() * 3.0,
        angle.sin() * 3.0,
        0.0
    );
    scene.set_position_3d(circle_id, position);
});
```

**Use cases:**
- Complex motion paths (circles, spirals, custom curves)
- Dependent motion (one object following another)
- Procedural animations
- Custom interpolation logic

**Note:** The `t` parameter is normalized from `0.0` to `1.0`, regardless of the actual duration. Each endpoint is invoked once per forward evaluation. Use `call_during_reversible(start, duration, callback, reset)` when seeking must be supported. The reset closure restores the callback-owned state before Murali replays the normalized callback at the requested time.

## Advanced Timeline Features

### Morphing Groups

Morph multiple tattvas at once:

```rust
timeline.morph_matching_staged(
    source_ids,      // Vec<TattvaId>
    target_ids,      // Vec<TattvaId>
    &mut scene,
    1.0,             // start time
    2.0,             // duration
    Ease::InOutCubic,
);
```

This automatically stages (hides) the target tattvas and morphs them from the sources.

### Signal Playback

For procedural or signal-driven animations:

```rust
use murali::engine::timeline::SignalPlayback;

// Play once
let playback = SignalPlayback::once(0.0, 2.0, Ease::OutCubic);
timeline.play_signal(tattva_id, playback);

// Round trip (there and back)
let playback = SignalPlayback::round_trip(0.0, 2.0, Ease::InOutQuad);
timeline.play_signal(tattva_id, playback);

// Loop multiple times
let playback = SignalPlayback::looped(0.0, 1.0, 5, Ease::Linear);
timeline.play_signal(tattva_id, playback);
```

### Wait Until

Ensure the scene runs until a specific time, even if all animations finish earlier:

```rust
timeline.wait_until(10.0);
```

This is useful for adding a pause at the end of your animation before it loops or exits.

### End Time

Get when the timeline finishes:

```rust
let end = timeline.end_time();
println!("Animation ends at t={}", end);
```

This considers all scheduled animations and any `wait_until` calls.

## Sequencing Patterns

### Sequential (One After Another)

```rust
let mut timeline = Timeline::new();
let mut first = Clip::new();
let mut second = Clip::new();
let mut third = Clip::new();

first.animate(id1).at(0.0).for_duration(2.0).move_to(...).spawn();
second.animate(id2).at(0.0).for_duration(1.5).scale_to(...).spawn();
third.animate(id3).at(0.0).for_duration(1.0).appear().spawn();

timeline.append(first).append(second).append(third);
```

Each section stays locally authored from zero. Changing the first clip's duration automatically moves the clips that follow it.

### Parallel (All at Once)

```rust
let mut timeline = Timeline::new();
let mut content = Clip::new();
let mut background = Clip::new();

content.animate(id1).at(0.0).for_duration(2.0).move_to(...).spawn();
content.animate(id2).at(0.0).for_duration(2.0).scale_to(...).spawn();
background.animate(id3).at(0.0).for_duration(3.0).fade_to(...).spawn();

timeline.append(content).overlay(background);
```

### Staggered (Overlapping)

```rust
let mut timeline = Timeline::new();
let stagger_delay = 0.2;

for (i, id) in tattva_ids.iter().enumerate() {
    timeline
        .animate(*id)
        .at(i as f32 * stagger_delay)
        .for_duration(1.0)
        .ease(Ease::OutCubic)
        .appear()
        .spawn();
}
```

### Overlapping (Start Before Previous Ends)

```rust
let mut timeline = Timeline::new();

// Animation 1: 0.0 to 2.0
timeline.animate(id1).at(0.0).for_duration(2.0).move_to(...).spawn();

// Animation 2: 1.5 to 3.0 (overlaps with 1)
timeline.animate(id2).at(1.5).for_duration(1.5).scale_to(...).spawn();

// Animation 3: 2.5 to 3.5 (overlaps with 2)
timeline.animate(id3).at(2.5).for_duration(1.0).appear().spawn();
```

Animations writing different properties compose normally. When multiple core animations write the same property on the same tattva, the animation with the later start time has precedence. If their start times are equal, the one spawned later has precedence. Its terminal value remains authoritative after it finishes, so direct seeking and frame-by-frame playback produce the same result.

The scheduler evaluates every crossed animation start boundary exactly, even when that boundary falls between rendered frames. This ensures an animation captures the same starting value at every frame rate.

## Common Patterns

### Intro → Content → Outro

```rust
let mut timeline = Timeline::new();

// Intro: Title appears
timeline.animate(title_id).at(0.0).for_duration(1.0).appear().spawn();

// Content: Main animation
timeline.animate(content_id).at(1.5).for_duration(3.0).draw().spawn();

// Outro: Everything fades out
timeline.animate(title_id).at(5.0).for_duration(1.0).fade_to(0.0).spawn();
timeline.animate(content_id).at(5.0).for_duration(1.0).fade_to(0.0).spawn();
```

### Build Up Then Transform

```rust
let mut timeline = Timeline::new();

// Build: Reveal all pieces
for (i, id) in piece_ids.iter().enumerate() {
    timeline
        .animate(*id)
        .at(i as f32 * 0.3)
        .for_duration(0.8)
        .ease(Ease::OutCubic)
        .appear()
        .spawn();
}

// Transform: Move pieces into final positions
let transform_start = piece_ids.len() as f32 * 0.3 + 1.0;
for (i, id) in piece_ids.iter().enumerate() {
    timeline
        .animate(*id)
        .at(transform_start)
        .for_duration(2.0)
        .ease(Ease::InOutQuad)
        .move_to(final_positions[i])
        .spawn();
}
```

### Synchronized Multi-Property Animation

```rust
let mut timeline = Timeline::new();

// Move and scale at the same time
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

// Fade out while moving back
timeline
    .animate(id)
    .at(3.0)
    .for_duration(1.5)
    .ease(Ease::InCubic)
    .move_to(Vec3::ZERO)
    .spawn();

timeline
    .animate(id)
    .at(3.0)
    .for_duration(1.5)
    .ease(Ease::InCubic)
    .fade_to(0.0)
    .spawn();
```

## Timeline Best Practices

### Do's

✅ **Use clips for independently authored sections**
```rust
let mut explanation = Clip::new();
explanation.animate(id).at(0.0).for_duration(2.0).draw().spawn();

timeline.append(explanation);
```

✅ **Use descriptive timing constants**
```rust
const INTRO_START: f32 = 0.0;
const INTRO_DURATION: f32 = 1.5;
const CONTENT_START: f32 = INTRO_START + INTRO_DURATION + 0.5;
const CONTENT_DURATION: f32 = 3.0;
```

✅ **Group related animations**
```rust
// Title animations
timeline.animate(title_id).at(0.0).for_duration(1.0).appear().spawn();
timeline.animate(title_id).at(5.0).for_duration(1.0).fade_to(0.0).spawn();

// Content animations
timeline.animate(content_id).at(1.5).for_duration(2.0).draw().spawn();
timeline.animate(content_id).at(5.0).for_duration(1.0).undraw().spawn();
```

✅ **Use staggering for visual interest**
```rust
for (i, id) in ids.iter().enumerate() {
    timeline
        .animate(*id)
        .at(i as f32 * 0.2)
        .for_duration(1.0)
        .appear()
        .spawn();
}
```

✅ **Add pauses between sections**
```rust
// Section 1: 0.0 to 3.0
// Pause: 3.0 to 3.5
// Section 2: 3.5 to 6.0
```

### Don'ts

❌ **Don't forget `.spawn()`**
```rust
// This does nothing!
timeline.animate(id).at(0.0).for_duration(2.0).move_to(...);
// Missing .spawn()
```

❌ **Don't encode section placement into every animation**
```rust
// Bad
timeline.animate(id1).at(12.0).for_duration(1.0).appear().spawn();
timeline.animate(id2).at(13.5).for_duration(2.0).draw().spawn();

// Better: author the section from local zero, then place it once.
let mut section = Clip::new();
section.animate(id1).at(0.0).for_duration(1.0).appear().spawn();
section.animate(id2).at(1.5).for_duration(2.0).draw().spawn();
timeline.place_at(12.0, section);
```

❌ **Don't make timings too tight**
```rust
// Bad - no breathing room
timeline.animate(id1).at(0.0).for_duration(1.0).appear().spawn();
timeline.animate(id2).at(1.0).for_duration(1.0).appear().spawn();

// Better - add small gaps
timeline.animate(id1).at(0.0).for_duration(1.0).appear().spawn();
timeline.animate(id2).at(1.3).for_duration(1.0).appear().spawn();
```

❌ **Don't try to control timeline playback speed**
```rust
// This doesn't exist (yet)
// timeline.set_speed(0.5);  // ❌ Not supported
```

## Debugging Timelines

### Print Timeline Info

```rust
let end_time = timeline.end_time();
println!("Timeline ends at: {:.2}s", end_time);
```

### Add Debug Callbacks

```rust
timeline.call_at(1.0, |_scene| {
    println!("Checkpoint 1 reached");
});

timeline.call_at(2.5, |_scene| {
    println!("Checkpoint 2 reached");
});

timeline.call_at(5.0, |_scene| {
    println!("Animation complete");
});
```

### Visualize Timing

```rust
// Print a simple timeline visualization
println!("Timeline:");
println!("0.0s: Title appears");
println!("1.5s: Content draws");
println!("3.0s: Transform begins");
println!("5.0s: Fade out");
println!("6.0s: End");
```

## Troubleshooting

**Animations don't play:**
- Did you call `scene.play(timeline)`?
- Did you forget `.spawn()` at the end of animations?
- Are start times reasonable (not negative, not too large)?

**Animations happen in wrong order:**
- Check your `.at(time)` values
- Confirm whether `.at(time)` is global on a `Timeline` or local on a `Clip`
- Print `timeline.cursor()` to inspect where the next clip will be appended
- Make sure you're not reusing the same time for sequential animations
- Print `timeline.end_time()` to verify total duration

**Animation feels wrong:**
- Try different easing functions
- Adjust duration (too fast or too slow?)
- Add small delays between animations for breathing room

**A clip starts at the wrong time:**
- Remember that clip timestamps start from local `0.0`
- Use `append` for sequential placement
- Use `overlay` to share the previous append origin
- Use `place_at` for an explicit absolute placement

## What's Next?

- **[Animations](./animations)** - Learn all available animation verbs
- **[Scene and App](./scene-and-app)** - Understand scene management
- **[Updaters](./updaters)** - For frame-by-frame custom logic
- **[Camera](./camera)** - Animate the camera
