---
sidebar_position: 7
---

# Roadmap

This page tracks what Murali already ships in `v0.1.6`, what still feels rough, and what we want to improve next.

The goal is not to promise dates. The goal is to preserve product judgment: when we revisit Murali later, we should be able to see not just a feature list, but the usability priorities behind the next round of work.

## v0.1.6 Snapshot

Murali now has a broad enough surface that the main challenge is no longer raw capability. The challenge is making the platform easier to learn, easier to predict, and easier to author confidently.

The current release already includes:

- a deterministic scene and timeline model
- world-space authoring with camera controls
- a broad tattva surface across primitives, text, math, graphs, 3D, AI diagrams, storytelling, tables, and utilities
- a large reference-example set inside this repository
- export support for image and video workflows
- richer text support across `Label`, `Latex`, `Typst`, and `CodeBlock`

That means the next roadmap should focus less on “add more categories” and more on “make the existing categories feel coherent and ergonomic.”

## Product Direction

The next phase of Murali should optimize for four things:

1. explicit authored behavior over hidden smart behavior
2. preset-driven ergonomics for common patterns
3. stronger API discoverability and decision guidance
4. tighter alignment between shipped APIs, examples, and docs

In short: Murali should feel more trustworthy, more composable, and easier to reach for without reading the source.

## Near-Term Priorities

These are the areas we should treat as the first layer of post-`0.1.6` work.

### 1. CodeBlock Ergonomics

`CodeBlock` is now useful, but it is also the clearest example of where Murali can feel too clever in the wrong places.

What we want to preserve:

- explicit text sizing through `CodeBlock::new(..., world_height)`
- explicit block sizing through `.with_content_box_size(...)`
- explicit code placement through `.with_content_offset(...)`
- built-in theme and surface presets

What we want to improve:

- make authored values win consistently
- avoid hidden auto-fit behavior in explicit mode
- make panel size, code size, block position, and internal content offset easier to understand at a glance
- improve docs and examples so the authored model is obvious without reading implementation details

What we should intentionally defer until the API matures:

- custom themes
- aggressive auto-layout intelligence
- large-block renderer redesign as part of the public authoring contract

### 2. Presets For Common Authored Patterns

Murali already exposes strong low-level authored control. The next step is to add more ergonomic presets without taking that control away.

Good preset opportunities:

- text hierarchy helpers for title, subtitle, and body styles
- `CodeBlock` convenience presets that bundle theme + surface combinations
- camera and reveal presets for common 3D scene structures
- AI diagram presets for common transformer, token, and signal-flow compositions

The rule should be:

- presets should reduce repetition
- presets should not hide authored geometry
- presets should always be overridable

### 3. Stronger API Decision Guidance

Murali now has enough overlap that choice friction matters.

The most important docs to strengthen are:

- `which-api-should-i-use`
- tattva family pages
- example index pages

The most important decision boundaries to keep clarifying are:

- `Label` vs `Typst` vs `Latex` vs `CodeBlock`
- `play` vs `play_named` vs `set_timeline`
- scene-level helpers vs typed tattva mutation
- `streamlines` vs `force_fields`
- `surfaces_3d` vs `wireframe_surfaces`

The goal is not just documentation completeness. The goal is to help users choose the highest-level API that matches their intent.

### 4. Reference-Grade Documentation Discipline

Murali now has enough documentation and examples that drift becomes a real product risk.

Post-`0.1.6`, we should treat these as a synchronized surface:

- public docs
- reference examples
- public API behavior

Whenever an API changes meaningfully, the corresponding review should include:

- tattva-page update
- `which-api-should-i-use` review
- example-catalog review
- reference-example wording review

This is especially important for:

- text APIs
- 3D surfaces
- AI diagrams
- export behavior

## Medium-Term Priorities

These are important, but slightly less urgent than the ergonomics issues above.

### 5. Higher-Level AI Composition

The AI tattva family is already a differentiator for Murali, but it still feels closer to a capable toolkit than a fully ergonomic authored surface.

What we want next:

- better defaults for spacing and composition
- more reusable layout patterns for AI diagrams
- easier construction of transformer-style scenes
- higher-level builders where repetition is obvious and common

The guiding idea:

- preserve composability
- reduce boilerplate
- make the “good default diagram” easy to author

### 6. More Systematic Layout Ergonomics

Layout helpers are already useful, but the next phase should make authored arrangement feel more systematic across the crate.

Potential directions:

- more predictable group and stack ergonomics
- better documentation around bounds and anchors
- helper surfaces for common “panel + annotation + content” arrangements
- clearer guidance for when to use layout helpers vs manual positioning

This should build on Murali’s explicit world-space model rather than trying to become a constraint-layout system.

### 7. Example-Set Curation

The reference set is now large enough. Future work should favor sharper roles over more examples.

We should continue to:

- keep the reference set curated and non-redundant
- use this repository for official reference-quality examples
- publish selected creative and exploratory results through the video showcase

The reference examples should answer:

- what the platform can do
- what the recommended patterns look like
- where users should start learning

## Longer-Term Engine And Renderer Work

These are real needs, but they should follow the ergonomics work rather than replace it.

### 8. Better Support For Large Text And Code Surfaces

Current `CodeBlock` rendering is good enough for reference scenes, but the underlying limitations are known:

- Typst-backed code is rasterized before GPU rendering
- large snippets can hit texture-size limits
- oversized blocks are not yet a solved “just works” problem

Longer-term directions:

- tiled rendering for large code surfaces
- better renderer behavior for oversized Typst content
- a clearer distinction between authored block size and render resource strategy

This work matters, but it should be done without destabilizing the simpler explicit authoring model.

### 9. Additional Import / Material / Asset Pipelines

This is still interesting long term, especially for richer 3D or mixed-media scenes, but it is no longer the highest-leverage next step.

When revisited, the bar should be:

- it expands Murali’s teaching and storytelling range
- it fits the authored scene model cleanly
- it does not distract from the current ergonomics priorities

## What We Should Keep Protecting

As Murali grows, these qualities should remain non-negotiable:

- deterministic timelines
- explicit world-space authoring
- clear authored intent
- strong visual semantics
- examples that teach, not just impress

If a future improvement makes the engine more powerful but less predictable, it should be treated with caution.

## Summary

The post-`0.1.6` roadmap is not primarily about adding new categories of features.

It is about making Murali feel:

- more explicit
- more ergonomic
- more teachable
- more internally consistent

The highest-value next work is:

1. stabilize and simplify `CodeBlock` ergonomics
2. add more preset-driven authored helpers
3. strengthen API decision guidance
4. keep docs and examples tightly synchronized
5. make AI diagram composition easier at the authored level

That is the next layer of maturity for Murali.
