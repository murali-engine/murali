---
sidebar_position: 4
---

# Tables

Table tattvas live under `murali::frontend::collection::table`.

## Table

Use `Table` when you want structured rows and columns with labels, titles, and line-by-line reveal behavior.

```rust
use murali::frontend::collection::table::Table;

scene.add_tattva(
    Table::new(vec![
        vec!["Layer", "Width", "Activation"],
        vec!["Input", "784", "-"],
        vec!["Hidden", "256", "ReLU"],
        vec!["Output", "10", "Softmax"],
    ]),
    Vec3::ZERO,
);
```

## Common customizations

Tables expose a `TableConfig` for the shared visual styling:

- `h_buff`, `v_buff` - spacing between cells
- `line_color`, `line_thickness` - grid styling
- `text_color`, `text_height` - text styling
- `include_outer_lines` - whether the outer border is drawn
- `background_color` - optional cell fill
- `labels_inside` - whether row and column labels are placed inside the grid

Tables can also include:

- row labels
- column labels
- a title
- top or bottom title placement via `TableTitlePosition`

## When to use it

Use tables for:

- model architecture summaries
- comparison slides
- datasets and metrics
- teaching visuals where row or column labeling matters

Pair especially well with staged write-on or progressive reveal animations.
