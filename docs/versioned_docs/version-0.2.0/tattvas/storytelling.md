---
sidebar_position: 7
---

# Storytelling

Storytelling tattvas live under `murali::frontend::collection::storytelling`. They are higher-level constructs for building step-by-step explanations and narrative animations.

## Stepwise

`Stepwise` renders a flow of nodes that reveal progressively — useful for algorithm walkthroughs, decision trees, and step-by-step proofs.

It is a tattva: it implements `Project` and `Bounded`, so it's added to the scene like any other shape.

### Building a Stepwise flow

Use `Stepwise::from_script(...)` to define nodes and connections through the script DSL:

```rust
use murali::frontend::collection::storytelling::stepwise::{
    Stepwise, StepwiseLayout,
};

let flow = Stepwise::from_script(|s| {
    let a = s.step("Input Layer");
    let b = s.step("Processing");
    let c = s.step("Output");

    s.connect(a, b);
    s.connect(b, c);
})
.with_layout(StepwiseLayout::horizontal(2.0));
```

Each call to `s.step()` returns an index. `s.connect(from, to)` registers a directed edge between two steps.

### Adding to the scene

```rust
let id = scene.add_tattva(flow, Vec3::ZERO);
```

`StepwiseLayout::horizontal(spacing)` places nodes left-to-right with the given gap between them. `StepwiseLayout::vertical(spacing)` places them top-to-bottom.

### Animating reveal

Drive the reveal phase with a timeline animation:

```rust
timeline.animate(id)
    .at(0.0)
    .for_duration(3.0)
    .ease(Ease::InOutQuad)
    .propagate_to(1.0)
    .spawn();
```

`propagate_to(1.0)` drives `progress` from 0 to 1, revealing nodes and edges in sequence.

### Animating signal flow

A second animation drives the signal dot along the route:

```rust
timeline.animate(id)
    .at(4.0)
    .for_duration(5.0)
    .ease(Ease::Linear)
    .signal_to(1.0)
    .spawn();
```

`signal_to(1.0)` drives `signal_progress` independently from the reveal phase. The signal dot travels the sequence defined in the model.

### Custom sequences

By default the sequence follows the order nodes were connected. You can override it explicitly, including cycles:

```rust
let flow = Stepwise::from_script(|s| {
    let a = s.step("A");
    let b = s.step("B");
    let c = s.step("C");

    s.connect(a, b);
    s.connect(b, c);

    // Signal visits A → B → C → B → C
    s.with_sequence(vec![a, b, c, b, c]);
});
```

### Back-path routing

Connections can go backwards. Use `.route()` to describe the path shape:

```rust
use murali::frontend::collection::storytelling::stepwise::{
    model::Direction,
    Stepwise,
};

let flow = Stepwise::from_script(|s| {
    let a = s.step("Start");
    let b = s.step("Middle");
    let c = s.step("End");

    s.connect(a, b);
    s.connect(b, c);

    // Back edge: C → A, routed above the diagram
    // Two Left steps because C is two nodes to the right of A
    s.connect(c, a).route(vec![
        Direction::Up,
        Direction::Left,
        Direction::Left,
    ]);

    s.with_sequence(vec![a, b, c, a, b, c]);
});
```

Each `Direction::Left` or `Direction::Right` moves exactly one node in the layout. The engine automatically selects the correct entry face on the target node based on where the path arrives.

For a deeper explanation of how routing works internally, see [Stepwise Internals](../feature-internals/stepwise).
