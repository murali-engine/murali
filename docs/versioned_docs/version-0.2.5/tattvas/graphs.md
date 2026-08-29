---
sidebar_position: 4
---

# Graphs

Graphing tattvas live under their subject owners:
`murali::frontend::collection::maths::calculus` for continuous curves, fields, and surfaces, and
`murali::frontend::collection::maths::data_geometry` for sampled point data. Graph-specific helpers
such as legends live under `murali::frontend::collection::maths::graph`.

Use these families when the scene is primarily about curves, fields, surfaces, or geometric data.
If you are teaching calculus, vector fields, geometry, or 3D surfaces, start with `calculus`. If
you are teaching sampled data, embeddings, or point clouds, start with `data_geometry`.

If you need text, labels, or formulas around the graph, pair this page with [Text](./text.md), [Math](./math.md), [Camera](../camera.md), and [Examples](../examples/showcase).

## Quick Decision Guide

| Need | Use | Why |
|---|---|---|
| Plot `y = f(x)` | `FunctionGraph` | Best for ordinary 2D function plots |
| Plot a 2D curve `t -> Vec2` | `ParametricCurve` | Use when x and y both depend on a parameter |
| Plot a 3D curve `t -> Vec3` | `ParametricCurve3D` | Best for spirals, trajectories, and 3D paths |
| Plot a cloud of points | `ScatterPlot` | Best for samples, embeddings, or discrete data |
| Show arrows of a field | `VectorField` | Best for local direction/magnitude |
| Show traced flow through a field | `StreamLines` | Best for flow structure over time |
| Show a 3D surface `(u, v) -> Vec3` | `ParametricSurface` | Best for spheres, waves, and geometry surfaces |
| Add a legend/key | `PlotLegend` | Best for naming plotted series without making a one-off layout |

## FunctionGraph

Plots a Rust function `f32 -> f32` over an x range as a polyline.

```rust
use murali::frontend::collection::maths::calculus::function_graph::FunctionGraph;
use murali::colors::*;
use murali::positions::*;

fn sine(x: f32) -> f32 { x.sin() }

scene.add_tattva(
    FunctionGraph::new((-5.0, 5.0), sine)
        .with_samples(200),
    Vec3::ZERO,
);
```

`with_samples(n)` controls the number of line segments. More samples = smoother curve, more primitives.

Use `FunctionGraph` when the natural explanation is “here is `y` as a function of `x`.” If you find yourself deriving both coordinates from one parameter, move to `ParametricCurve`.

## PlotLegend

Adds a compact graph legend for plotted series.

```rust
use murali::frontend::collection::maths::graph::{PlotLegend, PlotLegendEntry};
use murali::colors::*;

scene.add_tattva(
    PlotLegend::new(vec![
        PlotLegendEntry::new("sine curve", BLUE_B),
        PlotLegendEntry::new("sampled extrema", GOLD_C),
    ]),
    Vec3::new(4.4, 1.45, 0.0),
);
```

Use `PlotLegend` when the graph needs a reusable visual key. Keep curve and point rendering in
`calculus` or `data_geometry`; keep the graph-specific presentation helper here.

## ScatterPlot

Plots a list of `Vec2` points as dots.

```rust
use murali::frontend::collection::maths::data_geometry::scatter_plot::ScatterPlot;
use glam::vec2;

scene.add_tattva(
    ScatterPlot::new(vec![
        vec2(-2.0, 1.5),
        vec2(0.0, 0.0),
        vec2(2.0, -1.5),
    ]),
    Vec3::ZERO,
);
```

## ParametricCurve

A 2D parametric curve `t -> Vec2`.

```rust
use murali::frontend::collection::maths::calculus::parametric_curve::ParametricCurve;

scene.add_tattva(
    ParametricCurve::new((0.0, std::f32::consts::TAU), |t| {
        glam::vec2(t.cos(), t.sin())
    }).with_samples(128),
    Vec3::ZERO,
);
```

Use `ParametricCurve` when:

- the curve loops or doubles back in x
- both x and y are naturally parameterized
- you are drawing geometric paths rather than “function graphs”

## ParametricCurve3D

A 3D parametric curve `t -> Vec3`.

```rust
use murali::frontend::collection::maths::calculus::parametric_curve3d::ParametricCurve3D;
use std::f32::consts::TAU;

scene.add_tattva(
    ParametricCurve3D::new((0.0, 3.0 * TAU), |t| {
        Vec3::new(t.cos(), t.sin(), 0.15 * t)
    })
    .with_samples(256),
    Vec3::ZERO,
);
```

This is a good fit for helices, orbital paths, trajectories, and “camera needs to matter” scenes.

## ParametricSurface

A 3D surface defined by `(u, v) -> Vec3`. Renders as a wireframe mesh.

```rust
use murali::frontend::collection::maths::calculus::parametric_surface::ParametricSurface;
use std::f32::consts::PI;

scene.add_tattva(
    ParametricSurface::new(
        (0.0, PI),
        (0.0, 2.0 * PI),
        |u, v| Vec3::new(u.sin() * v.cos(), u.sin() * v.sin(), u.cos()),
    )
    .with_samples(40, 40)
    .with_color(TEAL_C),
    Vec3::ZERO,
);
```

`with_samples(u_steps, v_steps)` controls mesh resolution.

Use `ParametricSurface` when the scene is fundamentally 3D. If the concept can still be taught with a 2D curve or field, prefer that first because it is easier to frame and annotate.

You can also keep surface construction separate from texture loading and let `Scene` do the path-based loading:

```rust
let globe = ParametricSurface::new(
    (0.0, PI),
    (0.0, 2.0 * PI),
    |u, v| Vec3::new(u.sin() * v.cos(), u.cos(), u.sin() * v.sin()),
)
.with_samples(48, 72);

scene.add_textured_surface_with_path(
    globe,
    "src/resource/assets/earthmap1k.jpg",
    Vec3::ZERO,
)?;
```

## VectorField

Displays arrows at grid points representing a vector function `Vec2 -> Vec2`.

```rust
use murali::frontend::collection::maths::calculus::vector_field::VectorField;

scene.add_tattva(
    VectorField::new(
        (-4.0, 4.0), (-3.0, 3.0),
        8, 6,
        |p| glam::vec2(-p.y, p.x), // rotation field
    )
    .with_color(Vec4::new(BLUE_B.x, BLUE_B.y, BLUE_B.z, 0.8))
    .with_length_scale(0.4),
    Vec3::ZERO,
);
```

Color vectors by magnitude:

```rust
.with_color_fn(|magnitude| {
    let t = (magnitude / 3.0).clamp(0.0, 1.0);
    Vec4::new(t, 0.5, 1.0 - t, 0.8)
})
```

Use `VectorField` when you want viewers to read local direction and magnitude at many points.

## StreamLines

Traces flow paths through a vector field using Euler integration.

```rust
use murali::frontend::collection::maths::calculus::stream_lines::{StreamLines, line_start_points};

scene.add_tattva(
    StreamLines::from_grid(
        (-4.0, 4.0), (-3.0, 3.0),
        6, 5,
        |p| glam::vec2(-p.y, p.x),
    )?
    .with_color(Vec4::new(0.4, 0.8, 1.0, 0.7))
    .with_step_size(0.05)
    .with_max_steps(500)
    .with_bounds(glam::Vec2::new(-5.0, -4.0), glam::Vec2::new(5.0, 4.0)),
    Vec3::ZERO,
);
```

Helper functions for starting points:

```rust
// Points along a line
line_start_points(Vec2::new(-3.0, 0.0), Vec2::new(3.0, 0.0), 10)?

// Points in a circle
circle_start_points(Vec2::ZERO, 2.0, 12)?
```

Grid axes and line seeds require at least two samples; circle seeds require at least one. Invalid
counts, ranges, bounds, integration steps, seed points, or non-finite vector-field output return a
`ValidationError` instead of producing invalid geometry. `StreamLines::try_new` validates manually
assembled seed lists immediately; projection repeats validation so direct public-field mutation is
also caught.

Use `StreamLines` when you want viewers to read global flow structure instead of per-sample arrows.

In many teaching scenes, `VectorField` and `StreamLines` work well together:

- `VectorField` explains what happens locally
- `StreamLines` explains the overall flow

## 2D Graph Vs 3D Surface

Prefer 2D first when:

- the idea is fundamentally a function or planar curve
- labels and narration matter more than immersion
- the extra camera complexity does not teach anything

Move to 3D when:

- the geometry really lives in 3D
- depth or topology is part of the idea
- camera motion helps understanding instead of just looking impressive

## Sampling And Resolution

- increase `with_samples(...)` for smoother curves
- increase surface sample counts for cleaner wireframes and textures
- keep counts moderate while authoring, then raise them for final export if needed
- overly dense sampling makes preview slower without always improving clarity

## Best Animation Pairings

- `draw()` for graphs, curves, and many line-based surfaces
- `appear()` for scatter plots or instant comparison scenes
- `fade_to(...)` for de-emphasizing supporting layers
- camera animation for `ParametricCurve3D` and `ParametricSurface`

## Gotchas

- if a 3D graph feels confusing, the camera is often the problem rather than the graph
- dense fields and surfaces can become visually noisy very quickly
- if a plot needs many labels and equations, simplify the graph before adding more annotation

## Related Docs

- [Camera](../camera)
- [Animations](../animations)
- [Examples](../examples/showcase)
- [Math](./math)
