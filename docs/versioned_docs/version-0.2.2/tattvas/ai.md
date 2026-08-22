---
sidebar_position: 9
---

# AI Diagrams

AI diagram tattvas live under `murali::frontend::collection::ai`.

This family is for higher-level teaching visuals and model diagrams rather than generic geometric graphing.

## Included tattvas

- `AttentionMatrix` - heatmap-style token-to-token attention grid
- `ContextWindow` - ordered, role-aware model context with token budgets and explicit truncation
- `KvCacheView` - semantic key/value cache occupancy backed by token-feature tensors
- `TensorSnapshot`, `TensorSlice`, and `TensorView` - validated arbitrary-rank tensor state, explicit semantic slicing, and rank-2 visual projections
- `AiTrace` - versioned JSON ingestion for tokens, tensors, model metadata, and timestamped teaching events
- `SignalFlow` - animated pulse moving through one or more paths
- `NeuralNetworkDiagram` - layered node-and-edge network diagram
- `TokenSequence` - ordered token visualization
- `TransformerBlockDiagram` - semantic encoder, decoder, or custom transformer composition
- `DecisionBoundaryPlot` - classifier-style 2D decision region plot
- `MuraliAiIndicator` - animated Murali-branded AI activity indicator with optional loop-until timing
- `NextTokenDistribution` - computed temperature, top-k, top-p, and deterministic sampling view
- `NormalizationView` - computed LayerNorm or RMSNorm before/after view with per-group statistics
- `AgenticFlowChart` - routed flow-chart style diagrams (available, but not currently being actively evolved; prefer `Stepwise` for new storytelling-oriented flows)

`DecisionBoundaryPlot` extracts the classifier's zero contour with interpolated marching squares.
Disconnected regions remain separate, and ambiguous saddle cells use a deterministic center-sign
rule instead of joining near-zero samples in scan order.

## Examples

`MuraliAiIndicator` is useful as a branded AI activity mark, loading state, intro bumper, or background element. It is a composite helper: it adds several primitives to the scene and returns `MuraliAiIndicatorIds` so the group can be hidden, revealed, and looped from a timeline.

```rust
use murali::frontend::collection::ai::{
    MURALI_AI_INDICATOR_DURATION,
    MuraliAiIndicator,
};

let mut scene = Scene::new();
let mut timeline = Timeline::new();
let loop_until = 12.0_f32.max(MURALI_AI_INDICATOR_DURATION);

let indicator = MuraliAiIndicator::new()
    .with_subtitle("agent is thinking")
    .add_to_scene(&mut scene, Vec3::ZERO);

indicator.hide_all(&mut scene);
indicator.animate(&mut timeline, loop_until);
timeline.wait_until(loop_until);
scene.play(timeline)?;
```

For video exports, "infinite" animation is represented as a finite `loop_until` value. The intro plays once, then the active signal and pulse cycle repeats until that timestamp.

`AttentionMatrix` is useful when values matter cell-by-cell:

```rust
use murali::frontend::collection::ai::attention_matrix::AttentionMatrix;

scene.add_tattva(
    AttentionMatrix::new(
        vec![
            vec![1.0, 0.5, 0.1],
            vec![0.3, 0.9, 0.2],
            vec![0.1, 0.4, 0.8],
        ],
        Some(vec!["The".into(), "cat".into(), "sat".into()]),
    ),
    Vec3::ZERO,
);
```

`TokenSequence` can share stable IDs with a tensor axis instead of treating display strings as
identity:

```rust
let token_axis = TensorAxis::with_elements(
    "query",
    "Tokens",
    [("token.0", "AI"), ("token.1", "learns")],
);
let tokens = TokenSequence::try_from_axis(&token_axis, 0.22)?;
```

`ContextWindow` shows exactly what is assembled for one model invocation. Each block has stable
identity, a semantic role, an original token count, and a retained token count. Truncation is never
inferred silently: when tokens are omitted, the author must name whether they were removed from the
start or end of that block.

```rust
use murali::prelude::{
    ContextBlock, ContextBlockRole, ContextTruncation, ContextWindow,
};

let context = ContextWindow::try_new(
    vec![
        ContextBlock::new(
            "instructions",
            "Core instructions",
            ContextBlockRole::System,
            620,
        ),
        ContextBlock::new(
            "history",
            "Conversation history",
            ContextBlockRole::User,
            4_900,
        )
        .with_preview("Earlier turns and decisions")
        .truncated_to(2_700, ContextTruncation::FromStart),
        ContextBlock::new(
            "retrieval",
            "Retrieved documents",
            ContextBlockRole::Retrieved,
            1_850,
        ),
    ],
    8_192,
)?;

assert_eq!(context.used_tokens(), 5_170);
assert_eq!(context.available_tokens(), 3_022);
scene.add_tattva(context, Vec3::ZERO);
```

Available roles are `System`, `User`, `Assistant`, `Tool`, and `Retrieved`. Validation rejects
empty or duplicate block IDs, zero token counts, retained counts larger than their source blocks,
implicit truncation, and context usage beyond the declared budget. The runnable `context_window`
example presents a complete assembled prompt with visibly trimmed conversation history.

For new AI explainers, prefer `TensorSnapshot` when the values have model meaning beyond their
appearance. The snapshot owns shape, values, named axes, and stable element identity. `TensorView`
is a rank-2 projection of that state, so styling does not erase semantics:

```rust
use murali::prelude::{TensorAxis, TensorSelector, TensorSnapshot, TensorView};

let axes = vec![
    TensorAxis::with_elements(
        "query",
        "Queries",
        [("token.0", "The"), ("token.1", "model")],
    ),
    TensorAxis::with_elements(
        "key",
        "Keys",
        [("token.0", "The"), ("token.1", "model")],
    ),
];
let logits = TensorSnapshot::try_new(
    "layer.3.head.1.logits",
    vec![2, 2],
    vec![1.2, -0.4, 0.3, 0.9],
    axes.clone(),
)?;
let attention_weights = TensorSnapshot::try_new(
    "layer.3.head.1.logits",
    vec![2, 2],
    vec![1.0, 0.0, 0.35, 0.65],
    axes,
)?;

let view = TensorView::try_new(logits)?
    .with_selection(TensorSelector::axis("query", 1))?;
let tensor_id = scene.add_tattva(view, Vec3::ZERO);

timeline
    .animate(tensor_id)
    .at(1.0)
    .for_duration(1.0)
    .tensor_to(attention_weights)
    .spawn();

timeline
    .animate(tensor_id)
    .at(2.0)
    .for_duration(0.5)
    .tensor_select(vec![TensorSelector::axis_element("query", "token.1")])
    .spawn();
```

`TensorSnapshot` supports arbitrary rank. `TensorSlice` fixes named axes by semantic element ID or
retains an explicitly ordered subset. `try_project_2d` requires exactly two remaining named axes,
so a batch, head, token, or feature dimension is never flattened away implicitly:

```rust
use murali::prelude::{TensorSlice, TensorView};

let head_view = activations.try_project_2d(
    "encoder.head.view",
    "token",
    "feature",
    &TensorSlice::new()
        .at("batch", "batch.0")
        .at("head", "head.1")
        .elements("feature", ["feature.0", "feature.2"]),
)?;
let view = TensorView::try_new(head_view)?;
```

Fixed axes are removed from the result. Subset axes keep their labels and stable element IDs in
the requested order; unmentioned axes remain unchanged. Invalid axes, unknown or duplicate
elements, duplicate rules for one axis, scalar-only slices, and projections retaining anything
other than two axes return `ValidationError`.

`tensor_to` matches cells by tensor, axis, and axis-element identity; reordered cells move,
new cells fade in, and removed cells fade out. Both value and selection transitions are reversible
under timeline seeking.

Attention stages can be computed from validated snapshots instead of authored as unrelated
matrices. Matrix multiplication aligns the contracted axis by semantic element ID, even when its
storage order differs:

```rust
let keys_transposed = keys.try_transpose_2d("layer.3.head.1.k_transposed")?;
let dot_products = queries.try_matmul(
    &keys_transposed,
    "layer.3.head.1.attention",
)?;
let scaled = dot_products.try_scaled((queries.shape[1] as f32).sqrt())?;
let masked = scaled.try_causal_masked(-20.0)?;
let attention_weights = masked.try_softmax("key")?;
```

Scaling, masking, and softmax preserve tensor identity, so their results can be passed directly to
successive `tensor_to` animations. Softmax uses max subtraction for numerical stability.

Shape and elementwise operations retain named-axis meaning as well. Reshape requires explicit
target axes, split and merge operate on an axis ID, and elementwise operations align reordered
elements by ID before applying broadcasting:

```rust
use murali::prelude::TensorElementwiseOp;

let heads = activations.try_reshape(
    "layer.3.heads",
    vec![head_axis, token_axis, channel_axis],
)?;
let parts = heads.try_split("head", &[4, 4], &["local_heads", "global_heads"])?;
let heads_again = TensorSnapshot::try_merge(&parts, "head", "layer.3.heads.joined")?;

// `bias` may contain only the feature axis. It is broadcast across missing token axes.
let biased = activations.try_elementwise(
    &bias,
    "layer.3.biased",
    TensorElementwiseOp::Add,
)?;
```

The left operand defines an elementwise result's shape and axes. Right-hand axes may be omitted or
singleton for broadcasting; non-singleton axes must match by axis ID and semantic element IDs.
Invalid shapes, duplicate identities, unknown axes, and non-finite results return
`ValidationError` rather than silently changing the computation.

LayerNorm and RMSNorm operate along an explicitly named axis. The operation is non-affine: learned
scale and bias tensors can be applied afterward through the existing semantic elementwise API.
`NormalizationView` binds the input, computed output, and the per-group statistics used by that
same operation:

```rust
use murali::prelude::{NormalizationView, TensorNormalization};

let normalized = residual.try_normalized(
    "layer.7.attention.normalized",
    "feature",
    TensorNormalization::LayerNorm,
    1e-5,
)?;

let view = NormalizationView::try_new(
    residual,
    "layer.7.attention.normalized",
    "feature",
    TensorNormalization::LayerNorm,
    1e-5,
)?;
assert_eq!(view.normalized, normalized);
scene.add_tattva(view, Vec3::ZERO);
```

LayerNorm subtracts each slice mean and divides by its standard deviation. RMSNorm preserves the
mean and divides by root mean square instead. Both preserve shape, axis order, and semantic element
IDs. The focused rank-2 view supports up to 12 groups by 12 normalized features and rejects output
values or displayed statistics that no longer match the declared computation.

Categorical sampling accepts explicit unit-interval variates, so generated-token choices are
deterministic under preview, export, and timeline seeking:

```rust
let probabilities = logits.try_softmax("vocabulary")?;
let choices = probabilities.try_sample_categorical(
    "vocabulary",
    &[0.15, 0.55, 0.78],
)?;
let generated = choices.last().unwrap();
```

`NextTokenDistribution` turns a focused rank-1 vocabulary logit tensor into one complete sampling
decision. It computes temperature-scaled softmax, applies top-k and then top-p filtering,
renormalizes the retained probabilities, and uses the supplied unit-interval sample to select a
token. The view keeps the original model probability beside the filtered sampling probability, so
the effect of filtering remains visible:

```rust
use murali::prelude::{
    NextTokenDistribution, NextTokenSampling, TensorAxis, TensorSnapshot,
};

let logits = TensorSnapshot::try_new(
    "decoder.step.12.logits",
    vec![5],
    vec![2.8, 2.25, 1.7, 1.05, 0.4],
    vec![TensorAxis::with_elements(
        "vocabulary",
        "Candidate tokens",
        [
            ("token.scattered", "scattered"),
            ("token.blue", "blue"),
            ("token.across", "across"),
            ("token.through", "through"),
            ("token.softly", "softly"),
        ],
    )],
)?;

let distribution = NextTokenDistribution::try_from_logits(
    &logits,
    "vocabulary",
    NextTokenSampling::new(0.61)
        .with_temperature(0.85)
        .with_top_k(4)
        .with_top_p(0.90),
)?;
let selected_token = &distribution.selected().token;
scene.add_tattva(distribution, Vec3::ZERO);
```

The input must be rank 1 and retain at most 32 explicitly authored candidates. Full production
vocabularies should first be semantically sliced to the candidates being taught; the component
does not silently aggregate or hide entries. Invalid temperatures, filter values, sample values,
axes, and direct state mutations produce `ValidationError` diagnostics.

The runnable `self_attention_lesson` example combines these APIs into one computed path from input
tokens through Q/K/V, causal attention, value aggregation, a residual addition, logits, softmax,
and a sampled next token. Its tokens, embeddings, weights, model metadata, and source events come
from a checked-in version 1 JSON trace; all subsequent values are computed from those inputs.

## Importing AI traces

`AiTrace` is a framework-neutral, versioned boundary for recorded model state. It includes stable
tokens, validated tensor snapshots, model/layer/head metadata, and timestamped token, tensor,
operation, generation, metric, or authored-cue events. Import does not execute a model.

```rust
use murali::prelude::AiTrace;

let trace = AiTrace::from_json_path("recordings/attention.json")?;
let embeddings = trace.require_tensor("embedding.query")?.clone();
let token_axis = trace.token_axis("query", "Tokens");
```

For a self-contained binary, use `AiTrace::from_json_str(include_str!(...))`. Both entry points
reject unsupported schema versions, malformed tensors, duplicate IDs, non-finite or out-of-order
timestamps, and event references to unknown tokens or tensors. See
`examples/data/self_attention_trace.json` for the complete version 1 shape.

Transformer diagrams are semantic compositions rather than fixed stacks of labels. Built-in
pre-norm encoder and decoder compositions expose stable stage IDs, typed stage roles, tensor
bindings, and explicit residual sources. Custom compositions use the same model:

```rust
let block = TransformerBlockDiagram::try_from_stages(vec![
    TransformerStage::new(
        "qkv",
        "QKV Projection",
        TransformerStageKind::Projection,
    )
    .with_tensors(["residual.input"], ["q", "k", "v"]),
    TransformerStage::new(
        "attention",
        "Self-Attention",
        TransformerStageKind::SelfAttention,
    )
    .with_tensors(["q", "k", "v"], ["context"]),
    TransformerStage::new(
        "add",
        "Residual Add",
        TransformerStageKind::ResidualAdd,
    )
    .with_residual_from("input"),
])?;
```

Stage focus is a typed, seekable timeline operation:

```rust
timeline
    .animate(block_id)
    .at(2.0)
    .for_duration(0.5)
    .transformer_focus("self_attention")
    .spawn();
```

`KvCacheView` binds separate rank-2 key and value snapshots through shared token and feature axes.
Occupied rows show actual tensor values, future rows remain visibly empty, and the newest row is
highlighted as generation appends to the cache:

```rust
use murali::prelude::{KvCacheView, TensorSnapshot};

let cache = KvCacheView::try_new(
    key_snapshot,
    value_snapshot,
    "token",
    "feature",
    2,
)?;
let capacity = cache.capacity();
let cache_id = scene.add_tattva(cache, Vec3::ZERO);

timeline
    .animate(cache_id)
    .at(1.0)
    .for_duration(1.2)
    .kv_cache_fill_to(capacity)
    .spawn();
```

The key and value tensors may store their two axes in either order, but must share token IDs,
token labels, feature IDs, and dimensions. The focused teaching view supports up to 16 token slots
by 16 features; larger model tensors should be sliced explicitly. `kv_cache_fill_to` continuously
reveals new rows and reconstructs the same occupancy under repeated timeline seeking.

`SignalFlow` is useful when motion along a route is the story:

```rust
use murali::frontend::collection::ai::signal_flow::SignalFlow;

scene.add_tattva(
    SignalFlow::new(vec![
        Vec3::new(-2.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(2.0, 0.0, 0.0),
    ]),
    Vec3::ZERO,
);
```

`NeuralNetworkDiagram` is useful when layer structure matters:

```rust
use murali::frontend::collection::ai::neural_network_diagram::NeuralNetworkDiagram;

scene.add_tattva(
    NeuralNetworkDiagram::new(vec![3, 5, 2])
        .with_labels(vec!["Input", "Hidden", "Output"]),
    Vec3::ZERO,
);
```

## Runnable References

- `tensor_semantics` introduces named axes, stable identity, semantic selection, and computed attention.
- `tensor_operations` demonstrates reshape, split/merge, and named-axis broadcasting.
- `tensor_slicing` projects rank-4 activations into an explicit token-by-feature view.
- `self_attention_lesson` loads a versioned JSON trace and computes the full teaching path from Q/K/V through next-token sampling.
- `context_window` visualizes role-tagged prompt assembly, retained tokens, available capacity, and explicit truncation.
- `next_token_distribution` computes and visualizes temperature, top-k, top-p, and deterministic token selection.
- `kv_cache` grows a real key/value cache one token position at a time with seekable playback.
- `normalization` compares residual values before and after computed LayerNorm with per-token statistics.
- `transformer_attention` focuses on visual composition with tokens, an attention matrix, and a semantic transformer block.
- `neural_networks` focuses on network structure and signal playback.

Run any reference from the repository with `cargo run --example <name>`.
