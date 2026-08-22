---
sidebar_position: 3
---

# Composite

Composite components are made of multiple primitives or tattvas. They live under `murali::frontend::sangrah::composite`.

## Beta Components

Beta composites live under `composite::beta` and are not re-exported by Murali's main prelude.
Their APIs and choreography may change as they mature.

### Opening

`Opening` is an opinionated temporal composite for 3D capital-letter openings. It drops a title
into a perspective scene, shakes the settled letters apart, scatters glyph-shaped particles, and
reveals a tagline. The component creates normal tattvas and authors its sequence into either a
`Timeline` or a local-time `Clip`. See the dedicated [Opening (Beta)](./opening) guide for the full
style and timing reference, projection and background ownership, validation, standalone export,
and SceneView handoff.

```rust
use glam::Vec3;
use murali::frontend::sangrah::composite::beta::opening::{
    Opening, OpeningStyle, OpeningTiming,
};
use murali::colors::{BLUE, GOLD, PINK, TEAL};
use murali::{BuiltinTexture, Clip, Scene, TextureImage, Timeline};

let opening = Opening::new("KAVRIQ", "The Science Behind AI")
    .with_font_path("assets/fonts/brand-bold.ttf")
    .with_texture(TextureImage::builtin(BuiltinTexture::WhiteMarble))
    .with_style(OpeningStyle {
        letter_gap: 0.34,
        particle_count: 700,
        particle_palette: vec![TEAL, BLUE, PINK, GOLD],
        tagline_font_name: Some("Brand Sans".to_owned()),
        ..OpeningStyle::default()
    })
    .with_timing(OpeningTiming {
        dissolve_duration: 0.82,
        ..OpeningTiming::default()
    });

let ids = opening.add_to_scene(&mut scene, Vec3::ZERO)?;
let mut opening_clip = Clip::new();
ids.animate(&mut opening_clip);

let mut timeline = Timeline::new();
timeline.append(opening_clip);
scene.play(timeline)?;
```

Titles currently accept ASCII capitals and spaces. A custom 3D title font and texture are optional;
without a font path the bundled Inter font is used. The marble texture shown above is embedded in
Murali and needs no asset path; use `with_texture_path` for a project-owned image. The tagline uses
Murali's normal `Label` font registry: call `register_font_path` first, then set
`tagline_font_name` to the registered name.
The scene camera and export background remain host concerns, just as they are for other 3D
composites. For a standalone opening, export the scene directly and use the chosen background as
`ExportSettings::clear_color`. Wrap the scene in a `SceneView` only when it needs an independent
clock, camera, framed background, or whole-opening transform inside another Murali scene.

Run `opening_scene_view` for a complete perspective-opening-to-orthographic-content handoff:

```bash
cargo run --release --example opening_scene_view
```

## Card

A `Card` is a small rounded rectangle plus centered label. Use it for diagram nodes, callouts, dashboards, and explainer scenes where you want a reusable labeled box without manually adding a background and text every time.

```rust
use murali::frontend::sangrah::composite::Card;

let card = Card::new("Retrieved Context", 2.2, 0.58)
    .with_radius(0.12)
    .with_fill(Vec4::new(0.10, 0.14, 0.18, 0.72))
    .with_stroke(0.025, Vec4::new(0.42, 0.86, 0.96, 0.9))
    .with_text_style(0.18, Vec4::new(0.95, 0.98, 1.0, 1.0));

let ids = card.add_to_scene(&mut scene, Vec3::new(0.0, 0.0, 0.0));
```

`add_to_scene` returns `CardIds`:

```rust
ids.background; // rounded rectangle TattvaId
ids.label;      // label TattvaId
ids.all();      // [background, label]
```

That makes cards easy to animate as a unit:

```rust
for id in ids.all() {
    scene.hide_tattva(id);
    timeline.animate(id).at(0.4).for_duration(0.5).appear().spawn();
}

timeline
    .animate(ids.background)
    .at(0.4)
    .for_duration(0.6)
    .draw()
    .spawn();
timeline
    .animate(ids.label)
    .at(0.75)
    .for_duration(0.35)
    .typewrite_text()
    .spawn();
```

## Axes

2D coordinate axes with tick marks.

```rust
use murali::frontend::sangrah::composite::axes::Axes;

let axes = Axes::new(x_range: (f32, f32), y_range: (f32, f32))
    .with_step(1.0)          // tick spacing on both axes
    .with_thickness(0.03)    // line thickness
    .with_tick_size(0.16)    // tick mark length
    .with_color(Vec4)
    .without_ticks();        // optional: hide ticks

scene.add_tattva(axes, Vec3::ZERO);
```

Fields can also be set directly:

```rust
let mut axes = Axes::new((-5.0, 5.0), (-3.0, 3.0));
axes.x_step = 1.0;
axes.y_step = 0.5;
axes.thickness = 0.03;
axes.tick_size = 0.18;
axes.color = Vec4::new(0.75, 0.79, 0.85, 1.0);
```

Projects to `RenderPrimitive::Line` segments — one for each axis and one per tick mark.

## NumberPlane

A full grid background with distinct axis and grid line colors.

```rust
use murali::frontend::sangrah::composite::number_plane::NumberPlane;

scene.add_tattva(
    NumberPlane::new((-5.0, 5.0), (-3.5, 3.5))
        .with_step(1.0),
    Vec3::ZERO,
);
```

Default colors: grid lines are a muted grey, axis lines are brighter. Both are configurable via struct fields:

```rust
let mut plane = NumberPlane::new((-5.0, 5.0), (-3.5, 3.5));
plane.grid_color = Vec4::new(0.25, 0.28, 0.33, 1.0);
plane.axis_color = Vec4::new(0.78, 0.82, 0.88, 1.0);
plane.grid_thickness = 0.01;
plane.axis_thickness = 0.03;
```

Typically layered behind `Axes` and a graph:

```rust
scene.add_tattva(NumberPlane::new(...), Vec3::ZERO);  // bottom
scene.add_tattva(Axes::new(...), Vec3::ZERO);          // middle
scene.add_tattva(FunctionGraph::new(...), Vec3::ZERO); // top
```

## Axes3D

3D coordinate axes for use with parametric surfaces and 3D graphs.

```rust
use murali::frontend::sangrah::composite::axes3d::Axes3D;

scene.add_tattva(
    Axes3D::new((-1.5, 1.5), (-1.5, 1.5), (-1.5, 1.5))
        .with_step(0.5)
        .with_axis_thickness(0.04),
    Vec3::ZERO,
);
```

Layout helpers arrange transformed XY bounds. In perspective scenes, `to_edge` uses the camera
frame where it intersects the world `z = 0` layout plane. Use manual 3D placement when content does
not live on that plane.
