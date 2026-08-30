---
sidebar_position: 7
---

# Core And Add-ons

Murali separates the engine from optional teaching domains as the Python bindings and companion
packages grow. That keeps the base install small, makes the public API easier to understand, and
leaves room for free and paid add-ons later.

## Product Boundary

**Murali Engine** is the core runtime. It should stay useful without AI, advanced math collections,
or domain-specific teaching packages.

The core should own:

- scene graph and tattva identity
- timelines, clips, animations, easing, and seeking
- camera, frames, SceneView, preview, export, and capture
- renderer/backend integration
- primitive geometry needed by most scenes
- text basics needed by examples and labels
- style, colors, layout primitives, and resource loading
- stable extension points for add-ons

Add-ons should own specialized teaching surfaces. They can depend on Murali Engine, but the engine
should not need to depend on them.

## Package Shape

The current Rust crate remains `murali` while the product name is **Murali Engine**. Over time, the
repository can grow into a Rust workspace if that lowers maintenance cost:

```text
crates/
  murali-engine/        # core runtime and stable authoring API
  murali-collection/    # free reusable visual components
  murali-ai/            # free AI teaching components
  murali-python/        # PyO3 bindings for the core surface
  murali-plugin-sdk/    # extension traits, manifests, licensing hooks
```

Python packages should mirror the same idea:

```text
murali-engine           # core Python bindings
murali-kit              # free general, maths, and AI authoring add-ons
murali-premium          # possible future paid package, if that name is chosen
```

The `murali-engine` package is the released Python engine package. If the `murali` name is available
on PyPI, it can be a convenience metapackage later.

## Free Add-ons

The first free add-on repo is
[`murali-engine/murali-kit`](https://github.com/murali-engine/murali-kit). It should collect
components that help adoption, examples, education, and community growth:

- common primitives, layouts, tables, basic composites, and examples
- basic math, graphing, calculus, statistics, probability, optimization, information theory, and
  linear algebra components
- neural networks, transformers, attention, KV cache, token distributions, traces, and agentic-flow
  visuals
- optional asset packs, themes, fonts, demo models, and textures

The exact internal module split can evolve inside `murali-kit`. Separate free PyPI packages can wait
until a domain becomes heavy enough to justify its own install. The important rule is that these
domains should not become required dependencies of the engine.

## Premium Add-ons

Premium add-ons should be separate packages, not hidden branches inside the core engine. They should
depend on `murali-engine` directly unless they intentionally want `murali-kit` helpers.

Candidates:

- polished course-ready AI visualization packs
- high-level diagram generators for LLMs, agents, RAG, evaluation, and fine-tuning
- template packs for commercial explainers and branded technical videos
- advanced export workflows or batch rendering integrations
- hosted asset/theme packs
- enterprise-specific integrations

The open-source engine should remain credible and useful. Premium packages should add leverage,
finish, and specialized workflows rather than holding back basic engine capability.

## Extension Contract

Before paid packages exist, the engine needs a clean add-on contract:

- public traits or registration functions for add-on tattvas
- stable scene/timeline/animation handles
- stable resource registration for fonts, textures, models, and generated assets
- explicit feature flags and package boundaries
- version compatibility rules
- Python import boundaries that match Rust crate boundaries
- optional licensing hooks that are not required for free add-ons

The core should avoid reaching into add-on internals. Add-ons should register capabilities with the
engine through stable interfaces.

## Python Binding Strategy

Start with the released `murali-engine` package for Python:

```python
from murali_engine import Scene, Timeline, Circle, Label
```

Then add optional packages:

```python
from murali_kit.ai import NeuralNetworkDiagram, AttentionMatrix
from murali_kit.maths.linear_algebra import VectorArrow2D, TransformableGrid2D
```

The Python engine package should expose the core engine surface: create a scene, add common visual
objects, animate them, preview, and export PNG frames. Add-ons can build on that boundary without
becoming required dependencies of the engine package.

## Decision Rules

Keep something in core when:

- most Murali projects need it
- add-ons need it as infrastructure
- it is required for preview, export, rendering, scene composition, or timelines
- it is small, stable, and domain-neutral

Move something to an add-on when:

- it is domain-specific
- it brings heavy dependencies
- it is experimental or course-specific
- it is useful but not necessary for the engine to run
- it may later become a commercial workflow or asset pack

## Near-Term Plan

1. Keep the current Rust crate working while the Python package matures.
2. Treat `murali-engine` as the shared engine boundary for Python packages.
3. Use `murali-kit` as the first consumer of the engine package and as the home for broader Python
   examples.
4. Add extension-facing traits and wrappers only where real add-on packages need them.
5. Move optional domains toward free add-ons only when the split lowers complexity.
6. Keep premium add-ons as a future packaging and licensing concern, not as a blocker for the
   open-source core.

The immediate priority is not monetization plumbing. It is a clean engine boundary that lets free
and premium packages exist later without redesigning the API.
