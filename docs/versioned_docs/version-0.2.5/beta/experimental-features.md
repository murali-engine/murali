---
sidebar_position: 2
---

# Experimental Features

Some Murali APIs are available only behind Cargo feature flags. These surfaces are useful for
examples and early production work, but their names, module paths, builder methods, defaults, and
rendering details may change before they are promoted.

## Enable An Experimental API

Enable the feature in your `Cargo.toml`:

```toml
[dependencies]
murali = { version = "0.2.5", features = ["experimental"] }
```

Or enable it from the command line when running a repository example:

```bash
cargo run --features experimental --example linear_algebra_vectors
```

The repository helper scripts automatically enable `experimental` for examples tagged
`linear-algebra`:

```bash
./preview_all.sh --tag linear-algebra --auto
./export_all.sh --tag linear-algebra --release
```

## Current Experimental Surfaces

### Linear Algebra

The linear-algebra visual toolkit is available under:

```rust
use murali::frontend::collection::maths::linear_algebra;
```

This module is gated behind:

```rust
#[cfg(feature = "experimental")]
```

It currently includes early primitives and prototype views for vectors, coordinate readouts, basis
grids, projection shadows, dot-product meters, matrix transforms, matrix-vector flows, determinant
area views, and column-combination scenes.

Use it when:

- you are authoring Murali scenes directly
- you are comfortable updating code as the API evolves
- you want to test or improve the linear-algebra visual language

Avoid it for long-lived downstream libraries until the surface is promoted out of `experimental`.

## Stability Expectations

Experimental APIs are not part of Murali's stable compatibility promise yet. In particular, these
may change without a major-version bump while the crate is still early:

- module paths
- type names
- builder method names
- layout defaults
- styling defaults
- computed helper names
- returned IDs and composition structure

When an experimental feature becomes broadly useful and survives real examples or produced videos,
it can be moved into the default public API.
