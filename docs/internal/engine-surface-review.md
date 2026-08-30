# Murali Engine Surface Review

This document records the repo-boundary review for `murali` after the Python
package split.

## Intended Role

The `murali` repo should provide the engine implementation and the Python
binding surface exposed as `murali_engine`.

It should not try to be the main high-level authoring library for animation
creation. Authored collections, teaching components, themes, and example-driven
helpers should move to `murali-kit` or future add-on packages.

## Engine Responsibilities

The engine should keep:

- scene graph, tattva lifecycle, transforms, layers, and bounds
- timeline, animation, easing, callbacks, seeking, playback, and scene views
- renderer, preview, export, screenshots, resources, and asset loading
- camera, frame, projection, depth, and render synchronization logic
- primitive geometry and generic renderables needed by many scenes
- PyO3 bindings for the stable low-level Python authoring surface
- tests for runtime behavior and contracts required by add-ons

## Reference-Only Areas

The current Rust examples and many frontend collection modules are useful
reference material, but they should not define the long-term public authoring
surface.

Reference material can stay while it helps with:

- behavior comparison against `murali-kit`
- visual parity checks
- recovering layout or animation algorithms
- deciding which lower-level engine hooks are missing

When the Python kit has equivalent components and examples, the Rust-side
collection code can be reviewed for removal, internalization, or conversion into
engine tests.

## Removal Candidates

These trees should be treated as migration/removal candidates, not deleted
blindly:

- `examples/`
- `src/frontend/collection/ai/`
- `src/frontend/collection/composite/`
- `src/frontend/collection/layout/`
- `src/frontend/collection/maths/`
- `src/frontend/collection/storytelling/`
- `src/frontend/collection/table/`
- domain-specific pieces under `src/frontend/collection/common/`

Some modules inside `src/frontend/collection/text/`, `src/frontend/collection/primitives/`,
and `src/frontend/collection/utility/` may remain engine-owned because they
represent generic primitives or generic renderables. They should be reviewed
file by file rather than moved as a whole.

## Current Python Binding Notes

The current `murali_engine` binding intentionally exposes enough surface for
example parity. A few exposed classes probably belong in `murali-kit` over time:

- `ContextBlock`
- `ContextWindow`
- `SignalFlow`
- `OptimizationPath2D`

They should remain available until kit-owned replacements exist. After examples
prefer the kit-owned imports, the engine exports can be deprecated or retained
only as compatibility shims.

## Public Rust Prelude

`src/frontend/collection/mod.rs` currently exposes a broad `prelude` that
re-exports collection modules. This keeps Rust examples convenient, but it also
encourages the engine repo to behave like a frontend authoring library.

Before removing or shrinking this prelude:

1. Find all in-repo usages.
2. Find Python binding dependencies on the same types.
3. Confirm `murali-kit` has equivalent Python APIs where needed.
4. Move reusable low-level pieces into engine-owned primitive modules if they are
   still required.
5. Remove or hide only the pieces that are no longer part of the engine contract.

## Safe Removal Process

For each candidate module:

1. Classify it as `engine primitive`, `generic renderable`, `kit collection`,
   `reference only`, or `test only`.
2. Search all Rust, Python binding, test, and docs usage.
3. If it is used by `murali_engine`, decide whether the binding should remain,
   move to kit, or become a compatibility shim.
4. If it belongs to kit, create the kit-side API first.
5. Update examples and docs to use the kit API.
6. Add or update contract tests.
7. Remove the Rust source only when no public engine behavior depends on it.

## Near-Term Recommendation

Do not delete collection code immediately. The safer next iteration is:

1. Make the Python engine surface coherent.
2. Move kit-shaped Python exports behind kit-owned wrappers.
3. Update examples to prefer kit imports.
4. Add engine/kit contract tests.
5. Then remove or internalize Rust collection modules in small batches.

