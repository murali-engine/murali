---
sidebar_position: 1
---

# Stepwise Internals

This page explains how `Stepwise` is structured internally for contributors who want to understand or extend it.

For the user-facing API, see [Storytelling](../tattvas/storytelling).

## What Stepwise Is

`Stepwise` is a higher-level tattva for step-by-step explanations. Conceptually, it combines:

- a graph-like model of steps and transitions
- a layout strategy for placing nodes in world space
- a timeline-to-state computation layer
- a projection layer that renders nodes, edges, labels, and the signal dot

It is still a normal Murali tattva:

- it implements `Project`
- it implements `Bounded`
- it lives in the scene like any other frontend object

So its complexity is not in special engine treatment. Its complexity is in how much internal structure it manages inside one tattva.

From an authored API point of view, the preferred entrypoint is now `Stepwise::from_script(...)`.
That keeps the script DSL as the single authoring path while making Stepwise feel more like other Murali tattvas.

## Internal Pieces

The Stepwise implementation is split into a few focused modules:

- `model.rs`
- `layout.rs`
- `timeline.rs`
- `state.rs`
- `tattva.rs`

Each has a distinct role.

### `model.rs`

This defines the authored structure:

- `Step`
- `Transition`
- `StepwiseModel`
- `StepContent`
- `TattvaContent`

`StepwiseModel` is the semantic source of truth for the step graph. It stores:

- the steps
- the transitions
- the reveal sequence

The `StepContent` trait is the main extensibility hook. It lets a step host custom projected content instead of only default node text.

`TattvaContent<T>` is the adapter that wraps any existing `Project` implementor as step content. That is a key design choice because it lets Stepwise embed other Murali tattvas without requiring those tattvas to know anything about Stepwise.

Custom step content is stored with shared ownership, so cloning a `StepwiseModel` no longer drops embedded content. That makes the authored model safer to reuse and pass around than the earlier implementation.

### `layout.rs`

This defines `StepwiseLayout` and `StepwiseDirection`.

The layout module is intentionally simple right now. It computes:

- node positions by index
- horizontal or vertical arrangement
- interpolated positions for the signal dot

The important architectural point is that layout is deterministic and index-based. It is not a generic graph layout engine.

### `state.rs`

This defines derived runtime state:

- `StepState`
- `TransitionState`
- `StepwiseState`

This state is not authored directly by the user. It is computed from:

- the model
- the current progress value

That separation keeps the authored model stable while allowing the rendered state to evolve over time.

### `timeline.rs`

This contains `TimelineEngine::compute`.

This module is the bridge from continuous progress to discrete step/transition state. It:

- maps global progress into an active sequence segment
- computes per-step states
- computes per-transition states

In other words, this is where the reveal logic lives.

### `tattva.rs`

This contains the actual `Stepwise` tattva and its `Project` / `Bounded` implementations.

It is responsible for:

- storing runtime progress values
- storing runtime style values
- computing `StepwiseState`
- projecting node and edge visuals
- projecting signal flow visuals
- rendering default node appearance when custom content is absent

It also contains `StepwiseStyle`, which gathers the visual defaults that used to live as hardcoded constants directly inside the tattva implementation.

## State Model

The core runtime fields on `Stepwise` are:

- `model`
- `progress`
- `signal_progress`
- `layout`
- `style`
- `debug`

The most important split is between `progress` and `signal_progress`.

### `progress`

`progress` drives the build/reveal phase of the diagram.

It determines:

- which steps are pending
- which step is currently active
- which transitions are hidden, drawing, or complete

### `signal_progress`

`signal_progress` is independent from reveal progression.

It drives the signal dot along the fully or partially built route. This is a good design choice because it lets authors separate:

- "diagram is being introduced"
- "conceptual signal is flowing through it"

That makes Stepwise more expressive than a single monolithic progress value would be.

For readability, the tattva also exposes `with_reveal_progress(...)` and `with_journey_progress(...)` as clearer aliases over the original progress setters.

## Reveal Computation

The reveal logic lives in `TimelineEngine::compute`.

At a high level:

1. clamp progress into `[0, 1]`
2. identify the active segment of the reveal sequence
3. compute local progress within that segment
4. mark steps as `Pending`, `Active { t }`, or `Completed`
5. mark transitions as `Hidden`, `Drawing { t }`, or `Completed`

This means Stepwise does not store a fully materialized per-node render state between frames. It recomputes derived step/transition state from the model and current progress value each frame.

That keeps the system deterministic and easy to reason about.

## Rendering Strategy

`Stepwise` renders itself entirely through `Project`.

The main projected parts are:

- transitions
- nodes
- labels or custom content
- signal dot

### Nodes

The default node renderer uses a two-phase write-style appearance:

- outline draw
- fill and label reveal

That logic is encoded in helper functions inside `tattva.rs`, not in the engine. This is a good example of a higher-level tattva implementing its own internal animation look while still fitting inside Murali's standard projection model.

### Custom Content

If a step provides `StepContent`, Stepwise projects that content inside the node.

If the content does not draw its own background, Stepwise renders the standard container behind it. That makes custom step content composable while preserving a consistent default appearance.

### Signal Dot

Signal flow is projected independently from step reveal state. This makes signal traversal a separate visual layer instead of mixing it into step reveal computation.

## Extensibility Points

The main extension points today are:

- `StepContent` for custom node rendering
- `TattvaContent<T>` for adapting existing tattvas
- layout direction and spacing through `StepwiseLayout`
- visual customization through `StepwiseStyle`
- animation builders that target Stepwise-specific progress values

If you want to extend Stepwise without redesigning it, these are usually the safest places to do that.

## Back-Path Routing

Stepwise supports non-linear edges — transitions that go backwards or skip nodes. These are authored with `.route()` on a connection.

### How routing works

Each `Direction` step in a route moves the path by exactly one node in the layout:

- `Direction::Left` — moves one node index to the left (decrements the virtual column)
- `Direction::Right` — moves one node index to the right
- `Direction::Up` — jumps to a clear lane above all involved nodes
- `Direction::Down` — jumps to a clear lane below all involved nodes

`Left` and `Right` snap to the center X of the adjacent node in the layout sequence. Two `Left` steps means two nodes back. This is deterministic — there is no guessing or interpolation.

`Up` and `Down` compute a Y position that clears the bounding boxes of both the source and target nodes, plus a lane margin derived from the layout spacing.

### Spatial arrival

The final connection into the target node is computed spatially. After all route steps are walked, the engine looks at where the path ended up relative to the target node's bounding box:

- if the path is clearly above or below the node, it enters the top or bottom face
- if the path is clearly to the left or right, it enters the side face
- a clean Manhattan elbow is inserted to connect the last waypoint to the selected face anchor

This means the route directions describe the shape of the path, and the engine handles the final entry cleanly without requiring the author to specify the arrival face explicitly.

### Example

```rust
// D is index 3, A is index 0 — three hops back
s.connect(d, a).route(vec![
    Direction::Up,    // exit top of D, rise to clear lane
    Direction::Left,  // move to C's column
    Direction::Left,  // move to B's column
    Direction::Left,  // move to A's column
    // spatial arrival: path is above A → enters top face automatically
]);
```

The signal dot follows the same computed polyline, so it travels the routed path correctly during signal animation.

## Current Tradeoffs And Limitations

A few important limitations are visible in the current design:

- layout is simple and sequence-based, not a full graph layout engine
- custom content opacity for mesh-heavy content is best-effort rather than perfect, because some alpha is baked into projected geometry
- the default rendering style is strongly opinionated

These are not necessarily problems, but they are important to know before extending the feature.

## Good Contributor Questions

If you are changing Stepwise, useful questions to ask are:

- is this changing the authored model, the derived runtime state, or only projection?
- should this be part of `TimelineEngine::compute`, or only a rendering detail?
- does this belong in generic architecture, or only inside Stepwise?
- does this preserve deterministic recomputation from model + progress?

That framing usually helps keep Stepwise changes clean and local.
