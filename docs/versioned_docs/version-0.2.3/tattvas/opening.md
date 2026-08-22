---
sidebar_position: 6
---

# Opening (Beta)

:::caution Beta API

`Opening` is intentionally published under `composite::beta`. Its names, defaults, and
choreography may change while the component is refined across real productions. Import it
explicitly rather than through the main prelude.

:::

`Opening` is an opinionated temporal composite for 3D capital-letter idents. It creates one
extruded letter and one matching particle cloud per title character, drops and settles the solid
letters, builds into a shake, bursts into particles, and reveals a tagline.

`Opening` is not one opaque tattva and it is not inherently a `SceneView`. It follows Murali's
normal composite idiom:

1. `Opening` stores authored configuration.
2. `add_to_scene` creates normal tattvas and returns `OpeningIds`.
3. `OpeningIds::animate` authors reusable local-time choreography into a `Timeline` or `Clip`.
4. The host scene chooses the camera, projection, background, and export settings.

## Import

```rust
use murali::frontend::sangrah::composite::beta::opening::{
    Opening, OpeningError, OpeningIds, OpeningStyle, OpeningTiming,
};
```

The beta types are not directly re-exported by `murali::prelude`.

## Basic Construction

```rust
use glam::Vec3;
use murali::{Clip, Scene, Timeline};

let mut scene = Scene::new();
let opening = Opening::new("KAVRIQ", "The Science Behind AI");
let ids = opening.add_to_scene(&mut scene, Vec3::ZERO)?;

let mut opening_clip = Clip::new();
ids.animate(&mut opening_clip);
let duration = opening_clip.duration();

let mut timeline = Timeline::new();
timeline.append(opening_clip);
scene.play(timeline)?;
```

Titles currently accept ASCII capitals `A` through `Z` and spaces. Spaces participate in layout
but do not create letter or particle tattvas. The title must contain at least one capital letter.

## Configure In Rust

Opening configuration belongs in the animation script. No TOML file or environment-variable
layer is required.

```rust
use glam::Vec4;
use murali::{BuiltinTexture, TextureImage};

const TITLE: &str = "KAVRIQ";
const TAGLINE: &str = "The Science Behind AI";
const FONT: &str = "assets/fonts/brand-bold.ttf";

const PARTICLE_PALETTE: [Vec4; 4] = [
    Vec4::new(0.18, 0.82, 0.78, 1.0),
    Vec4::new(0.32, 0.58, 1.0, 1.0),
    Vec4::new(0.92, 0.38, 0.68, 1.0),
    Vec4::new(1.0, 0.72, 0.22, 1.0),
];

let opening = Opening::new(TITLE, TAGLINE)
    .with_font_path(FONT)
    .with_texture(TextureImage::builtin(BuiltinTexture::WhiteMarble))
    .with_style(OpeningStyle {
        letter_height: 2.4,
        letter_depth: 0.95,
        letter_gap: 0.34,
        particle_count: 700,
        particle_size: 0.028,
        particle_palette: PARTICLE_PALETTE.to_vec(),
        tagline_height: 0.48,
        ..OpeningStyle::default()
    })
    .with_timing(OpeningTiming {
        dissolve_duration: 0.82,
        end_hold: 0.59,
        ..OpeningTiming::default()
    });
```

`with_font_path` controls the extruded title and its matching particle silhouettes. `with_texture`
accepts an in-memory texture; `BuiltinTexture::BlackMarble` and `BuiltinTexture::WhiteMarble` are
embedded in Murali and need no asset path. Use `with_texture_path` for your own image files. Without
a font, the bundled Inter font is used, and without a texture, face colors are rendered directly.

## OpeningStyle

`OpeningStyle` contains visual, layout, and particle-motion controls. All fields are public and
can be set with struct update syntax.

| Field | Default | Meaning |
| --- | ---: | --- |
| `letter_height` | `2.4` | Capital height in world units |
| `letter_depth` | `0.95` | Extrusion depth in world units |
| `letter_gap` | `0.34` | Gap between adjacent title slots |
| `space_width` | `0.85` | Width reserved by a title space |
| `final_y` | `-0.42` | Settled title baseline offset from the supplied origin |
| `front_color` | warm bone white | Front-face color and texture tint |
| `back_color` | muted bone gray | Back-face color and texture tint |
| `side_color` | dark bone gray | Extruded side color and texture tint |
| `particle_count` | `700` | Particles sampled per capital letter |
| `particle_size` | `0.028` | Base particle radius in world units |
| `particle_color` | bone gray | Initial particle color before palette transition |
| `particle_palette` | six colors | Destination colors used during scatter |
| `particle_distance` | `4.8` | Base radial scatter distance |
| `particle_rise` | `2.35` | Vertical particle displacement |
| `particle_curl` | `1.0` | Curl applied during scatter |
| `tagline_height` | `0.48` | Tagline text height in world units |
| `tagline_color` | near white | Tagline text color |
| `tagline_font_name` | `None` | Optional registered `Label` font name |

Higher particle counts improve silhouette density but increase tessellation, synchronization, and
render cost. Count is per letter, so a six-letter title with `700` creates `4,200` particles.

### Tagline Font

The 3D title font path and 2D tagline font intentionally use Murali's existing separate font
systems. Register a label font once, then select it by name:

```rust
use murali::register_font_path;

register_font_path("Brand Sans", "assets/fonts/brand-bold.ttf")?;

let style = OpeningStyle {
    tagline_font_name: Some("Brand Sans".to_owned()),
    ..OpeningStyle::default()
};
```

If `tagline_font_name` is `None`, the tagline uses Murali's bundled default label font.

## OpeningTiming

`OpeningTiming` describes phase durations in local seconds.

| Field | Default | Meaning |
| --- | ---: | --- |
| `intro_delay` | `0.4` | Delay before the first letter starts falling |
| `landing_stagger` | `0.3` | Delay between adjacent letter landings |
| `landing_duration` | `1.38` | Fall and rotation-settle duration |
| `bounce_up_duration` | `0.18` | First impact rebound duration |
| `bounce_down_duration` | `0.24` | Return from rebound to rest |
| `settled_hold` | `0.55` | Pause after the final letter settles |
| `shake_duration` | `0.97` | Total accelerating shake phase |
| `shake_beats` | `11` | Number of alternating shake beats |
| `particle_scatter_duration` | `1.5` | Duration of particle displacement and color transition |
| `tagline_reveal_delay` | `0.72` | Delay from burst to tagline reveal and particle fade |
| `dissolve_duration` | `0.82` | Particle fade and tagline reveal duration |
| `end_hold` | `0.59` | Stable tagline hold after all active effects finish |

The schedule adapts to title length. For `n` capital letters:

```text
shake start = intro delay
            + (n - 1) * landing stagger
            + landing duration
            + bounce up duration
            + bounce down duration
            + settled hold

burst time = shake start + shake duration

total duration = burst time
               + max(particle scatter duration,
                     tagline reveal delay + dissolve duration)
               + end hold
```

With the defaults, a six-letter title lasts `7.35` seconds.

## OpeningIds

`add_to_scene` returns `OpeningIds`:

```rust
ids.letters;   // Vec<OpeningLetterIds>
ids.tagline;   // TattvaId
ids.all();     // solids, particle clouds, and tagline
ids.duration();
ids.animate(&mut timeline);
```

Each `OpeningLetterIds` exposes:

```rust
letter_ids.solid;
letter_ids.particles;
```

Use these IDs when a production needs additional per-letter animation after the built-in
choreography. `animate` always authors from local time `0.0`; write into a `Clip` when the opening
must be appended, overlaid, or placed later on another timeline.

## Projection

`Opening` does not change the host camera. Its depth, behind-camera entry, and particle motion are
designed for perspective projection:

```rust
use murali::engine::camera::Projection;

scene.camera_mut().projection = Projection::Perspective {
    fov_y_rad: 43.0_f32.to_radians(),
    aspect: scene.frame().aspect_ratio(),
    near: 0.1,
    far: 80.0,
};
scene.camera_mut().position = vec3(0.0, 2.15, 10.8);
scene.camera_mut().target = vec3(0.0, -0.35, 0.0);
```

An orthographic host can render the component, but it will not produce the intended perspective
arrival. When the continuation should remain orthographic, put the perspective opening scene in a
[`SceneView`](../scene-views) instead of changing the parent projection.

## Background

`Scene` does not currently own an animatable clear color. Background ownership depends on how the
opening is presented:

| Presentation | Background source |
| --- | --- |
| Standalone direct export | `ExportSettings::clear_color` |
| Opening inside a `SceneView` | `SceneView::background(...)` |
| Transparent child composition | `SceneView::transparent_background()` |

For a standalone export:

```rust
let settings = ExportSettings {
    clear_color: BACKGROUND,
    duration_seconds: ids.duration(),
    ..ExportSettings::from_scene(&scene)
};
```

A SceneView background belongs to the view. Moving, scaling, rotating, or fading the parent-facing
view ID affects the child render and its background together.

## Continue Into Another Scene

Use a full-frame SceneView when the opening needs its own perspective camera while the continuing
parent remains orthographic:

```rust
let (opening_scene, opening_duration) = build_opening_scene()?;
let (frame_width, frame_height) = parent.frame().logical_size();

let opening_view = parent.add_scene_view(
    SceneView::new(opening_scene)
        .size(vec2(frame_width, frame_height))
        .background(BACKGROUND)
        .playback(SceneViewPlayback::Once)
        .resolution(1280, 720),
    Vec3::ZERO,
);

parent_timeline
    .animate(opening_view)
    .at(opening_duration - 0.3)
    .for_duration(0.72)
    .ease(Ease::InOutCubic)
    .fade_to(0.0)
    .spawn();
```

`SceneViewPlayback::Once` stops the child clock at the end and keeps the final frame visible. It
does not automatically remove the opening. Fade, hide, move, or remove the parent-facing view ID
as part of the parent timeline.

See [SceneView: Full-Screen Opening Handoff](../scene-views#full-screen-opening-handoff) for the
complete composition model.

## Errors And Validation

`add_to_scene` returns `Result<OpeningIds, OpeningError>`. Construction fails for:

- an empty title or a title containing no capital letters
- unsupported title characters
- zero, negative, non-finite, or empty settings where the field requires a valid value
- unreadable or invalid font files
- unreadable or invalid texture files supplied through `with_texture_path`
- missing glyphs or glyph tessellation failures

Validation happens before the opening mutates the scene whenever the error can be determined from
configuration alone. Asset and glyph failures are reported with their source error.

## Choosing Standalone Or SceneView

| Need | Recommended form |
| --- | --- |
| Export the opening as a separate video for an editor | Direct `Scene` export |
| Continue in the same projection and camera | Add the opening and later content to one `Scene` |
| Continue with a different projection or camera | Full-frame `SceneView` |
| Fade the opening background and content atomically | Full-frame `SceneView` |
| Dock, tilt, scale, or replay the complete opening | `SceneView` parent handle |

## Reference Examples

Standalone branded export with Rust constants:

```bash
cargo run --release --example kavriq_opening
```

Perspective SceneView opening followed by ongoing orthographic content:

```bash
cargo run --release --example opening_scene_view
```

The Satoshi font used by the KAVRIQ repository example is a private local asset and is not bundled
with Murali. The example falls back to the bundled Inter font when that file is unavailable. The
`opening_scene_view` example uses only portable checked-in or bundled assets.
