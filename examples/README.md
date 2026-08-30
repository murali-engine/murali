This folder contains the reference runnable examples for Murali.

Each example is meant to teach one idea clearly, and together they form the reference example surface for the crate.

## Active Examples

`cargo run --example hello_shapes`
A first scene with a square, circle, rectangle, and polygon placed by hand.

`cargo run --example layout_and_groups`
Placement helpers like `next_to`, `align_to`, `HStack`, `VStack`, and `Group.move_to`.

`cargo run --example scene_view`
A hand-built transformer running on its own looping child timeline while the complete view docks,
tilts, scales, and returns inside a separate parent composition.

`cargo run --example kavriq_opening`
Extruded, bone-textured capital letters fall through a perspective scene, settle into KAVRIQ,
burst into glyph-shaped particles, and dissolve to reveal a stable tagline end card. It demonstrates
the beta `Opening` composite and keeps the title, tagline, font, texture, colors, particle count,
and timing choices as Rust constants at the top of the script.

```bash
cargo run --release --example kavriq_opening
```

`cargo run --example opening_scene_view`
The beta `Opening` runs in a full-frame perspective `SceneView`, including its own background. The
parent remains orthographic and reveals an ongoing inference-flow scene as the complete opening
view fades away. This is the reference for continuing into Murali-authored content without changing
the parent camera or flattening both sequences onto one projection.

`cargo run --example portrait_video`
An intentional 9:16 composition demonstrating portrait preview, export, and frame-relative layout.

`cargo run --example style_and_paths`
Fill, stroke, dashes, arrows, and one authored path in a single calm scene.

`cargo run --example motion_basics`
Move, scale, rotate, and fade with small readable easing examples.

`cargo run --example text_animation`
Typewriter text, centered reveal, indicate pulses, and one simple path draw/undraw.

`cargo run --example chat_input_box`
The beta `ChatInputBox` composite with left/right bottom tips, optional send button, and
typewriter-ready prompt text.

`cargo run --example context_window`
A semantic model-context view with role-tagged instructions, history, retrieval, tool output,
token-budget usage, and explicit history truncation.

`cargo run --example next_token_distribution`
A computed next-token decision showing logits, model probabilities, temperature, top-k and top-p
filtering, a deterministic sample, and the selected token.

`cargo run --example kv_cache`
A semantic key/value cache backed by two token-feature tensors, with empty future slots and
deterministic row-by-row cache growth.

`cargo run --example normalization`
A computed LayerNorm view showing residual values before and after normalization together with the
mean and standard deviation used for each token.

`cargo run --example code_blocks`
Syntax-highlighted code snippets placed and paced as first-class scene elements.

`cargo run --example graphs_2d`
A number plane, axes, one function graph, sampled points, and basic axis labels.

`cargo run --example latex_and_typst`
Static LaTeX and Typst rendering plus one compact vector morph sequence.

`cargo run --example equation_and_matrix_animation`
Equation continuity and matrix highlight steps in one staged math-semantics example.

`cargo run --features experimental --example linear_algebra_vectors`
Vector arrows, labels, coordinate readouts, and a compact feature-list readout.

`cargo run --features experimental --example linear_algebra_span`
Linear combinations, span regions, and basis-vector intuition on a number plane.

`cargo run --features experimental --example linear_algebra_dot_product`
Angle arcs, projection shadows, orthogonality marking, and dot/cosine readout.

`cargo run --features experimental --example linear_algebra_basis_change`
One vector shown against the standard grid and a custom basis grid, with both coordinate readouts.

`cargo run --features experimental --example linear_algebra_matrix_transform`
A transformed grid, colored basis-vector images, a linked 2x2 matrix panel, and one sample input
vector mapped to `Ax`.

`cargo run --features experimental --example linear_algebra_matrix_vector`
A compact `A x = b` computation view beside the geometric input and output vectors.

`cargo run --features experimental --example linear_algebra_column_combination`
Columns of `A` scaled by the entries of `x`, summed geometrically, with an optional target `b`.

`cargo run --features experimental --example linear_algebra_determinant`
Three determinant cases: area scaling, orientation flip, and collapse to a line.

`cargo run --features experimental --example linear_algebra_composition`
Side-by-side transform composition showing `A` then `B` as `BA`, with an order comparison.

`cargo run --features experimental --example linear_algebra_transform_order_scene_view`
SceneView-based transform-order demo: play `A` then `B`, shrink it aside, then compare with `B`
then `A`.

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

`cargo run --example prop3d_glb`
A local faceted `.glb` prop loaded as a `Prop3D`, then moved and rotated with ordinary timeline
transforms.

`cargo run --example prop3d_gltf`
A loose `.gltf` apple prop loaded with its sibling `.bin` file, then moved and rotated with ordinary
timeline transforms.

`cargo run --example model_inspector -- demo-apple`
A quick 3D asset inspection scene that automatically resolves, centers, frames, and continuously
rotates a GLB/GLTF model. Use `--help` for scale, orientation, camera, and preview controls.

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

### Rust Examples

Clone the engine repository and run Rust examples locally:

```bash
cargo run --example hello_shapes
```

Preview or export marked subsets:

```bash
./preview_all.sh --tag linear-algebra --auto
./export_all.sh --tag linear-algebra --release
./preview_all.sh --example linear_algebra_span --auto
./preview_all.sh --tag linear-algebra --dry-run
./preview_all.sh --list-tags
```

The linear-algebra examples use the feature-gated experimental API. Running one directly requires
`--features experimental`; the helper scripts add that feature automatically for examples tagged
`linear-algebra`.

Mark an example by adding a top-level comment:

```rust
// murali-example-tags: linear-algebra,math,reference
```

Use this README as the full catalog, and use the top-level project README for a smaller curated subset.

### Python Examples

Python examples are split by package boundary. The engine repository keeps a small smoke example in
`python/examples`, while broader companion examples live in the separate `murali-kit` repository.

From this engine repository, install the local Python extension and run the smoke example:

```bash
python3 -m venv .venv
.venv/bin/python -m pip install maturin
.venv/bin/maturin develop --features python
.venv/bin/python python/examples/hello_shapes.py
```

From a sibling `murali-kit` checkout, create the kit environment and point it at the adjacent local
engine checkout:

```bash
cd ../murali-kit
python3 -m venv .venv
.venv/bin/python -m pip install -r requirements-local.txt
source .venv/bin/activate
python preview_all.py --list
python examples/hello_shapes.py
```

On supported platforms, install the released kit package with:

```bash
python3 -m pip install murali-kit==0.1.0
```

`murali-kit` depends on the published `murali-engine` package instead of requiring a local checkout.
Prebuilt `murali-engine` wheels cover macOS, Linux, and Windows on common architectures. The copied
Rust examples in `murali-kit/rust-reference/examples` are migration references; new Python examples
should be authored directly in `murali-kit/examples`.
