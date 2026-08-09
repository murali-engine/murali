---
sidebar_position: 4
---

# Dirty Flags

Dirty flags are how Murali classifies changes on a tattva and decides what kind of sync work has to happen next.

This page is specifically about change classification and sync consequences. For broader time progression, see [Scene & Timeline](./scene-timeline).

## Why Dirty Flags Exist

Not every change should trigger the same amount of work.

For example:

- moving an object is not the same as changing its geometry
- hiding an object is not the same as changing its text layout
- updating path trim state is not the same as recompiling LaTeX

Dirty flags let Murali distinguish those cases.

## The Flags

Each `Tattva<T>` carries a `DirtyFlags` bitmask.

The current flags are:

```rust
pub const TRANSFORM:    DirtyFlags = Self(1 << 0);
pub const GEOMETRY:     DirtyFlags = Self(1 << 1);
pub const STYLE:        DirtyFlags = Self(1 << 2);
pub const TEXT_LAYOUT:  DirtyFlags = Self(1 << 3);
pub const RASTER:       DirtyFlags = Self(1 << 4);
pub const BOUNDS:       DirtyFlags = Self(1 << 5);
pub const VISIBILITY:   DirtyFlags = Self(1 << 6);

pub const REBUILD: DirtyFlags =
    GEOMETRY | TEXT_LAYOUT | RASTER | BOUNDS;
```

At a conceptual level:

- `TRANSFORM` means shared transform props changed
- `GEOMETRY` means projected shape output changed
- `STYLE` means visual styling changed
- `TEXT_LAYOUT` means text layout needs recomputation
- `RASTER` means raster-backed content needs regeneration
- `BOUNDS` means layout-relevant bounds changed
- `VISIBILITY` means visibility or opacity-related state changed

## What The Sync Boundary Does With Them

The sync boundary reads dirty flags to decide what kind of update is required.

At a high level, the decision is:

```text
no dirty flags      -> do nothing
intersects REBUILD  -> re-project and rebuild backend entities
otherwise           -> runtime-only handling
```

The important split is between:

- rebuild work
- non-rebuild runtime changes

### Rebuild work

If the dirty flags intersect `REBUILD`, the sync boundary:

1. despawns cached backend entities for that tattva
2. projects the tattva again
3. materializes fresh backend entities from the new render primitives

This is required when the renderable structure itself may have changed.

### Runtime-only changes

If the change is limited to flags such as transform, style, or visibility, Murali does not necessarily need to rebuild backend entities.

That is possible because some runtime state is read at draw time from shared props rather than being baked into vertex data.

## Why Transform Changes Are Cheaper

One of the main reasons dirty flags exist is to preserve a cheaper path for transform-only changes.

If a tattva only changes:

- position
- rotation
- scale
- opacity
- visibility

then the underlying mesh or line structure may still be valid.

In those cases, Murali can often reuse the existing backend entity and simply draw it with different current props.

This is one of the core efficiency wins in the architecture.

## Where Dirty Flags Come From

Dirty flags are set when frontend state changes.

That usually happens in one of three places:

- animations
- updaters
- direct scene or tattva mutation

Typical examples:

- a move animation marks transform-related state dirty
- a write animation on a path marks geometry/style-related state dirty
- a text content change marks text layout or raster-related state dirty
- adding a new tattva starts it dirty so it is materialized on the first frame

The exact flags matter because they determine the cost of the next sync step.

## Important Practical Rule

Public typed mutation through `Scene::get_tattva_typed_mut` conservatively marks all dirty domains
before returning the tattva. Author code can mutate concrete `state` without remembering a separate
`mark_dirty` call.

Engine-owned animations and scene helpers use internal mutation paths with narrower dirty flags.
That preserves cheaper transform-only updates where the engine knows exactly what changed, while
the general public API favors correctness when it cannot infer which fields an arbitrary mutation
will touch.

## New And Removed Tattvas

New tattvas begin dirty so they are projected and materialized on the first frame.

Removed tattvas are handled separately from ordinary dirty updates. Their backend entities must be explicitly despawned so stale render entities do not survive after the frontend object is gone.

## What This Page Owns

This page is the home for:

- what each dirty flag means
- how flags affect sync decisions
- rebuild vs non-rebuild paths
- why transform changes can be cheaper

For the next stage after dirty flag classification, see [Projection](./projection) and [ECS](./ecs).
