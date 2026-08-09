This folder contains the reference runnable examples for Murali.

Each example is meant to teach one idea clearly, and together they form the reference example surface for the crate.

## Active Examples

`cargo run --example hello_shapes`
A first scene with a square, circle, rectangle, and polygon placed by hand.

`cargo run --example layout_and_groups`
Placement helpers like `next_to`, `align_to`, `HStack`, `VStack`, and `Group.move_to`.

`cargo run --example style_and_paths`
Fill, stroke, dashes, arrows, and one authored path in a single calm scene.

`cargo run --example motion_basics`
Move, scale, rotate, and fade with small readable easing examples.

`cargo run --example text_animation`
Typewriter text, centered reveal, indicate pulses, and one simple path draw/undraw.

`cargo run --example code_blocks`
Syntax-highlighted code snippets placed and paced as first-class scene elements.

`cargo run --example graphs_2d`
A number plane, axes, one function graph, sampled points, and basic axis labels.

`cargo run --example latex_and_typst`
Static LaTeX and Typst rendering plus one compact vector morph sequence.

`cargo run --example equation_and_matrix_animation`
Equation continuity and matrix highlight steps in one staged math-semantics example.

`cargo run --example tables`
One readable table that writes in, stays on screen briefly, and unwrites cleanly.

`cargo run --example curves_3d`
One parametric space curve with 3D axes and a few perspective camera frames.

`cargo run --example surfaces_3d`
One introductory parametric surface with a progressive reveal and a few camera frames.

`cargo run --example wireframe_surfaces`
One wireframe saddle surface where the mesh itself explains the curvature.

`cargo run --example textured_surface`
One textured globe that teaches image wrapping on a parametric surface.

`cargo run --example streamlines`
Seeded flow trajectories through one field, taught without mixing in vector arrows.

`cargo run --example force_fields`
Moving positive and negative charges with a field that updates continuously in response.

`cargo run --example particles`
One orbital particle belt with gentle camera framing and continuous evolution.

`cargo run --example traced_paths`
One rolling wheel whose rim point leaves behind a traced cycloid.

`cargo run --example neural_networks`
One network with a clean forward pass and a second signal playback variation.

`cargo run --example transformer_attention`
Token sequence, attention matrix, and transformer block composition in one staged AI explainer.

`cargo run --example tensor_semantics`
Computed self-attention stages with named axes, stable element identity, and semantic selection.

`cargo run --example tensor_operations`
Semantic broadcasting, named-axis split/merge, and explicit reshape in one staged tensor lesson.

`cargo run --example tensor_slicing`
Semantic projection of rank-4 activations into an animated token-by-feature head view.

`cargo run --example self_attention_lesson`
One JSON-backed computed path from stable token IDs through Q/K/V, attention, residuals, logits,
and sampling.

`cargo run --example stepwise_storytelling`
One narrative flow that reveals step by step and then replays a routed feedback journey.

`cargo run --example murali_logo`
The Murali brand mark as a visual reference study built from authored geometry and guide structure.

`cargo run --example murali_logo_transparent`
The Murali brand mark prepared for transparent PNG export with example-level toggles for frame visibility.

`cargo run --example fourier_formula_trace`
An advanced Fourier-series demo where ranked coefficients become epicycles that reconstruct a Typst pi outline.

`cargo run --example map_projection_morph`
An advanced demo where the Earth surface image bends through several classic map projections.

## Principles

- one example should answer one learning question
- one viewport should tell one story
- examples should be named by teaching intent, not by vague showcase language
- docs should only point to examples that actually exist

## Running Examples

Clone the repository and run examples locally:

```bash
cargo run --example hello_shapes
```

Use this README as the full catalog, and use the top-level project README for a smaller curated subset.
