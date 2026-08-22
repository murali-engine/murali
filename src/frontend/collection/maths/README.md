# Maths

`maths` is the math umbrella inside Murali's `collection`.

It owns math-facing notation, continuous geometry, quantitative views, and teaching-domain components. Keep math concepts here
instead of spreading them across top-level `collection` folders.

## Implementation Families

- `notation`: equations, matrices, and notation-level math surfaces.
- `graph`: graph-specific presentation helpers such as legends.
- `calculus`: function graphs, curves, surfaces, vector fields, and streamlines.
- `data_geometry`: scatter plots, embeddings, and sampled geometric data.
- `linear_algebra`: experimental vector, basis, projection, span, matrix-transform, and
  composition components.

## Teaching Domains

- `basic_math`
- `calculus`
- `probability`
- `statistics`
- `optimization`
- `information_theory`
- `data_geometry`

## Ownership Rule

Each component should still have one implementation owner. Domain modules may re-export a component
when lesson authors would naturally search by topic, but they should not duplicate the underlying
implementation.
