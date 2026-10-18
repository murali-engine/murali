---
sidebar_position: 6
---

# Layout

Layout helpers live under `murali::frontend::collection::layout`. They arrange groups of existing tattvas relative to each other.

Note: these are not tattvas themselves — they operate on `TattvaId`s already in the scene.

## HStack

Arranges tattvas horizontally left-to-right with a fixed gap.

```rust
use murali::frontend::collection::prelude::HStack;

let a = scene.add_tattva(Circle::new(0.5, 32, red), Vec3::ZERO);
let b = scene.add_tattva(Square::new(1.0, blue), Vec3::ZERO);
let c = scene.add_tattva(Circle::new(0.5, 32, green), Vec3::ZERO);

HStack::new(vec![a, b, c], 0.3).apply(&mut scene);
```

`apply` calls `scene.next_to` and `scene.align_to` internally to position each item to the right of the previous one, aligned on their bottom edges.

## VStack

Arranges tattvas vertically top-to-bottom with a fixed gap.

```rust
use murali::frontend::collection::prelude::VStack;

VStack::new(vec![title_id, body_id, footer_id], 0.2).apply(&mut scene);
```

Items are aligned on their left edges.

## Scene layout methods

These are available directly on `Scene` and are used by HStack/VStack internally:

```rust
// Place `id` to the right of `reference` with `gap` spacing
scene.next_to(id, reference, Direction::Right, gap);

// Align `id` to match `reference` on the given anchor
scene.align_to(id, reference, Anchor::Down);

// Push `id` to the screen edge with padding
scene.to_edge(id, Direction::Up, 0.35);
```

All layout operates in 2D (X/Y only). Z is not affected.
