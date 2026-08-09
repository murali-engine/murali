---
sidebar_position: 5
---

# Projection

Projection is the CPU-side step that turns a tattva's semantic state into backend-neutral render primitives.

This page is about that translation layer only. It does not cover ECS entity storage or renderer pipelines in detail.

## The Job Of Projection

By the time projection runs, the frontend already has:

- current tattva state
- current shared props
- dirty flags indicating that sync work is needed

Projection answers the next question:

> Given this tattva in its current state, what should be drawn?

The answer is a list of `RenderPrimitive` values.

## The `Project` Trait

Each tattva state type implements `Project`:

```rust
pub trait Project: Send + Sync {
    fn project(&self, ctx: &mut ProjectionCtx);
}
```

This method is pure CPU work.

It does not:

- upload GPU buffers
- create backend entities
- issue draw calls

It only emits render-oriented primitives into the provided context.

## ProjectionCtx

`ProjectionCtx` is the temporary container used during projection.

Conceptually, it holds:

- the tattva's shared props
- the render primitives emitted during projection
- structured `ValidationError` diagnostics raised during projection

The props are available so projection code can reference runtime visual state when needed. The
primitive list is the output collected by the sync boundary. Diagnostics are fail-closed: the sync
boundary rejects that tattva's projection and `Engine::update` returns `EngineError::Validation`
instead of silently keeping invalid or partial geometry.

So `ProjectionCtx` is the handoff object between:

- the semantic tattva
- the later backend materialization step

## RenderPrimitive

The output of projection is a `Vec<RenderPrimitive>`.

These primitives are backend-neutral descriptions such as:

- `Mesh`
- `Line`
- `Text`
- `Latex`
- `Typst`

Each variant represents a different kind of renderable output.

### Mesh

Used for pre-tessellated geometry such as:

- filled shapes
- polygonal surfaces
- text quads after mesh building

### Line

Used for line-like output such as:

- strokes
- guides
- path segments emitted as lines

### Text, Latex, Typst

These represent text-oriented render requests that will later be materialized into backend resources using the appropriate text pipeline.

## Example Mental Model

For a circle, projection may emit:

- one mesh primitive for the fill
- several line primitives for the stroke

For a label, projection may emit:

- a text primitive that later becomes a textured mesh

For a path, projection may emit:

- line primitives for stroke segments
- mesh primitives for trimmed fill geometry

So projection is not tied to one object becoming one draw call. It is a translation step, not a final rendering step.

## Why Projection Is Separate

Murali keeps projection separate from both the frontend and the renderer for a reason.

If it lived inside the renderer:

- semantic object logic and rendering logic would get mixed together

If it lived entirely inside the frontend object model:

- the bridge into backend-neutral render output would become less explicit

Projection gives Murali a clean middle layer where:

- tattvas stay semantic
- render output becomes explicit
- backend materialization can remain generic

## Projection Is Still Derived State

Projection output is not authoritative state. It is derived from:

- tattva state
- shared props

That means if frontend state changes again, projection can simply be recomputed. This is another reason Murali keeps the source of truth in the scene rather than in backend entities.

## What This Page Owns

This page is the home for:

- the `Project` trait
- the purpose of `ProjectionCtx`
- the role of `RenderPrimitive`
- why projection is a separate CPU-side stage

For what happens next, see [ECS](./ecs) and [Renderer](./renderer).
