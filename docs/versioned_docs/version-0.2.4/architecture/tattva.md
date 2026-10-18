---
sidebar_position: 3
---

# Tattva

The word *tattva* comes from Sanskrit and means "element", "essence", or "that which is real". In Murali, a tattva is the fundamental unit of a scene — anything that can be placed, animated, and rendered.

This page is about the tattva abstraction itself: the wrapper, the concrete state object, the shared props, and the traits that make a type scene-compatible.

## Philosophy

A tattva is not a render object. It doesn't know about GPU buffers, pipelines, or draw calls. It only knows two things:

1. What it looks like geometrically (via `Project`)
2. How big it is (via `Bounded`)

Everything else — identity, position, rotation, scale, opacity, dirty tracking — is handled by the `Tattva<T>` wrapper that the engine provides. This separation keeps shape implementations simple and focused. A `Circle` is just math. The engine handles the rest.

This also means tattvas are pure, deterministic, and easy to test. Given the same state, `project()` always produces the same primitives.

## The two layers

There are two distinct things called "tattva":

### `Tattva<T>` — the wrapper

```rust
pub struct Tattva<T> {
    pub id: TattvaId,
    pub state: T,        // the concrete shape (Circle, Square, Latex, ...)
    pub props: SharedProps,  // position, rotation, scale, opacity, visibility
    dirty: DirtyFlags,
}
```

This is what lives in `Scene::tattvas`. It owns the shape state and all runtime visual properties. It starts with `DirtyFlags::GEOMETRY` set so it gets materialized on the first frame.

### The concrete state type — e.g. `Circle`

```rust
pub struct Circle {
    pub radius: f32,
    pub segments: u32,
    pub style: Style,
}
```

This is pure data. No engine coupling, no GPU knowledge. It implements two traits and that's it.

## Traits a tattva state must implement

### `Project`

```rust
pub trait Project: Send + Sync {
    fn project(&self, ctx: &mut ProjectionCtx);
}
```

The shape's responsibility: given a `ProjectionCtx`, emit `RenderPrimitive` values that describe how to draw it. This is called by the sync boundary when the tattva is dirty and needs to be re-materialized on the GPU.

`project()` is pure — it reads `self` and writes to `ctx`. No side effects, no GPU calls.

### `Bounded`

```rust
pub trait Bounded {
    fn local_bounds(&self) -> Bounds;
}
```

Returns the axis-aligned bounding box in local space (before any transform from `SharedProps`). Used by layout helpers (`to_edge`, `next_to`) to position tattvas relative to each other or the screen edges.

`Bounds` is a 2D `min`/`max` rectangle — width and height can differ. Layout is inherently 2D; Z is ignored.

## How a concrete type becomes a tattva

The `IntoTattva` blanket impl handles this automatically:

```rust
impl<T> IntoTattva for T
where
    T: Project + Bounded + Send + Sync + 'static,
{
    fn into_tattva(self) -> Tattva<Self> {
        Tattva::new(0, self)
    }
}
```

Any type that implements `Project + Bounded` gets `IntoTattva` for free. `add_tattva` calls this internally — you never construct `Tattva<T>` directly.

## `TattvaTrait` — the object-safe interface

`Scene::tattvas` is a `HashMap<TattvaId, Box<dyn TattvaTrait>>`. To store heterogeneous types (`Circle`, `Square`, `Latex`, ...) in one collection, they need a common object-safe interface.

`TattvaTrait` is that interface. It's implemented via a blanket impl on `Tattva<T>`:

```rust
impl<T> TattvaTrait for Tattva<T>
where
    T: Project + Bounded + Send + Sync + 'static,
{ ... }
```

The trait exposes:
- `props()` — access to `SharedProps` (position, rotation, scale, opacity, visibility)
- `local_bounds()` — delegates to the inner state's `Bounded` impl
- `dirty_flags()`, `mark_dirty()`, `clear_dirty()` — dirty flag management
- `id()`, `set_id()` — identity
- `project()` — delegates to the inner state's `Project` impl
- `as_any()` / `as_any_mut()` — for downcasting back to `Tattva<T>` when you need typed access

The scene, animation system, and sync boundary all talk exclusively through `dyn TattvaTrait`. They never need to know the concrete type.

## `SharedProps` and `DrawableProps`

```rust
pub type SharedProps = Arc<RwLock<DrawableProps>>;

pub struct DrawableProps {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub visible: bool,
    pub opacity: f32,
    pub tag: Option<String>,
}
```

`SharedProps` is an `Arc<RwLock<DrawableProps>>`. The `Arc` allows the same props to be referenced by both the frontend tattva and the ECS component without copying. The `RwLock` allows the animation system and updaters to write to props while the renderer reads them.

The model matrix is computed from props at draw time:

```rust
Mat4::from_scale_rotation_translation(scale, rotation, position)
```

It's never baked into vertex data, which is why transform animations (`move_to`, `rotate_to`, `scale_to`) are cheap — they only update the props, not the GPU buffers.

## Limitations

- Layout helpers produce axis-aligned XY bounds after signed scale, rotation, and translation.
  `to_edge` intersects the camera frame with the world `z = 0` layout plane, including for
  perspective cameras. It does nothing when the camera rays are parallel to that plane.
- There is no retained parent/child hierarchy: tattvas are a flat list. `Group` can measure and move a set of IDs together, but later transforms are not propagated through a persistent group node.
- `project()` is called on the CPU every time a tattva is dirty with `REBUILD`. For very complex shapes with expensive tessellation, this can be a bottleneck
- `SharedProps` uses a `RwLock` — contention is unlikely in practice but worth knowing if you're writing high-frequency updaters

## What This Page Owns

This page is the home for:

- what a tattva is
- `Tattva<T>` vs the concrete state type
- `TattvaTrait`
- `Project` and `Bounded`
- `SharedProps` and `DrawableProps`

For time progression and animation lifecycle, see [Scene & Timeline](./scene-timeline).
