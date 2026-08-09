---
sidebar_position: 3
---

# Composite

Composite tattvas are made of multiple primitives. They live under `murali::frontend::collection::composite`.

## Card

A `Card` is a small rounded rectangle plus centered label. Use it for diagram nodes, callouts, dashboards, and explainer scenes where you want a reusable labeled box without manually adding a background and text every time.

```rust
use murali::frontend::collection::composite::Card;

let card = Card::new("Retrieved Context", 2.2, 0.58)
    .with_radius(0.12)
    .with_fill(Vec4::new(0.10, 0.14, 0.18, 0.72))
    .with_stroke(0.025, Vec4::new(0.42, 0.86, 0.96, 0.9))
    .with_text_style(0.18, Vec4::new(0.95, 0.98, 1.0, 1.0));

let ids = card.add_to_scene(&mut scene, Vec3::new(0.0, 0.0, 0.0));
```

`add_to_scene` returns `CardIds`:

```rust
ids.background; // rounded rectangle TattvaId
ids.label;      // label TattvaId
ids.all();      // [background, label]
```

That makes cards easy to animate as a unit:

```rust
for id in ids.all() {
    scene.hide_tattva(id);
    timeline.animate(id).at(0.4).for_duration(0.5).appear().spawn();
}

timeline
    .animate(ids.background)
    .at(0.4)
    .for_duration(0.6)
    .draw()
    .spawn();
timeline
    .animate(ids.label)
    .at(0.75)
    .for_duration(0.35)
    .typewrite_text()
    .spawn();
```

## Axes

2D coordinate axes with tick marks.

```rust
use murali::frontend::collection::composite::axes::Axes;

let axes = Axes::new(x_range: (f32, f32), y_range: (f32, f32))
    .with_step(1.0)          // tick spacing on both axes
    .with_thickness(0.03)    // line thickness
    .with_tick_size(0.16)    // tick mark length
    .with_color(Vec4)
    .without_ticks();        // optional: hide ticks

scene.add_tattva(axes, Vec3::ZERO);
```

Fields can also be set directly:

```rust
let mut axes = Axes::new((-5.0, 5.0), (-3.0, 3.0));
axes.x_step = 1.0;
axes.y_step = 0.5;
axes.thickness = 0.03;
axes.tick_size = 0.18;
axes.color = Vec4::new(0.75, 0.79, 0.85, 1.0);
```

Projects to `RenderPrimitive::Line` segments — one for each axis and one per tick mark.

## NumberPlane

A full grid background with distinct axis and grid line colors.

```rust
use murali::frontend::collection::composite::number_plane::NumberPlane;

scene.add_tattva(
    NumberPlane::new((-5.0, 5.0), (-3.5, 3.5))
        .with_step(1.0),
    Vec3::ZERO,
);
```

Default colors: grid lines are a muted grey, axis lines are brighter. Both are configurable via struct fields:

```rust
let mut plane = NumberPlane::new((-5.0, 5.0), (-3.5, 3.5));
plane.grid_color = Vec4::new(0.25, 0.28, 0.33, 1.0);
plane.axis_color = Vec4::new(0.78, 0.82, 0.88, 1.0);
plane.grid_thickness = 0.01;
plane.axis_thickness = 0.03;
```

Typically layered behind `Axes` and a graph:

```rust
scene.add_tattva(NumberPlane::new(...), Vec3::ZERO);  // bottom
scene.add_tattva(Axes::new(...), Vec3::ZERO);          // middle
scene.add_tattva(FunctionGraph::new(...), Vec3::ZERO); // top
```

## Axes3D

3D coordinate axes for use with parametric surfaces and 3D graphs.

```rust
use murali::frontend::collection::composite::axes3d::Axes3D;

scene.add_tattva(
    Axes3D::new((-1.5, 1.5), (-1.5, 1.5), (-1.5, 1.5))
        .with_step(0.5)
        .with_axis_thickness(0.04),
    Vec3::ZERO,
);
```

Layout helpers arrange transformed XY bounds. In perspective scenes, `to_edge` uses the camera
frame where it intersects the world `z = 0` layout plane. Use manual 3D placement when content does
not live on that plane.
