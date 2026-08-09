---
sidebar_position: 5
---

# Math

Math tattvas live under `murali::frontend::collection::math`. These are higher-level constructs for mathematical notation.

Use this family when the scene needs mathematical structure rather than just rendered text. If the visual meaning depends on equation parts, matrix cells, or continuity across transforms, these tattvas are usually a better fit than plain `Latex` or `Typst`.

For general text choices, read [Text](./text) first. For motion and continuity, also see [Animations](../animations) and [Examples](../examples/showcase).

## Quick Decision Guide

| Need | Use | Why |
|---|---|---|
| Simple static equation rendering | `Latex` or `Typst` text tattvas | Best when you only need rendered math |
| Equation with individually styled parts | `EquationLayout` | Best for emphasis and authored part-level control |
| Matrix with cell-level styling | `Matrix` | Best for row/column/cell attention |
| Formula as morphable vector geometry | `VectorTypstEquation` or `VectorLatexEquation` | Best for continuity-driven math animation |

## EquationLayout

Renders a sequence of individually colored and animatable equation parts, laid out horizontally.

```rust
use murali::frontend::collection::math::equation::{EquationLayout, EquationPart};
use murali::colors::*;
use murali::positions::*;

scene.add_tattva(
    EquationLayout::new(
        vec![
            EquationPart::new("E").with_color(GOLD_B),
            EquationPart::new("=").with_color(WHITE),
            EquationPart::new("mc²").with_color(Vec4::new(0.4, 0.8, 1.0, 1.0)),
        ],
        0.5, // world_height
    ),
    Vec3::ZERO,
);
```

Each `EquationPart` can have:
- `.with_color(Vec4)` — individual color
- `.with_key(str)` — identity key for equation continuity animations
- `.with_opacity(f32)` — per-part opacity
- `.with_scale(f32)` — per-part size scale
- `.with_offset(Vec3)` — per-part positional nudge

Parts are laid out left-to-right, centered as a group. Gap between parts defaults to `world_height * 0.35`.

Use `EquationLayout` when:

- you want to color or emphasize specific tokens
- you want continuity between equation versions
- the expression is better thought of as authored parts than as one compiled string

### Continuity Keys

`EquationPart::with_key(...)` matters when two equations should preserve identity across a morph-like transition.

```rust
EquationPart::new("x").with_key("lhs_x")
```

If you do not provide a key, Murali falls back to a generated continuity key based on the text and position. Explicit keys are safer when:

- the same symbol appears multiple times
- you are reordering terms
- you want predictable part matching across steps

## Matrix

Renders a 2D matrix with bracket notation.

```rust
use murali::frontend::collection::math::matrix::Matrix;

scene.add_tattva(
    Matrix::new(
        vec![
            vec!["1", "2", "3"],
            vec!["4", "5", "6"],
            vec!["7", "8", "9"],
        ],
        0.4, // cell_height
    ),
    Vec3::ZERO,
);
```

Individual cells can be styled:

```rust
let mut m = Matrix::new(entries, 0.4);

// Highlight a cell
m.cell_mut(1, 1).unwrap().highlight = Some(GREEN_B);

// Color a cell
m.cell_mut(0, 0).unwrap().color = Vec4::new(1.0, 0.5, 0.2, 1.0);

// Assign a key for animation continuity
m.cell_mut(0, 0).unwrap().key = Some("a11".to_string());
```

Bracket style is configurable:

```rust
let mut m = Matrix::new(entries, 0.4);
m.bracket_color = WHITE;
m.bracket_thickness = 0.03;
m.h_gap = 0.5;  // horizontal gap between columns
m.v_gap = 0.2;  // vertical gap between rows
```

Use `Matrix` when:

- row/column structure matters visually
- you want to highlight specific entries
- matrix continuity is part of the animation story

### Cell Keys And Continuity

Matrix cells also support continuity keys:

```rust
m.cell_mut(0, 0).unwrap().key = Some("a11".to_string());
```

This is worth doing when matrix entries move, persist, or transform across multiple scenes or states.

## Vector equations

Use vector equations when you want formulas represented as morphable paths rather than rasterized text.

Prefer the authored scene helpers:

```rust
use murali::frontend::collection::math::equation::VectorTypstEquation;

let handle = scene.add_vector_formula_typst(
    VectorTypstEquation::new("$(a + b)^2$", 1.2)
        .with_color(Vec4::new(0.4, 0.7, 1.0, 1.0)),
);
```

`add_to_scene(...)` and the `scene.add_vector_formula_*` helpers are the primary authoring APIs.
The lower-level `spawn(...)` methods still exist for advanced/internal use when you explicitly want raw tattva IDs.

Use vector formulas when:

- you want high-quality morphing between formulas
- path identity matters more than “just render this expression”
- the formula itself is a geometric actor in the scene

Prefer plain `Latex` or `Typst` text when:

- the formula is mostly static
- you do not need continuity-aware transforms
- faster authoring matters more than geometry-level control

## Equation Layout Vs Vector Formula Vs Typst Math

Use `EquationLayout` when you want authored part-level control over color, opacity, and matching.

Use vector formulas when you want geometry-based morphing between formulas.

Use `Typst` or `Latex` text tattvas when the formula is mostly static and you care more about rendering convenience than part-level or path-level control.

## Best Animation Pairings

- `reveal_text()` for staged mathematical exposition
- `fade_to(...)` for supporting equations or de-emphasis
- continuity-aware morph workflows for vector formulas and authored equation transitions

## Gotchas

- continuity falls apart quickly if repeated parts do not have stable keys
- matrix cells with similar text benefit from explicit keys during transitions
- vector formulas are more powerful, but also heavier than plain text math

## Related Docs

- [Text](./text)
- [Animations](../animations)
- [Examples](../examples/showcase)
