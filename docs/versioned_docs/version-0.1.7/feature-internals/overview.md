---
sidebar_position: 0
---

# Feature Internals

This section is for contributors who want to understand how complex built-in Murali features are structured internally.

These pages are intentionally different from the user-facing tattva docs.

User-facing docs focus on:

- what the feature does
- how to use the public API
- examples and authoring patterns

Feature internals focus on:

- internal data model
- render strategy
- animation and state flow
- extension points
- limitations and tradeoffs

These pages do **not** belong in the core [Architecture](../architecture/overview) section because they are not engine-wide internals. They are higher-level systems built on top of the core architecture.

## What Lives Here

This section is a good fit for features that:

- have a meaningful internal model of their own
- are more complex than a simple primitive tattva
- benefit from contributor-oriented explanation

Current pages:

- [Stepwise](./stepwise)
- [Neural Network Diagram](./neural-network)

As Murali grows, this section is also a good home for contributor deep dives on features like:

- agentic flow chart, if it is revisited in a future iteration
- transformer block diagrams
- advanced storytelling tattvas
- AI teaching templates
