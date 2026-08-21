---
sidebar_position: 1
---

# Beta And Experimental

Murali's beta and experimental APIs are a place for useful video-making components that have not
settled into a stable long-term shape yet.

These APIs are intentionally unstable. Names, defaults, module paths, visual styling, returned IDs,
timing helpers, and construction patterns may change rapidly in future releases to improve
usability, ergonomics, appearance, and production fit.

Use this section when you want components that are helpful for real videos today, but are still
being tested across actual scenes.

## Building Blocks And Comfort Tattvas

Murali's stable core should stay focused on composable building blocks. In an ideal workflow, users
can create most composite visual elements from primitives, text, layout, animation, and camera tools
as each video needs them.

At the same time, Murali is an opinionated library for making explanatory videos. It will sometimes
include higher-level comfort tattvas, such as AI attention visuals, chat bubbles, prompt-entry
boxes, or other reusable video elements. Their purpose is practical: they make common scenes easier
and more comfortable to build.

This can create bloat if it goes unchecked. Beta is the proving ground for these opinionated
components. They may stay here for a long time while their API, appearance, and ergonomics are
tested. Some may eventually become stable; some may be trimmed, renamed, or redesigned.

## What Belongs Here

Beta components are often storytelling or video-production primitives. They may not belong cleanly
under math, AI, graphs, layout, or basic shapes.

Examples include:

- chat input boxes
- message bubbles
- opening-title treatments
- prompt-entry scenes
- reusable explainer UI pieces
- opinionated visual compositions

The goal is to make these components available early without pretending their APIs are finished.

## Stability Expectations

Code in `murali::frontend::collection::composite::beta` should be imported explicitly and treated
as provisional:

```rust
use murali::frontend::collection::composite::beta::ChatInputBox;
```

Avoid depending on beta APIs for long-lived public libraries unless you are comfortable updating
your code as Murali evolves.

## Current Beta Components

- [Chat Input Box](./chat-input-box.md) - a prompt/chat bubble composite with configurable bottom tip.
- [Opening](../tattvas/opening.md) - an opinionated 3D title-opening composite.
