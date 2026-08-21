---
sidebar_position: 2
---

# Neural Network Diagram Internals

This page explains the internal structure of `NeuralNetworkDiagram` for contributors.

For user-facing documentation, this feature should be explained through the AI/composite/graph tattva docs rather than here.

## What It Is

`NeuralNetworkDiagram` is a specialized projected tattva for rendering a layered feed-forward network diagram.

It combines:

- a compact semantic model of network layers
- simple deterministic layout rules
- direct projection into edges, nodes, optional labels, and optional activation visuals

Like other Murali tattvas, it is still just:

- a frontend object
- implementing `Project`
- implementing `Bounded`

So its internal complexity comes from how it organizes and projects its own data, not from any special engine pathway.

## Core Data Model

The key fields are:

- `layers`
- spacing and sizing controls
- active/inactive styling
- optional `layer_labels`
- optional `activation`
- `inactive_nodes`

### `layers`

`layers: Vec<usize>` is the main semantic input. Each element gives the number of nodes in that layer.

This is intentionally small and opinionated. The current feature is not a generic computation graph. It is a layered neural-network-style diagram.

That design keeps layout and projection simple.

### Styling Fields

The diagram keeps explicit visual parameters such as:

- `layer_spacing`
- `node_spacing`
- `node_radius`
- `node_color`
- `edge_color`
- `edge_thickness`
- inactive variants of node and edge colors

This means the feature does not depend on hidden theme state internally. Themes can still be applied through helpers such as `AiUnderTheHoodTemplates`, but the diagram itself remains explicit.

### Active vs Inactive Nodes

Inactive nodes are tracked as:

```rust
HashSet<(usize, usize)>
```

This is a clean representation because it:

- is easy to query from projection
- does not require duplicating the layer structure
- keeps the base authored model small

The current implementation uses active/inactive state not only for nodes but also for whether an edge should be treated as active.

## Layout Strategy

The layout logic is very direct:

- each layer gets an x-position from `layer_x`
- each node gets a y-position from `node_y`
- the whole structure is centered around the origin

This gives the diagram a stable deterministic layout without introducing a separate layout engine.

That is the right tradeoff for this feature because:

- layered network diagrams have a very regular structure
- contributors can reason about placement quickly
- bounds are straightforward to compute

## Path Utilities

The diagram also exposes helper methods such as:

- `node_position`
- `path_points`
- `all_path_points`
- `indication_path_points`

These are important because they make the diagram useful for higher-level animation and teaching flows, not just static rendering.

In particular, `path_points` and `all_path_points` let other systems derive semantic routes through the network. That makes the diagram composable with effects such as traced paths, signal overlays, or author-defined highlighting logic. Because `all_path_points` is exhaustive, its result grows as the product of the layer widths. Built-in indication instead uses `indication_path_points`, which selects at most 64 deterministic active routes and keeps per-frame work bounded.

## Projection Strategy

Projection happens in two broad phases:

1. edges between adjacent layers
2. nodes and optional overlays per layer

### Edges

For each adjacent pair of layers, the diagram projects a full bipartite connection set.

That means every node in one layer connects to every node in the next layer.

This is a deliberate modeling decision:

- it matches the visual expectation for a dense feed-forward network
- it keeps projection straightforward
- it avoids needing a separate explicit edge list for the current feature

### Nodes

Each node is projected at its computed position with active or inactive styling depending on the inactive set.

This keeps the visual state derived from the semantic model rather than stored separately as projected artifacts.

### Activation Visuals

`ActivationFunc` allows the diagram to project a small symbolic visual between layers.

Current options are:

- `None`
- `ReLU`
- `Sigmoid`

These are not computational graph semantics. They are purely visual annotations rendered during projection.

That distinction is important if you want to evolve the feature later.

## Bounds

`Bounded` is implemented so the diagram can participate correctly in layout helpers and scene composition.

Because the layout is regular and centered, bounds can be computed from:

- number of layers
- widest layer
- spacing parameters
- node radius

This is much simpler than bounds for more free-form graph layouts.

## Templates And Theming

`AiUnderTheHoodTemplates::neural_network` is a thin theming helper around the base diagram.

That separation is a good design choice:

- the diagram remains a reusable feature-level tattva
- themed presets stay out of the core implementation
- contributors can change rendering logic without entangling it with one theme

So when changing this feature, it is useful to keep the distinction clear between:

- diagram structure
- default styling
- external themed presets

## Current Tradeoffs And Limitations

The current design is intentionally simple. Important limitations include:

- it models dense adjacent-layer connectivity, not arbitrary sparse graphs
- an edge is active only when both of its endpoint nodes are active
- activation visuals are symbolic annotations, not first-class graph elements
- there is no separate animation model inside the feature itself

These are good constraints to be aware of before adding more expressive behavior.

## Good Contributor Questions

If you extend `NeuralNetworkDiagram`, useful questions include:

- is this still a layered neural-network diagram, or is it becoming a generic graph system?
- should this be represented in semantic data, or only in projection?
- should this feature own the animation behavior, or should external animations drive its state?
- does this belong in the base diagram or in a template/helper layer?

Those questions help prevent this feature from quietly turning into a second graph framework.
