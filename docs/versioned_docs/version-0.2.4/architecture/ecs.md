---
sidebar_position: 6
---

# ECS & components

Murali uses [hecs](https://github.com/Ralith/hecs) as a lightweight ECS (Entity Component System) for the GPU-side world. The ECS is entirely internal — user code never interacts with it directly.

This page is specifically about how backend entities are represented once projection has already happened.

## Why ECS?

The backend needs to iterate over potentially thousands of drawable objects per frame and batch them efficiently. ECS gives cache-friendly iteration over homogeneous component sets without the overhead of vtable dispatch.

## The world

`Backend` owns a `hecs::World`. Entities in this world represent individual GPU draw calls — not tattvas. A single tattva can produce multiple entities (e.g. a composite shape with several sub-meshes, or a text label with a separate background).

## Components

### MeshComponent

```rust
pub struct MeshComponent(pub Arc<MeshInstance>);
```

Wraps a GPU-uploaded mesh (vertex buffer + index buffer + optional texture bind group). Used for all filled geometry and text quads.

### LineComponent

```rust
pub struct LineComponent {
    pub start: Vec3,
    pub end: Vec3,
    pub thickness: f32,
    pub dash_length: f32,
    pub gap_length: f32,
    pub dash_offset: f32,
}
```

Lines are not uploaded as vertex buffers. Instead, line data is packed into a GPU storage buffer each frame and the geometry is generated entirely in the vertex shader. This avoids re-uploading when line endpoints change.

### ColorComponent

```rust
pub struct ColorComponent(pub Vec4);
```

Stores the RGBA color for the line shader. Mesh colors are baked into vertex data.

### TransformComponent (SharedProps)

```rust
pub type TransformComponent = SharedProps;
```

`SharedProps` is used directly as a component. It holds position, rotation, scale, opacity, and visibility. The renderer reads this at draw time to compute the model matrix — it's never baked into vertex data, which is why transform animations are cheap.

## Entity lifecycle

Entities are created and destroyed by `SyncBoundary`, not by user code:

1. Tattva marked dirty with `REBUILD` flag
2. `SyncBoundary` despawns all entities previously associated with that tattva
3. `tattva.project()` produces `Vec<RenderPrimitive>`
4. Each primitive becomes one or more new entities
5. Entity IDs are stored in `entity_cache` keyed by `TattvaId`

When a tattva is removed from the scene, its cached entities are despawned.

## What This Page Owns

This page is the home for:

- why the backend uses ECS
- what backend entities represent
- the major backend components
- how entities are created and destroyed

For what gets stored before ECS materialization, see [Projection](./projection). For how the renderer draws those entities, see [Renderer](./renderer).
