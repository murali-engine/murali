---
sidebar_position: 7
---

# Which API Should I Use?

Murali offers multiple ways to accomplish similar tasks. This guide helps you choose the right API for your situation.

As a general rule, prefer the highest-level authored API that clearly expresses your intent. Reach for lower-level APIs only when the authored surface stops being expressive enough.

## Timeline vs Clip

### Author Directly on `Timeline`

**When:** The scene is small or its timestamps naturally belong to the global scene clock.

```rust
let mut timeline = Timeline::new();
// ... add animations
scene.play(timeline)?;
```

**Why:** This is the shortest path for simple scenes. `.at(...)` values on a timeline are absolute scene times.

### Author a `Clip`

**When:** A section should start from local `0.0`, be rearranged, overlap another section, or be reusable.

```rust
let mut intro = Clip::new();
intro.animate(title_id).at(0.0).for_duration(1.0).appear().spawn();

let mut explanation = Clip::new();
explanation.animate(graph_id).at(0.0).for_duration(3.0).draw().spawn();

let mut timeline = Timeline::new();
timeline.append(intro).append(explanation);
scene.play(timeline)?;
```

**Why:** Clip-local timestamps stay stable when earlier sections change duration or the clip moves elsewhere on the global timeline.

### Choose a Placement Method

- `append(clip)` for one section after another
- `overlay(clip)` for a clip sharing the most recent append origin
- `place_at(time, clip)` for explicit absolute placement

Every scene plays at most one timeline through `scene.play(timeline)`. Clips are flattened into that timeline before playback; they are not independent clocks.

## Group vs Clip vs SceneView

- Use a `Group` when several tattvas only need to move or lay out together.
- Use a `Clip` when several animations need reusable local timestamps on the same scene clock.
- Use a `SceneView` when the content needs its own scene, timeline, camera, updaters, playback
  speed, or loop.

```rust
let view_id = parent.add_scene_view(
    SceneView::new(child)
        .playback(SceneViewPlayback::Loop { duration: 3.0 }),
    Vec3::ZERO,
);
```

The returned ID makes the complete child scene behave like one parent object for movement, scale,
rotation, opacity, layout, and layering. See [SceneView](./scene-views) for construction, styling,
and playback details.

For the beta [`Opening`](./tattvas/opening) composite:

- Add it directly to a `Scene` when the opening and continuation share one camera and projection.
- Put it in a full-frame `SceneView` when a perspective opening must reveal orthographic content,
  when its background must fade with it, or when the complete opening needs one parent transform.

The `opening_scene_view` example demonstrates the second form end to end.

---

## Adding Tattvas: add_tattva() vs add()

### Use `scene.add_tattva(state, position)`

**When:** Adding any visual object (99% of cases)

```rust
let id = scene.add_tattva(
    Circle::new(1.0, 48, color),
    Vec3::new(0.0, 0.0, 0.0),
);
```

**Why:** Type-safe, clear intent, handles wrapping automatically.

### Use `scene.add(tattva)`

**When:** Working with already-wrapped tattvas or internal systems

```rust
let tattva = state.into_tattva();
let id = scene.add(tattva);
```

**Why:** Lower-level API for advanced scenarios.

**Recommendation:** Always use `add_tattva()` in application code.

---

## Modifying Tattvas: Intent Helpers vs Typed Access

### Use Intent Helpers

**When:** Changing common properties (position, scale, opacity, visibility)

```rust
scene.set_position_2d(id, Vec2::new(1.0, 2.0));  // For 2D scenes
scene.set_position_3d(id, Vec3::new(1.0, 2.0, 0.0));  // For 3D scenes
scene.set_scale(id, Vec3::splat(2.0));
scene.set_opacity(id, 0.5);
scene.hide(id);
scene.show(id);
```

**Why:** Clear, works on any tattva type, easy to read.

### Use Typed Access

**When:** Changing type-specific properties

```rust
if let Some(circle) = scene.get_tattva_typed_mut::<Circle>(id) {
    circle.state.radius = 2.0;
    circle.state.segments = 64;
}
```

**Why:** Access to type-specific state when the shared scene helpers are not enough. Successful
mutable access automatically marks the tattva for backend reconstruction.

**Recommendation:** Prefer intent helpers when possible, use typed access for type-specific needs.

---

## Visibility: hide() vs set_opacity(0.0) vs fade_to(0.0)

### Use `scene.hide(id)`

**When:** You want to immediately hide something (no animation)

```rust
scene.hide(id);
```

**Why:** Instant, clear intent, no animation overhead.

### Use `scene.set_opacity(id, 0.0)`

**When:** You want to set opacity immediately (no animation) through the shared styling surface

```rust
scene.set_opacity(id, 0.0);
```

**Why:** This is the low-level opacity control. In most authored code, `scene.hide(id)` is the clearer choice when you mean "make this disappear now."

### Use `.fade_to(0.0)` animation

**When:** You want to animate the fade out

```rust
timeline
    .animate(id)
    .at(2.0)
    .for_duration(1.0)
    .ease(Ease::InCubic)
    .fade_to(0.0)
    .spawn();
```

**Why:** Smooth animated transition.

**Recommendation:**
- Staging objects before reveal → `scene.hide(id)`
- Instant hide → `scene.hide(id)`
- Immediate opacity adjustment → `scene.set_opacity(id, value)`
- Animated fade → `.fade_to(0.0)`

---

## Appearance: appear() vs fade_to(1.0) vs show()

### Use `.appear()` animation

**When:** Animating an object from hidden to visible

```rust
scene.hide(id);  // Stage it first

timeline
    .animate(id)
    .at(0.0)
    .for_duration(1.0)
    .ease(Ease::OutCubic)
    .appear()
    .spawn();
```

**Why:** Clear intent, handles staging automatically if object was hidden.

### Use `.fade_to(1.0)` animation

**When:** Animating opacity to full (object is already partially visible)

```rust
timeline
    .animate(id)
    .at(0.0)
    .for_duration(1.0)
    .ease(Ease::OutCubic)
    .fade_to(1.0)
    .spawn();
```

**Why:** Explicit about target opacity value.

### Use `scene.show(id)`

**When:** Immediately showing a hidden object (no animation)

```rust
scene.show(id);
```

**Why:** Instant, no animation overhead.

**Recommendation:**
- Animated reveal → `.appear()`
- Instant show → `scene.show(id)`
- Specific opacity target → `.fade_to(value)`

---

## Text: Label vs Typst vs Latex vs CodeBlock

### Use `Label`

**When:** Simple text, titles, labels, UI text

```rust
Label::new("Hello World", 0.32)
```

**Why:** Fast, simple, no external dependencies, good for most text.

**Limitations:** Basic styling only, no math, no complex formatting.

### Use `Typst`

**When:** Rich text, formatted documents, modern typesetting

```rust
Typst::new("*Bold* and _italic_ text", 0.5)  // source, world_height
```

**Why:** Modern, fast, good formatting, easier than LaTeX.

**Limitations:** Newer ecosystem, fewer legacy packages than LaTeX.

### Use `Latex`

**When:** Mathematical equations, academic content, complex formulas

```rust
Latex::new(r"\frac{a}{b} + \sqrt{c}", 0.5)  // source, world_height
```

**Why:** Industry standard for math, extensive symbol support.

**Limitations:** Requires LaTeX installation, slower rendering, complex syntax.

### Use `CodeBlock`

**When:** Displaying code with syntax highlighting

```rust
CodeBlock::new("fn main() { println!(\"Hello\"); }", "rust", 0.5)  // code, language, world_height
```

**Why:** Syntax highlighting, monospace font, code-specific formatting.

**Limitations:** Not for prose or math.

**Recommendation:**
- Titles/labels → `Label`
- Math → `Latex`
- Rich text → `Typst`
- Code → `CodeBlock`

---

## Drawing Paths: draw() vs write() vs appear()

### Use `.draw()` animation

**When:** Progressively revealing a path, line, or stroke

```rust
timeline
    .animate(line_id)
    .at(0.0)
    .for_duration(2.0)
    .ease(Ease::OutCubic)
    .draw()
    .spawn();
```

**Why:** Natural "drawing" effect for paths.

**Works on:** Lines, arrows, paths, curves, vector graphics.

### Use `.write()` animation (legacy)

**When:** Same as `.draw()` (legacy name)

```rust
timeline.animate(line_id).at(0.0).for_duration(2.0).write().spawn();
```

**Why:** Older name, still works but `.draw()` is preferred.

### Use `.appear()` animation

**When:** Fading in any object (including paths)

```rust
timeline
    .animate(circle_id)
    .at(0.0)
    .for_duration(1.0)
    .ease(Ease::OutCubic)
    .appear()
    .spawn();
```

**Why:** Simple fade-in effect.

**Recommendation:**
- Path-like objects (lines, arrows) → `.draw()`
- Filled shapes (circles, squares) → `.appear()`
- Legacy code → `.write()` works but migrate to `.draw()`

---

## Text Animation: typewrite_text() vs reveal_text() vs appear()

### Use `.typewrite_text()`

**When:** Character-by-character reveal, typing effect

```rust
timeline
    .animate(label_id)
    .at(0.0)
    .for_duration(2.0)
    .ease(Ease::Linear)
    .typewrite_text()
    .spawn();
```

**Why:** Mimics typing, good for code, terminal output, dramatic reveals.

**Best for:** Code snippets, terminal text, dramatic emphasis.

### Use `.reveal_text()`

**When:** Centered shifting reveal effect

```rust
timeline
    .animate(title_id)
    .at(0.0)
    .for_duration(1.5)
    .ease(Ease::OutCubic)
    .reveal_text()
    .spawn();
```

**Why:** Elegant reveal with motion, good for titles.

**Best for:** Titles, headings, emphasis text.

### Use `.appear()`

**When:** Simple fade-in

```rust
timeline
    .animate(label_id)
    .at(0.0)
    .for_duration(1.0)
    .ease(Ease::OutCubic)
    .appear()
    .spawn();
```

**Why:** Clean, simple, works for any text.

**Best for:** Body text, labels, when you want subtle appearance.

**Recommendation:**
- Dramatic/typing effect → `.typewrite_text()`
- Elegant title reveal → `.reveal_text()`
- Simple fade → `.appear()`

---

## Layout: Scene Helpers vs Manual Positioning

### Use Scene Helpers

**When:** Common layout tasks (edges, alignment, relative positioning)

```rust
scene.to_edge(id, Direction::Up, 0.35);
scene.next_to(label_id, circle_id, Direction::Right, 0.5);
scene.align_to(id1, id2, Anchor::Center);
```

**Why:** Clear intent, handles calculations for you, easier to read.

### Use Manual Positioning

**When:** Precise control, custom layouts, mathematical positioning

```rust
scene.set_position_2d(id, Vec2::new(2.5, 1.3));  // For 2D scenes
scene.set_position_3d(id, Vec3::new(2.5, 1.3, 0.0));  // For 3D scenes
```

**Why:** Full control, exact coordinates.

**Recommendation:**
- Common layouts → Scene helpers
- Precise positioning → Manual
- Grid layouts → Manual with loops
- Edge alignment → `to_edge()`

---

## Camera: Orthographic vs Perspective

### Use Orthographic (Default)

**When:** 2D scenes, mathematical diagrams, UI-like content

```rust
scene.camera_mut().set_view_width(16.0);
```

**Why:** No perspective distortion, parallel lines stay parallel, easier to reason about.

**Best for:** Most Murali scenes, 2D content, diagrams.

### Use Perspective

**When:** 3D scenes, depth perception needed

```rust
scene.camera_mut().set_perspective(60.0);  // FOV in degrees
```

**Why:** Realistic 3D depth, objects get smaller with distance.

**Best for:** 3D visualizations, architectural views, realistic scenes.

**Recommendation:** Start with orthographic (default), switch to perspective only for 3D scenes.

---

## Callbacks: call_at() vs call_during() vs Animation Verbs

### Use Animation Verbs

**When:** Changing standard properties (position, scale, opacity)

```rust
timeline.animate(id).at(0.0).for_duration(2.0).move_to(...).spawn();
```

**Why:** Declarative, handles easing, cleaner code.

### Use `call_at()`

**When:** Discrete events, state changes, one-time actions

```rust
timeline.call_at(2.0, |scene| {
    scene.hide(id);
    println!("Checkpoint reached");
});
```

**Why:** Runs once at specific time, good for events.

### Use `call_during()`

**When:** Complex motion paths, custom interpolation, dependent motion

```rust
timeline.call_during(1.0, 2.0, |scene, t| {
    let angle = t * std::f32::consts::TAU;
    let pos = Vec3::new(angle.cos() * 3.0, angle.sin() * 3.0, 0.0);
    scene.set_position_3d(id, pos);
});
```

**Why:** Full control over motion, runs every frame.

**Recommendation:**
- Standard animations → Animation verbs
- One-time events → `call_at()`
- Custom motion → `call_during()`

---

## Quick Reference Table

| Task | Recommended API | Alternative |
|---|---|---|
| Add object | `scene.add_tattva()` | `scene.add()` (advanced) |
| Install scene timing | `scene.play(timeline)` | - |
| Sequential sections | `timeline.append(clip)` | Absolute timeline times |
| Concurrent sections | `timeline.overlay(clip)` | Same local start times in one clip |
| Explicit global placement | `timeline.place_at(time, clip)` | Direct timeline `.at(time)` |
| Shared transform for several tattvas | `Group` | Animate IDs individually |
| Independent child scene and clock | `SceneView` | - |
| Reusable 3D title ident | Beta `Opening` | Hand-authored letters and particles |
| Perspective ident into orthographic content | `Opening` inside `SceneView` | Instant parent projection switch |
| Change position (2D) | `scene.set_position_2d()` | Typed access |
| Change position (3D) | `scene.set_position_3d()` | Typed access |
| Hide instantly | `scene.hide()` | `scene.set_opacity(0.0)` |
| Fade out | `.fade_to(0.0)` | `scene.set_opacity()` |
| Reveal object | `.appear()` | `.fade_to(1.0)` |
| Simple text | `Label` | `Typst` |
| Math text | `Latex` | `Typst` |
| Draw path | `.draw()` | `.write()` (legacy) |
| Type text | `.typewrite_text()` | `.appear()` |
| Edge layout | `scene.to_edge()` | Manual position |
| 2D scene | Orthographic (default) | - |
| 3D scene | Perspective | - |
| Simple animation | Animation verb | `call_during()` |
| Custom motion | `call_during()` | Animation verb |

## General Principles

1. **Prefer high-level authored APIs** - They are clearer and closer to how Murali wants scenes to be written
2. **Use intent helpers first** - `scene.hide()` is clearer than `scene.set_opacity(0.0)`
3. **Use local time for sections** - Prefer clips when content may move or be reordered
4. **Match the tool to the task** - Don't use `.draw()` on filled shapes
5. **Be explicit** - `.appear()` is clearer than `.fade_to(1.0)` for reveals

## When in Doubt

- **For scenes:** Use `scene.play(timeline)`
- **For reusable sections:** Author a `Clip` from local `0.0`
- **For independently running content:** Add a `SceneView`
- **For the beta 3D ident:** Build an `Opening` into a `Clip`
- **For an opening with an independent projection:** Put its scene in a full-frame `SceneView`
- **For tattvas:** Use `scene.add_tattva()`
- **For visibility:** Use `scene.hide()` / `scene.show()`
- **For text:** Use `Label` first, `Latex` for math
- **For animations:** Use animation verbs, not callbacks
- **For layout:** Use scene helpers when available

## What's Next?

- **[Mental Model](./mental-model)** - Understand core concepts
- **[Animations](./animations)** - Learn all animation verbs
- **[Scene and App](./scene-and-app)** - Deep dive into scene management
- **[SceneView](./scene-views)** - Compose independently animated child scenes
- **[Opening (Beta)](./tattvas/opening)** - Configure and compose the reusable 3D opening
- **[Tattvas](./tattvas/)** - Explore all available objects
