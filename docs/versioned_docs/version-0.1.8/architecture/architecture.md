---
sidebar_position: 1
---

# Architecture

This page describes the static shape of the Murali system: the major layers, what each layer owns, and how data is allowed to flow between them.

If [Technical Overview](./overview) is the onboarding page, this page is the structural map.

## The Big Picture

Murali is organized around a one-way flow:

```text
User script
  -> Scene, timeline, and clips
  -> Tattvas and shared props
  -> Projection
  -> Sync boundary
  -> Backend ECS world
  -> Renderer
  -> GPU / pixels
```

That one-way flow is important. It means:

- authored scene state lives in the frontend
- the backend is a derived runtime view
- rendering never becomes the source of truth

## The Main Layers

At a high level, Murali has four conceptual layers:

```text
┌─────────────────────────────────────────┐
│              User code                  │
│   Scene scripts, examples, app setup    │
└────────────────────┬────────────────────┘
                     │
┌────────────────────▼────────────────────┐
│               Frontend                  │
│ Scene, Timeline, Clip, Tattva<T>, Props │
│ Layout, Animation state                 │
└────────────────────┬────────────────────┘
                     │
┌────────────────────▼────────────────────┐
│              Projection                 │
│ Project trait, ProjectionCtx,           │
│ RenderPrimitive                         │
└────────────────────┬────────────────────┘
                     │  SyncBoundary
┌────────────────────▼────────────────────┐
│               Backend                   │
│ ECS world, mesh instances, renderer,    │
│ device manager                          │
└─────────────────────────────────────────┘
```

## User Code

The outermost layer is ordinary user-authored Rust code.

This is where a scene is described:

- create a `Scene`
- add tattvas
- build clips and the global timeline
- schedule animations
- pass the scene into `App`

This layer expresses intent. It says what should exist and how it should evolve, but it does not deal with ECS entities, GPU buffers, or render passes.

## Frontend

The frontend is the semantic authoring layer.

It owns concepts such as:

- `Scene`
- `Timeline`
- `Clip`
- `Tattva<T>`
- `DrawableProps`
- layout helpers
- animation state

This layer answers questions like:

- what objects are in the scene?
- what is their current semantic state?
- where are they positioned?
- what animations are currently affecting them?

The frontend is also where time-driven progression happens. The timeline mutates frontend state, not backend render entities.

## Projection

Projection is the translation layer between semantic objects and render-oriented primitives.

Each tattva state implements `Project` and emits values into a `ProjectionCtx`. The output is a list of `RenderPrimitive` values such as:

- meshes
- lines
- text requests
- LaTeX and Typst raster requests

Projection is still CPU-side. It does not upload anything to the GPU. Its job is to turn "what this object means" into "what needs to be drawn."

## Sync Boundary

The sync boundary is the bridge between frontend state and backend runtime data.

Its job is to:

- inspect dirty flags on tattvas
- decide whether a tattva needs rebuild work or only runtime handling
- project dirty tattvas into `RenderPrimitive`s
- materialize those primitives into backend entities and GPU resources
- remove stale entities when a tattva goes away

This is one of the key architectural boundaries in Murali. It prevents frontend authoring concerns and backend rendering concerns from collapsing into one layer.

## Backend

The backend owns runtime rendering infrastructure:

- `hecs::World`
- uploaded mesh instances
- renderer state
- wgpu device, queue, and surface configuration

The backend does not reason about authored intent. It does not know that a circle was created because of a specific animation script or that a line is part of a teaching scene. It only knows how to manage renderable entities and draw them efficiently.

## Ownership Boundaries

The most important ownership rule in Murali is:

- the frontend owns meaning
- the backend owns execution

More concretely:

- `Scene` owns tattvas, the global timeline, time, and camera
- `Tattva<T>` owns semantic object state plus shared runtime props
- `ProjectionCtx` owns temporary projected output during a sync pass
- `SyncBoundary` owns the mapping between a frontend tattva and backend entities
- the backend ECS world owns concrete render entities
- `Renderer` owns pipelines, buffers, and draw encoding

This division keeps debugging clearer. If behavior is wrong, you usually inspect frontend state first. If rendering is wrong despite correct frontend state, you inspect projection, sync, or backend runtime layers next.

## Why The Architecture Is Split This Way

This structure gives Murali a few important properties:

### Determinism

Animations and scene logic evolve frontend state over time. The backend is derived from that state rather than mutating it independently.

### Separation of concerns

Scene authoring, projection, sync, and rendering can each evolve without turning into one giant mixed abstraction.

### Efficiency

Transform-only changes do not need to behave like full geometry rebuilds. Dirty flags and the sync boundary allow cheaper paths where appropriate.

### Better mental model

Contributors can reason about the system layer by layer:

- authored scene state
- projected render primitives
- backend entities
- GPU draw calls

## How To Read The Rest Of This Section

From here, the architecture docs go deeper into the main boundaries:

- [Scene & Timeline](./scene-timeline) explains time progression and state mutation
- [Tattva](./tattva) explains the object model and shared props
- [Dirty flags](./dirty-flags) explains how changes are classified
- [Projection](./projection) explains `Project`, `ProjectionCtx`, and `RenderPrimitive`
- [ECS](./ecs) explains backend entity representation
- [Renderer](./renderer) explains pipelines and frame encoding
- [End-to-end flow](./end-to-end-flow) walks one example all the way from script to pixels
