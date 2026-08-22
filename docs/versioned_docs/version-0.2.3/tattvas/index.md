---
sidebar_position: 3
---

# Tattvas

A tattva is any object that can be placed, animated, and rendered in a scene. The word comes from Sanskrit and means "element" or "essence".

Every tattva is added to the scene with `add_tattva(shape, position)` which returns a `TattvaId` for later reference:

```rust
let id = scene.add_tattva(Circle::new(1.0, 64, Vec4::new(0.2, 0.6, 1.0, 1.0)), Vec3::ZERO);
```

This is the preferred authoring API. The lower-level `scene.add(...)` path is intended for internal or advanced cases where you already have a fully constructed tattva object.

Before diving into a specific family, see [Common Tattva Properties](properties) for the shared scene-level behavior that most tattvas support.

Tattvas are organized into categories based on their purpose:

- [Primitives](primitives) — basic shapes such as circle, square, rectangle, line, polygon, arrow, path, cube, and Prop3D
- [Text](text) — label, latex, typst, and code block
- [Tables](tables) — structured grids with labels and titles
- [Composite](composite) — cards, axes, number plane, 3D axes, and beta components
- [Opening (Beta)](opening) — configurable 3D title, particle dissolve, and tagline choreography
- [Graphs](graphs) — function graphs, scatter plots, parametric curves, surfaces, vector fields, and stream lines
- [Math](math) — equation layout, matrices, and vector formulas
- [Layout](layout) — stacking and relative arrangement helpers
- [Storytelling](storytelling) — stepwise reveal for guided explanations
- [AI Diagrams](ai) — context windows, next-token distributions, KV caches, normalization, semantic tensors and operations, trace ingestion, attention and transformer views, neural diagrams, signal flows, and AI indicators
- [Utility](utility) — traced path, screenshot marker, and other scene helpers

## What should be covered

As a rule, the docs should cover all shipped tattva families, even when some pages begin as lightweight indexes.

A good content split is:

- family page for orientation
- concrete tattva entries for constructors and notable fields
- one shared page for common tattva properties

That keeps the docs discoverable without repeating the same transform and visibility guidance on every page.
