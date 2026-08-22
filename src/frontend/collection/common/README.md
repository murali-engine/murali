# Common

Internal shared collection infrastructure for domain-neutral building blocks.

Use this folder when multiple domains need the same low-level representation and neither domain
should own it. Prefer keeping subject-specific teaching components in their subject folders and
re-exporting them from facades when helpful.

This module is crate-private. Public APIs should expose these types through the owning teaching
domain, not through `collection::common`.

Current exports:

- `TensorAxis`
- `TensorSnapshot`
- `TensorView`
- `TensorSelector`
- `TensorNormalization`
- `TensorSlice`
- `TensorSample`
