---
sidebar_position: 2
---

# Scene & Timeline

This page explains how Murali advances authored scene state over time.

If the architecture page explains the system boundaries, this page explains the progression model inside the frontend: how `Scene`, `Timeline`, animations, updaters, props, and tattva state work together from frame to frame.

## Scene As The Authoritative Runtime Model

The `Scene` is the authoritative frontend container for what currently exists and what state it is in.

It owns:

- all active tattvas
- the global timeline
- scene time
- the camera
- updaters

This means the scene is where the current truth of the animation lives. The backend is only a derived runtime view of that truth.

In practical terms, the scene answers questions like:

- which tattvas exist right now?
- what is their current semantic state?
- what are their current transforms and visibility values?
- which animations are influencing them at this time?

## What `scene.update(dt)` Does

The engine advances Murali by calling:

```rust
scene.update(dt)?
```

That call is the center of frontend progression.

At a high level, `Scene::update` does this:

1. increment `scene_time`
2. update the global timeline against the new time
3. update derived utility systems such as traced paths
4. run updaters

So `dt` does not directly "move objects." Instead, it moves time forward, and time progression gives the timeline and updaters a chance to mutate scene state.

## Timeline As Time Scheduler

A timeline owns a list of scheduled animations. Each scheduled animation has:

- a start time
- a duration
- an animation object
- runtime status such as pending, running, or done

When the timeline is updated, it compares each scheduled animation against the current `scene_time`.

That determines whether the animation should:

- remain pending
- start now
- continue running
- finish

The timeline does not decide what the visual result is. It decides when an animation should be given control.

Murali installs at most one timeline on a scene. Clips provide local authoring frames, but composition flattens them into the timeline's absolute schedule before runtime. Consequently, scene update evaluates one deterministic schedule against one `scene_time` clock.

## Animation Lifecycle

Each animation participates in a small lifecycle:

- `reset(scene)`
- `on_start(scene)`
- `apply_at(scene, t)`
- `on_finish(scene)`

### `reset`

`reset` establishes the baseline state for the animation before its start time.

This matters because Murali is time-driven and supports seeking. The engine cannot assume that the scene has been manually prepared to match time zero.

### `on_start`

`on_start` is for one-time setup when the animation becomes active. This is often where temporary helper objects or cached starting values are prepared.

### `apply_at(scene, t)`

This is the main progression method. It receives normalized progress `t` in the range `[0, 1]` and applies the corresponding state change to the scene.

### `on_finish`

`on_finish` is where the final permanent state is established once the animation has completed.

## The Reverse Reset Pass

One of the more important details in Murali is the reverse reset pass.

When a timeline is first initialized, or when time is explicitly sought, the timeline calls `reset(scene)` on its animations in reverse chronological order.

This gives the earliest animation affecting an object the final say about how that object should look at time zero.

That is why authored scenes do not need to manually pre-hide every object before a write animation. The reset pass establishes the correct baseline state from the animation schedule itself.

## Shared Props vs Type-Specific State

Animations in Murali usually mutate one of two things:

- shared drawable properties
- type-specific tattva state

### Shared drawable properties

These live in `DrawableProps` and include:

- position
- rotation
- scale
- visibility
- opacity

These are the properties used by generic transitions such as motion, rotation, scaling, and fading.

### Type-specific tattva state

This is the concrete state owned by the tattva itself. Examples include:

- path trim values on `Path`
- character reveal progress on `Label`
- write progress on `Table`
- write progress on `ParametricSurface`

These are used when an animation needs to affect something more specific than a transform or opacity.

This split is what allows Murali to support both:

- broad reusable animation patterns
- highly specialized effects tied to a specific tattva type

## DrawableProps In Practice

`DrawableProps` are shared through `SharedProps`, which is an `Arc<RwLock<DrawableProps>>`.

That is useful because:

- the frontend animation system can mutate them
- the backend runtime can read them
- transforms and visibility remain consistent across both sides

Common use cases for `DrawableProps` include:

- move an object without touching geometry
- rotate or scale it without rebuilding mesh data
- hide or show an object
- apply opacity-based transitions

Because the model matrix is computed from props at draw time, many transform changes can stay cheap relative to geometry rebuilds.

## Updaters

The timeline is not the only thing that can change scene state.

Murali also supports updaters, which are callbacks run each frame during `Scene::update`.

Updaters are useful when the behavior is not best modeled as a fixed-duration scheduled animation. For example:

- keep one object tracking another
- update a label from the current position of a target
- compute procedural motion or dependent state each frame

The timeline is ideal for deterministic scheduled transitions. Updaters are useful for procedural or relationship-driven logic.

## Dirty Flags And Progression

When animations or updaters change a tattva, they also need to communicate what kind of downstream work is required.

That is the role of dirty flags.

Dirty flags tell Murali whether the change affected things like:

- transform
- geometry
- style
- visibility
- bounds
- text layout

This matters because the next stages of the system need to know whether:

- the existing backend representation can still be reused
- or whether the tattva must be re-projected and rebuilt

So from the perspective of `Scene` and `Timeline`, dirty flags are the bridge from "state changed" to "what kind of sync work should happen next."

## How Scene Progression Hands Off To Projection

At the end of `scene.update(dt)`, the scene has only updated frontend state. Nothing has been rendered yet.

What exists at that point is:

- updated tattva state
- updated shared props
- updated timeline state
- dirty flags describing what changed

The engine then hands those dirty tattvas to the sync boundary, which projects them into render primitives and updates the backend runtime representation.

So the scene/timeline layer is responsible for state progression, while projection and sync are responsible for turning that state into renderable output.

## What This Page Owns

This page is the home for:

- scene authority
- time progression
- timeline scheduling
- animation lifecycle
- reset behavior
- updaters
- shared props vs type-specific state

For the next layer in the pipeline, continue with:

- [Tattva](./tattva)
- [Dirty flags](./dirty-flags)
- [Projection](./projection)
