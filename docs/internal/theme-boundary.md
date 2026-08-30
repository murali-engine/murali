# Theme Boundary

This note defines the theme split for the Python-first Murali direction.

## Decision

`murali-engine` should not expose user-facing theme selection as a primary Python feature.

The engine owns:

- RGBA color tuples and internal default fills
- color parsing and conversion
- explicit per-object colors
- explicit scene background color
- font and asset-loading primitives
- renderer/runtime defaults required when no theme is applied

`murali-kit` owns:

- light and dark themes
- named theme packs
- theme objects that style an existing engine scene
- object presets derived from engine primitives
- authored combinations of colors, typography, spacing, surfaces, textures, and animation defaults

Future packages can own additional themes. For example, a later premium package can ship
professionally designed themes without depending on `murali-kit` if it only needs
`murali-engine`.

## API Direction

Engine-style usage should remain explicit:

```python
from murali_engine import Circle, Scene

scene = Scene(background=(1.0, 1.0, 1.0, 1.0))
scene.add(Circle(radius=0.5, color=(0.35, 0.77, 0.87, 1.0)))
```

Kit-style usage can be opinionated, but `Scene` still comes from the engine:

```python
from murali_engine import Scene
from murali_kit.themes import DarkTheme, apply_theme
from murali_kit.composite import TitleCard

scene = apply_theme(Scene(), DarkTheme())
TitleCard("Attention", "Query, key, value").add_to_scene(scene)
```

Premium-style usage can be independent:

```python
from murali_engine import Scene
from murali_premium.themes import EditorialDark, apply_theme

scene = apply_theme(Scene(), EditorialDark())
```

## Engine Responsibilities

The engine should make themes possible without owning the authored theme catalog.

For `0.3.0`, prioritize:

- `Scene(background=...)` or an equivalent explicit background API
- stable RGBA color tuples
- object-level colors that do not require global theme state
- a renderer default background when no background is set
- no required `murali-kit` dependency

Engine defaults should be plain and conservative. They should make scenes render correctly, not
express a branded visual identity.

## Kit Responsibilities

`murali-kit` should become the home for theme selection and designed looks.

For the first coherent kit theme layer, prioritize:

- `DarkTheme`
- `LightTheme`
- `apply_theme(scene, theme)` and `theme.apply(scene)`
- helpers that style common engine objects
- examples that show the same scene in light and dark modes

The kit can add more named themes later without changing the engine package.

## Migration From Current Rust Theme System

The Rust engine currently has `frontend::theme::Theme`, semantic color accessors, built-in dark and
light TOML themes, and `murali.toml` theme selection. That system should not be deleted blindly.

Migration policy:

- keep Rust theme support temporarily for backwards compatibility
- avoid exposing it as the primary Python theme API
- prefer explicit Python style arguments in `murali-engine`
- move Python theme ergonomics to `murali-kit`
- eventually reduce engine global theme state or make it an internal/default-render concern

This lets `0.3.0` become Python-coherent without breaking existing Rust users unnecessarily.

## Open Questions

- Should engine `Scene` accept `background=` directly in `0.3.0`?
- Should kit themes apply styles by constructing a scene, wrapping a scene, or both?
- Should kit themes style existing objects, or only provide constructors/presets for new objects?
- Should code-block syntax themes stay as engine internals or become kit-level style choices?
- How should premium themes share a common theme protocol without depending on `murali-kit`?
