# Frontend Sangrah Architecture

`src/frontend/sangrah` has one public categorization level.

Sangrah means collection. In Murali, it is the author-facing store of reusable visual tattvas and
teaching components.

There are three kinds of folders here:

- implementation families such as `ai`, `ganit`, `text`, `primitives`, `layout`,
  `storytelling`, `table`, `composite`, and `utility`
- teaching-domain categories such as `ai::deep_learning`, `ai::transformers_llms`,
  `ai::ml_components`, and `ai::systems_agentic_ai`
- crate-private shared infrastructure under `common`

Public category folders should own at least one real component or concrete implementation subtree.
They may also re-export related components from nearby categories when lesson authors would
naturally search by topic. The crate root also exposes this module as `murali::sangrah`.

## Ownership Rule

A functional component should have one implementation owner. For example, `AttentionMatrix` is
implemented in `ai::transformers_llms`, even if it is useful to `ai::deep_learning` and
`ganit::information_theory`.

Domain categories are discovery surfaces, not second owners. They may re-export the same component
when a lesson author would naturally look for it from more than one teaching domain, but each public
domain should still contain at least one owned component once it becomes part of the baseline API.

## Add New Code

- Put new component implementations in the closest owning category.
- Re-export a component from another category only when it helps authors find it by subject.
- Add a new top-level category only when an existing category would make ownership ambiguous.
- Keep scene-specific code in examples or production scenes until it proves reusable.

## Implementation Families

- `primitives`: atomic marks such as arrows, circles, lines, paths, and shapes.
- `text`: labels, LaTeX, Typst, code blocks, and 3D letters.
- `ganit`: math-facing notation, graphing, and quantitative teaching domains.
- `composite`: reusable composed visual objects such as axes, cards, logos, and number planes.
- `layout`: grouping and stack layout helpers.
- `storytelling`: reusable explanation-flow components such as `Stepwise`.
- `table`: table surfaces.
- `utility`: cross-cutting utility tattvas.
- `ai`: AI-specific semantic views, model-state components, and AI teaching domains.

## Existing Domain Categories

- `ai::deep_learning`
- `ai::transformers_llms`
- `ai::ml_components`
- `ai::systems_agentic_ai`

Math-facing domains such as `basic_math`, `graph`, `linear_algebra`, `probability`, `statistics`,
`calculus`, `optimization`, `information_theory`, and `data_geometry` live under `ganit`.
