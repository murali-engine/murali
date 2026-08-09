---
sidebar_position: 0
---

# Technical Overview

This section is meant to give you a high-level mental model of how Murali is wired before you go deeper into individual subsystems.

The goal is not to explain every type or every file. The goal is to help you answer these questions first:

- Where does user-authored animation logic live?
- What does the app runner do?
- What does the engine do every frame?
- Where is the source of truth?
- How do frontend objects become something the GPU can render?

Once this high-level picture is clear, the more detailed architecture pages become much easier to follow.

## Start With The Animation Script

Most Murali programs begin as a scene script.

That script typically:

- creates a `Scene`
- adds tattvas to the scene
- creates the global timeline and any local-time clips
- composes clips and animations onto the timeline
- hands the scene to `App`

You can think of this layer as the authored intent of the animation. It describes what should exist, how it should be arranged, and how it should evolve over time.

The important thing is that the script is not itself the render loop. It defines the scene and animation behavior, but something else is responsible for advancing time and drawing frames.

## What The App Does

`App` is the entry point that decides how Murali runs.

In preview mode, `App` is tied to the windowing/event loop model provided by `winit`. A window is created, redraw events are requested, and on each redraw Murali advances the engine and renders the next frame.

In export mode, the same authored scene is run headlessly without a preview window. Instead of drawing to a live surface, Murali advances the scene on a fixed timestep and renders images for export.

So the right way to think about `App` is:

- it owns the outer run loop
- it decides whether we are previewing or exporting
- it feeds frame timing into the engine
- it does not own animation logic itself

## What The Engine Does

`Engine` is the per-frame coordinator.

For each frame, it does two main things:

1. advances the frontend scene by `dt`
2. synchronizes changed frontend objects into backend render state

At a high level, that looks like this:

```rust
engine.update(dt)?;
engine.render()?;
```

Inside `update(dt)`, the engine:

- asks the scene to move forward in time
- lets the timeline apply animation state
- lets updaters run
- removes any frontend objects that were deleted
- syncs dirty tattvas into backend entities and GPU resources

The engine is the heartbeat of the system. It does not define the animation, but it is responsible for progressing the authored scene and keeping the render runtime in sync.

## Scene Is The Source Of Truth

The most important architectural idea in Murali is that the `Scene` is authoritative.

The scene owns:

- tattvas
- the global timeline
- scene time
- camera
- updaters

This means the backend is not the source of truth. The GPU world is only a derived runtime view of the scene.

That distinction matters because it keeps Murali deterministic:

- user code mutates scene state
- animations mutate scene state
- the backend is rebuilt or updated from that scene state

So if something looks wrong on screen, the place to reason about it is usually the frontend state first, not the GPU representation.

## Tattvas, Timeline, And Clips

Tattvas are the semantic objects in the scene: shapes, text, tables, graph objects, utility tattvas, and so on.

A timeline schedules animations against time, but the actual progression happens through the `Scene`.

When the engine calls `scene.update(dt)`, the scene does several things in order:

1. advances `scene_time`
2. ticks the global timeline
3. lets animations mutate tattva state or shared props
4. runs updaters
5. leaves the changed tattvas marked for later sync and projection

So the scene is where time, authored objects, and runtime progression come together.

When a timeline runs, it checks each scheduled animation relative to the current `scene_time` and decides whether the animation is:

- pending
- running
- done

Each animation can then participate in the frame lifecycle through methods such as:

- `reset`
- `on_start`
- `apply_at(scene, t)`
- `on_finish`

This is how Murali supports not just simple transforms, but also richer transitions such as:

- motion, rotation, and scale
- visibility and opacity changes
- path write and unwrite effects
- text reveal and unreveal effects
- table and surface progression
- temporary helper objects used during a transition

That means:

- the timeline decides when something should happen
- the animation decides how the target should change
- the scene stores the current resulting state

This is why Murali feels time-driven rather than frame-scripted. The authored intent is expressed in terms of time and state transitions, and each frame is derived from that.

## DrawableProps And Tattva State

Murali separates common runtime properties from type-specific object state.

The common runtime properties live in `DrawableProps`. These include:

- position
- rotation
- scale
- visibility
- opacity

These are shared through `SharedProps`, which allows the same properties to be referenced by both the frontend tattva and the backend runtime entity.

This makes `DrawableProps` the natural place for transitions such as:

- move an object
- rotate it
- scale it
- hide or show it
- fade it in or out

Type-specific state stays on the tattva itself. That includes things like:

- path segments and trim progress for a `Path`
- text reveal progress for a `Label`
- write progress for a `Table` or `ParametricSurface`
- any geometry or style data unique to that tattva

This split is useful because it gives Murali both:

- generic transitions that work across many tattvas
- specialized transitions that can target a concrete tattva type

## Dirty Flags

Once a tattva changes, Murali needs to know what kind of downstream work is required.

That is the role of dirty flags.

Dirty flags describe what changed on a tattva, for example:

- transform
- geometry
- style
- visibility
- text layout
- bounds

They matter because not every change should cause the same amount of work.

For example:

- a pure transform change can often reuse the same geometry
- a geometry change usually requires re-projection and backend rebuild
- a text content change may require fresh layout and raster work

So dirty flags are one of the key mechanisms that connect semantic state changes to efficient runtime updates.

## Projection And ProjectionCtx

After the scene has advanced, Murali still has not rendered anything yet. The next step is projection.

Projection is the CPU-side step where a tattva converts its semantic state into render-oriented primitives. This happens through the `Project` trait, which writes output into a `ProjectionCtx`.

`ProjectionCtx` is the container used during projection. It carries:

- the tattva's shared props
- the list of `RenderPrimitive` values emitted during projection

Those emitted primitives are backend-neutral descriptions such as:

- meshes
- lines
- text primitives
- LaTeX and Typst render requests

This stage is important because it keeps a clean separation of concerns:

- tattvas stay semantic
- projection translates them into draw-oriented primitives
- the backend later materializes those primitives into ECS entities and GPU resources

That is why projection sits between frontend state and backend rendering instead of being absorbed into either side.

## Frontend, Projection, Backend

A useful mental model for Murali is:

```text
Authored scene -> Frontend state -> Projection -> Backend runtime -> Renderer
```

Here is what each stage is responsible for:

### Frontend

The frontend contains the objects users think in:

- `Scene`
- `Tattva<T>`
- the timeline, clips, and animations
- shared drawable props
- layout helpers

This layer knows about meaning and intent.

### Projection

Projection converts semantic frontend objects into backend-neutral render primitives like:

- lines
- meshes
- text primitives
- raster-backed quads

Projection is still CPU-side. It is not the renderer. It is the stage where "what this object means" becomes "what needs to be drawn."

### Backend

The backend owns:

- the ECS render world
- GPU resources
- renderer state
- device/surface management

This layer knows how to upload buffers, cache entities, and issue draw calls. It does not know about user-authored animation scripts in the same way the frontend does.

## Why The Sync Boundary Exists

The sync boundary is what keeps the architecture clean.

It sits between frontend and backend and translates dirty frontend objects into backend runtime state.

That separation is useful because:

- the frontend stays semantic and deterministic
- the backend stays focused on rendering concerns
- GPU resources can be rebuilt only when needed
- transform-only changes can remain cheaper than full geometry rebuilds

Without the sync boundary, the authored scene model and the render runtime would get tangled together very quickly.

## Preview And Export Share The Same Model

One important design goal in Murali is that preview and export should both run from the same scene model.

They differ in how frames are produced:

- preview runs inside an interactive event loop
- export runs headlessly on a fixed timestep

But both are still driven by:

- the same `Scene`
- the same global timeline
- the same animation logic
- the same projection and rendering pipeline

This keeps behavior more predictable and reduces the chance that "preview mode" and "export mode" drift apart conceptually.

## What To Read Next

If this page gave you the high-level picture, the next useful pages are:

- [Scene & Timeline](scene-timeline)
- [End-to-end flow](end-to-end-flow)
- [Tattva](tattva)
- [Dirty flags](dirty-flags)

Those pages go one level deeper into how Murali maintains state, decides what changed, and turns authored objects into rendered output.
