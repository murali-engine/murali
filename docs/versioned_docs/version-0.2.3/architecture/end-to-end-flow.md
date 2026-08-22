---
sidebar_position: 9
---

# End-to-end flow

This document traces exactly what happens when you write a simple scene and run it — from the first line of user code to pixels on screen. We'll use a circle as the example throughout.

Unlike the other pages in this section, this page is intentionally example-driven. Its job is to walk one concrete object through the whole system rather than define each subsystem in isolation.

```rust
let mut scene = Scene::new();
let id = scene.add_tattva(
    Circle::new(1.0, 64, Vec4::new(0.2, 0.6, 1.0, 1.0)),
    Vec3::new(0.0, 0.0, 0.0),
);
scene.camera_mut().position = Vec3::new(0.0, 0.0, 10.0);
App::new()?.with_scene(scene).run_app()
```

---

## Step 1 — User constructs a Circle

```rust
Circle::new(1.0, 64, Vec4::new(0.2, 0.6, 1.0, 1.0))
```

`Circle` is plain data. No GPU calls, no allocation beyond the struct itself:

```rust
pub struct Circle {
    pub radius: f32,      // 1.0
    pub segments: u32,    // 64
    pub style: Style,     // fill color = (0.2, 0.6, 1.0, 1.0)
}
```

---

## Step 2 — add_tattva wraps it in Tattva\<Circle\>

```rust
scene.add_tattva(circle, Vec3::new(0.0, 0.0, 0.0))
```

Internally this calls `circle.into_tattva()` via the `IntoTattva` blanket impl, producing:

```rust
Tattva<Circle> {
    id: 0,           // assigned next
    state: circle,   // the Circle struct
    props: Arc<RwLock<DrawableProps {
        position: Vec3(0.0, 0.0, 0.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
        visible: true,
        opacity: 1.0,
    }>>,
    dirty: DirtyFlags::GEOMETRY,  // starts dirty — needs first-frame materialization
}
```

The position is written into `props`, the tattva is assigned the next available `TattvaId`, and stored in `scene.tattvas` as `Box<dyn TattvaTrait>`.

---

## Step 3 — App::run_app starts the event loop

`App::run_app` creates a winit event loop and an `Engine`. The engine owns:

- `scene` — the frontend state
- `backend` — GPU device, renderer, ECS world
- `sync_boundary` — the bridge between them

On each frame, winit fires a `RedrawRequested` event which calls:

```rust
engine.update(dt)?;
engine.render()?;
```

---

## Step 4 — engine.update(dt): frontend tick

```rust
pub fn update(&mut self, dt: f32) -> Result<(), EngineError> {
    self.scene.update(dt)?;      // 1. advance time, run timeline & updaters
    // 2. sync dirty tattvas to GPU
    for (_id, tattva) in self.scene.tattvas_iter_mut() {
        self.sync_boundary.sync_tattva(
            &mut self.backend.world,
            &self.backend.renderer.device_mgr.device,
            &self.backend.renderer,
            tattva.as_mut(),
        )?;
    }
    Ok(())
}
```

### 4a — scene.update(dt)

`scene_time` advances by `dt`. The global timeline ticks and applies any animations to tattva props. Updaters run. For our static circle on frame 1, nothing changes — the circle is still dirty with `GEOMETRY` from when it was added.

### 4b — sync_boundary.sync_tattva

The sync boundary checks the dirty flags:

```rust
let dirty = tattva.dirty_flags();
if dirty.is_empty() { return; }  // nothing to do

if dirty.intersects(REBUILD) {   // GEOMETRY is in REBUILD
    self.rebuild_render_entities(world, device, renderer, tattva);
    tattva.clear_all_dirty();
}
```

`GEOMETRY` is part of `REBUILD`, so a full rebuild is triggered.

---

## Step 5 — Projection: Circle → RenderPrimitive

Inside `rebuild_render_entities`, the sync boundary calls:

```rust
fn project_tattva(&self, tattva: &dyn TattvaTrait) -> Vec<RenderPrimitive> {
    let mut ctx = ProjectionCtx::new(tattva.props().clone());
    tattva.project(&mut ctx);
    ctx.primitives
}
```

This calls `Circle::project`:

```rust
impl Project for Circle {
    fn project(&self, ctx: &mut ProjectionCtx) {
        if let Some(fill) = &self.style.fill {
            let mesh = Mesh::circle(self.radius, self.segments, fill.clone());
            ctx.emit(RenderPrimitive::Mesh(mesh));
        }
        // stroke would emit RenderPrimitive::Line segments
    }
}
```

`Mesh::circle` tessellates the circle into 64 triangles using lyon, producing a `Mesh` with:
- `MeshData::Mesh(Vec<MeshVertex>)` — 65 vertices (center + 64 perimeter points)
- `indices: Vec<u32>` — 192 indices (64 triangles × 3)

`ctx.emit` pushes this as `RenderPrimitive::Mesh(mesh)` into `ctx.primitives`.

---

## Step 6 — Materialization: RenderPrimitive → ECS entity

Back in `materialize_primitives`, the sync boundary processes each `RenderPrimitive`:

```rust
RenderPrimitive::Mesh(mesh) => {
    upload_mesh(device, mesh.as_ref(), None).map(|mesh_instance| {
        world.spawn((
            MeshComponent(Arc::new(mesh_instance)),
            tattva.props().clone(),  // SharedProps as a component
        ))
    })
}
```

`upload_mesh` calls `MeshInstance::new` which creates two wgpu buffers:

```rust
// vertex buffer — uploaded to GPU VRAM
device.create_buffer_init(&BufferInitDescriptor {
    contents: bytemuck::cast_slice(&vertices),
    usage: BufferUsages::VERTEX,
})

// index buffer — uploaded to GPU VRAM
device.create_buffer_init(&BufferInitDescriptor {
    contents: bytemuck::cast_slice(&indices),
    usage: BufferUsages::INDEX,
})
```

`world.spawn(...)` creates a new hecs entity with two components:
- `MeshComponent` — wraps the GPU buffers in an `Arc<MeshInstance>`
- `SharedProps` — the same `Arc<RwLock<DrawableProps>>` the frontend tattva holds

The entity ID is stored in `sync_boundary.entity_cache[tattva_id]`. The dirty flags are cleared.

---

## Step 7 — engine.render(): draw the ECS world

```rust
pub fn render(&mut self) -> Result<()> {
    self.backend.renderer.render_scene(&self.scene, &self.backend.world)
}
```

Inside `render_scene`:

**7a — Acquire frame**

```rust
let (frame, view) = self.device_mgr.acquire_frame()?;
```

Gets the next surface texture from the swapchain.

**7b — Compute view-projection matrix**

```rust
let view_proj = scene.camera.view_proj_matrix();
```

The camera at `(0, 0, 10)` looking at the origin produces a perspective projection matrix.

**7c — Build draw list**

The renderer queries the ECS world for all `(MeshComponent, SharedProps)` pairs:

```rust
for (_, (mesh_comp, props)) in world.query::<(&MeshComponent, &SharedProps)>().iter() {
    let props = DrawableProps::read(props);
    if !props.visible || props.opacity <= 0.0 { continue; }
    list.push((mesh_comp.0.clone(), props.model_matrix(), mesh_comp.0.bind_group.clone(), props.opacity));
}
```

`props.model_matrix()` computes `Mat4::from_scale_rotation_translation(scale, rotation, position)` — for our circle at the origin with default scale/rotation this is the identity matrix.

**7d — Encode render pass**

A render pass is begun with a clear color (dark background). For each mesh in the draw list:

```rust
let mvp = view_proj * model;  // perspective * identity = perspective
let offset = draw_idx * uniform_slot_size;

// Write MVP + alpha into the pre-allocated uniform buffer
queue.write_buffer(&uniform_buffer, offset, bytemuck::cast_slice(&[
    Uniforms { mvp: mvp.to_cols_array_2d(), alpha: 1.0, .. }
]));

rpass.set_pipeline(&self.mesh_pipeline);
rpass.set_bind_group(0, &self.uniform_bind_group, &[offset as u32]);
rpass.set_bind_group(1, &self.default_texture_bind_group, &[]);
mesh.draw(&mut rpass);  // set_vertex_buffer + set_index_buffer + draw_indexed
```

**7e — Submit and present**

```rust
queue.submit(Some(encoder.finish()));
frame.present();
```

The GPU executes the render pass. The mesh shader reads the vertex positions, transforms them by the MVP matrix, and outputs the circle's triangles in clip space. The fragment shader reads the vertex color and outputs the final RGBA pixel values. The swapchain presents the frame to the window.

---

## Summary

```
User code
  └─ Circle::new()                    pure data, no GPU
  └─ scene.add_tattva()               wraps in Tattva<Circle>, sets DirtyFlags::GEOMETRY

Frame loop (per frame)
  └─ scene.update(dt)                 advance time, run timeline/updaters
  └─ sync_boundary.sync_tattva()
       └─ dirty & REBUILD?
            └─ tattva.project()       Circle → Vec<RenderPrimitive::Mesh>
            └─ upload_mesh()          tessellated vertices → wgpu vertex/index buffers
            └─ world.spawn()          MeshComponent + SharedProps → hecs entity
  └─ renderer.render_scene()
       └─ acquire_frame()             get swapchain texture
       └─ query ECS world             collect draw list
       └─ for each mesh:
            └─ compute MVP            view_proj * model_matrix
            └─ write uniform buffer   MVP + alpha
            └─ draw_indexed()         GPU draws triangles
       └─ present()                   frame appears on screen
```

On subsequent frames where nothing changes, `dirty_flags` is empty and the sync boundary skips the tattva entirely — no re-tessellation, no buffer uploads. The ECS entity from frame 1 is reused as-is.
