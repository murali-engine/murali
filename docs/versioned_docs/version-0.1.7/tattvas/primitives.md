---
sidebar_position: 1
---

# Primitives

Basic geometric shapes and procedural primitives. All live under `murali::frontend::collection::primitives`.

## Quick Start Example

```rust
use murali::frontend::collection::primitives::{
use murali::colors::*;
use murali::positions::*;
    circle::Circle,
    square::Square,
    line::Line,
};
use glam::{Vec3, Vec4};

let mut scene = Scene::new();

// Add a circle
let circle_id = scene.add_tattva(
    Circle::new(1.0, 48, Vec4::new(0.2, 0.6, 0.3, 1.0)),
    Vec3::ZERO,
);

// Add a square
let square_id = scene.add_tattva(
    Square::new(1.2, Vec4::new(0.9, 0.3, 0.3, 1.0)),
    Vec3::new(2.5, 0.0, 0.0),
);
```

## Basic Shapes

### Circle

A filled circle with optional stroke outline.

```rust
use murali::frontend::collection::primitives::circle::Circle;

Circle::new(radius: f32, segments: u32, color: Vec4)
```

**Parameters:**
- `radius` - Circle radius in world units
- `segments` - Number of segments for smoothness (32 for small, 64+ for large)
- `color` - Fill color (RGBA, 0.0-1.0)

**With stroke:**
```rust
Circle::new(1.0, 64, fill_color)
    .with_stroke(0.04, stroke_color)
```

**Best for:** Dots, markers, filled shapes, emphasis elements.

**Animation pairing:** Use `.appear()` or `.fade_to()`, not `.draw()`.

### Square

A filled square.

```rust
use murali::frontend::collection::primitives::square::Square;

Square::new(size: f32, color: Vec4)
```

**Parameters:**
- `size` - Side length in world units
- `color` - Fill color (RGBA)

**Best for:** Boxes, tiles, grid elements, UI shapes.

**Animation pairing:** Use `.appear()` or `.scale_to()`.

### Rectangle

A filled rectangle with independent width and height.

```rust
use murali::frontend::collection::primitives::rectangle::Rectangle;

Rectangle::new(width: f32, height: f32, color: Vec4)
```

**Parameters:**
- `width` - Width in world units
- `height` - Height in world units
- `color` - Fill color (RGBA)

**Best for:** Bars, panels, backgrounds, non-square shapes.

**Animation pairing:** Use `.appear()` or `.scale_to()` with non-uniform scaling.

### Rounded Rectangle

A filled rectangle with rounded corners.

```rust
use murali::frontend::collection::primitives::rounded_rectangle::RoundedRectangle;

RoundedRectangle::new(width: f32, height: f32, radius: f32, color: Vec4)
```

**Parameters:**
- `width` - Width in world units
- `height` - Height in world units
- `radius` - Corner radius in world units
- `color` - Fill color (RGBA)

**Best for:** Concept cards, UI panels, callouts, and diagram nodes.

**Animation pairing:** Use `.appear()`, `.write()`, or `.scale_to()`.

### Ellipse

A filled ellipse with independent radii.

```rust
use murali::frontend::collection::primitives::ellipse::Ellipse;

Ellipse::new(rx: f32, ry: f32, segments: u32, color: Vec4)
```

**Parameters:**
- `rx` - Horizontal radius
- `ry` - Vertical radius
- `segments` - Smoothness (32-64 typical)
- `color` - Fill color (RGBA)

**Best for:** Ovals, stretched circles, organic shapes.

**Animation pairing:** Use `.appear()` or `.scale_to()`.

### Polygon

Regular polygon with n sides.

```rust
use murali::frontend::collection::primitives::polygon::Polygon;

// Regular polygon with n sides
Polygon::regular(sides: u32, radius: f32, color: Vec4)
```

**Parameters:**
- `sides` - Number of sides (3 = triangle, 5 = pentagon, 6 = hexagon, etc.)
- `radius` - Distance from center to vertex
- `color` - Fill color (RGBA)

**Examples:**
```rust
// Triangle
Polygon::regular(3, 1.0, color)

// Pentagon
Polygon::regular(5, 1.0, color)

// Hexagon
Polygon::regular(6, 1.0, color)

// Octagon
Polygon::regular(8, 1.0, color)
```

**Best for:** Geometric shapes, tessellations, icons.

## Lines and Paths

### Line

A straight line segment with thickness.

```rust
use murali::frontend::collection::primitives::line::Line;

Line::new(start: Vec3, end: Vec3, thickness: f32, color: Vec4)
```

**Parameters:**
- `start` - Start point in 3D space
- `end` - End point in 3D space
- `thickness` - Line thickness in world units
- `color` - Line color (RGBA)

**With dashed pattern:**
```rust
Line::new(start, end, 0.04, color)
    .with_dash(dash_length, gap_length)
```

**Technical note:** Lines are rendered via GPU line pipeline—geometry is generated in the vertex shader, not uploaded as vertex data. This makes them very efficient.

**Best for:** Connections, axes, guides, underlines.

**Animation pairing:** Use `.draw()` to progressively reveal.

### Arrow

A line with an arrowhead.

```rust
use murali::frontend::collection::primitives::arrow::Arrow;
use glam::Vec2;

// Default tip proportions
Arrow::with_default_tip(start: Vec2, end: Vec2, thickness: f32, color: Vec4)

// Custom tip
Arrow::new(start, end, thickness, tip_length, tip_width, color)
```

**Parameters:**
- `start` - Start point (2D)
- `end` - End point (2D)
- `thickness` - Shaft thickness
- `tip_length` - Arrowhead length (custom only)
- `tip_width` - Arrowhead width (custom only)
- `color` - Arrow color (RGBA)

**Note:** `start` and `end` are 2D. The arrow is placed at the `Vec3` position passed to `add_tattva`.

**Best for:** Vectors, directions, flow indicators, pointers.

**Animation pairing:** Use `.draw()` to reveal from start to end.

### Path

A freeform path built from multiple points.

```rust
use murali::frontend::collection::primitives::path::Path;

let path = Path::new(vec![
    2.0 * LEFT,
    2.0 * UP,
    2.0 * RIGHT,
], thickness, color);
```

**Parameters:**
- `points` - Vector of 3D points defining the path
- `thickness` - Path thickness
- `color` - Path color (RGBA)

**Best for:** Custom curves, freeform shapes, traced paths, signatures.

**Animation pairing:** Use `.draw()` to progressively reveal along the path.

## 3D Primitives

### Cube

A 3D wireframe cube.

```rust
use murali::frontend::collection::primitives::cube::Cube;

Cube::new(size: f32, color: Vec4)
```

**Parameters:**
- `size` - Edge length
- `color` - Wireframe color (RGBA)

**Best for:** 3D scene orientation, coordinate system visualization, bounding boxes.

**Note:** Requires perspective camera for proper 3D effect.

## Procedural Primitives

### NoisyCircle

A circular contour displaced by Perlin noise, creating organic, animated shapes.

```rust
use murali::frontend::collection::primitives::noisy_circle::NoisyCircle;

// Basic usage
NoisyCircle::new(radius: f32, color: Vec4)

// With customization
NoisyCircle::new(1.5, color)
    .with_samples(180)                    // Smoothness
    .with_noise_amplitude(0.2)            // Displacement amount
    .with_noise_frequency(1.0)            // Noise detail
    .with_phase(0.0)                      // Animation phase
    .with_morph_speed(0.8)                // Animation speed
    .with_stroke(0.04, stroke_color)
```

**Key methods:**
- `.with_samples(n)` - More samples = smoother (default: 180)
- `.with_noise_amplitude(a)` - How much the circle wobbles (default: 0.18)
- `.with_noise_frequency(f)` - Noise detail level (default: 1.0)
- `.with_phase(p)` - Animation time parameter
- `.with_morph_speed(s)` - How fast it morphs (default: 0.8)
- `.with_noise_seed(s)` - Different random variation

**Multicolor gradient:**
```rust
NoisyCircle::new(1.5, color)
    .multicolor(vec![
        Vec4::new(0.2, 0.6, 0.9, 1.0),
        GOLD_C,
        GREEN_B,
    ])
    .with_gradient_cycles(2.0)            // Color cycles around circle
    .with_gradient_motion_rate(0.5)       // Color animation speed
```

**Best for:** Organic shapes, cells, blobs, animated backgrounds, energy effects.

**Animation tip:** Animate the `.phase` property over time for continuous morphing:
```rust
timeline.call_during(0.0, 10.0, move |scene, t| {
    if let Some(circle) = scene.get_tattva_typed_mut::<NoisyCircle>(id) {
        circle.phase = t * 5.0;  // Morph over time
    }
});
```

**Alias:** `PerlinNoiseCircle` (same thing, more explicit name)

### NoisyHorizon

A wavy horizon line with vertical fill, perfect for landscapes or data visualizations.

```rust
use murali::frontend::collection::primitives::noisy_horizon::NoisyHorizon;

NoisyHorizon::new(
    x_start: f32,      // Left edge
    x_end: f32,        // Right edge
    baseline_y: f32,   // Top of the wave
    bottom_y: f32,     // Bottom fill level
    palette: Vec<Vec4> // Color gradient
)
```

**Customization:**
```rust
NoisyHorizon::new(-5.0, 5.0, 1.0, -2.0, palette)
    .with_samples(200)                    // Smoothness
    .with_noise_amplitude(0.3)            // Wave height
    .with_noise_frequency(1.5)            // Wave detail
    .with_phase(0.0)                      // Animation phase
    .with_gradient_cycles(2.0)            // Color repetitions
    .with_gradient_motion_rate(0.4)       // Color animation
```

**Best for:** Landscapes, terrain, water surfaces, data fills, backgrounds.

**Animation tip:** Animate phase for flowing/moving effect:
```rust
timeline.call_during(0.0, 10.0, move |scene, t| {
    if let Some(horizon) = scene.get_tattva_typed_mut::<NoisyHorizon>(id) {
        horizon.phase = t * 2.0;  // Flowing motion
    }
});
```

### ParticleBelt

An animated belt of particles orbiting around a center, like an asteroid belt or particle ring.

```rust
use murali::frontend::collection::primitives::particle_belt::ParticleBelt;

// Basic usage
ParticleBelt::new(radius: f32)

// With customization
ParticleBelt::new(3.0)
    .with_particle_count(200)
    .with_band_width(0.6)
    .with_particle_size_range(0.01, 0.06)
    .with_palette(vec![
        Vec4::new(BLUE_A.x, BLUE_A.y, BLUE_A.z, 0.95),
        Vec4::new(GOLD_C.x, GOLD_C.y, GOLD_C.z, 0.88),
    ])
    .with_orbit_speed(0.8)
    .with_clockwise_ratio(0.5)            // 50% clockwise, 50% counter
    .with_band_breathing(0.08, 1.2)       // Pulsing effect
    .with_radial_jitter(0.10, 2.4)        // Wobble
    .with_phase(0.0)
```

**Key methods:**
- `.with_particle_count(n)` - Number of particles (default: 160)
- `.with_band_width(w)` - Width of the belt (default: 0.5)
- `.with_particle_size_range(min, max)` - Particle size variation
- `.with_palette(colors)` - Color palette for particles
- `.with_orbit_speed(s)` - How fast particles orbit (default: 0.8)
- `.with_clockwise_ratio(r)` - 0.0 = all counter-clockwise, 1.0 = all clockwise
- `.all_clockwise()` / `.all_anticlockwise()` - Convenience methods
- `.with_band_breathing(amp, rate)` - Pulsing effect
- `.with_radial_jitter(amp, rate)` - Random wobble
- `.with_angular_spread(angle)` - Partial ring (< TAU)
- `.with_seed(s)` - Different random distribution

**Best for:** Asteroid belts, particle rings, orbital systems, decorative elements, sci-fi effects.

**Animation tip:** Animate phase for continuous orbital motion:
```rust
timeline.call_during(0.0, 20.0, move |scene, t| {
    if let Some(belt) = scene.get_tattva_typed_mut::<ParticleBelt>(id) {
        belt.phase = t * 3.0;  // Continuous orbit
    }
});
```

**Alias:** `AsteroidBelt` (same thing, thematic name)

## Choosing the Right Primitive

| Need | Use | Why |
|---|---|---|
| Simple filled shape | Circle, Square, Rectangle | Fast, clean, standard |
| Geometric shape | Polygon | Flexible, regular shapes |
| Connection/line | Line, Arrow | Clear direction, efficient |
| Custom curve | Path | Full control over shape |
| 3D visualization | Cube | Spatial orientation |
| Organic/animated | NoisyCircle | Living, breathing effect |
| Landscape/terrain | NoisyHorizon | Natural, flowing |
| Particle system | ParticleBelt | Dynamic, complex motion |

## Animation Best Practices

### Filled Shapes (Circle, Square, Rectangle, Ellipse, Polygon)
✅ **Use:** `.appear()`, `.fade_to()`, `.scale_to()`
❌ **Don't use:** `.draw()` (no natural draw progression)

### Lines and Paths (Line, Arrow, Path)
✅ **Use:** `.draw()`, `.undraw()`
❌ **Don't use:** `.appear()` alone (less interesting)

### Procedural Primitives (NoisyCircle, NoisyHorizon, ParticleBelt)
✅ **Use:** Animate `.phase` property with `call_during()`
✅ **Also use:** `.appear()` for initial reveal
❌ **Don't use:** `.draw()` (not path-like)

## Common Patterns

### Grid of Shapes
```rust
let spacing = 2.0;
for row in 0..3 {
    for col in 0..3 {
        let x = (col as f32 - 1.0) * spacing;
        let y = (row as f32 - 1.0) * spacing;
        scene.add_tattva(
            Circle::new(0.4, 32, color),
            Vec3::new(x, y, 0.0),
        );
    }
}
```

### Connected Shapes with Lines
```rust
let circle1_id = scene.add_tattva(Circle::new(0.5, 32, color), 2.0 * LEFT);
let circle2_id = scene.add_tattva(Circle::new(0.5, 32, color), 2.0 * RIGHT);

let line_id = scene.add_tattva(
    Line::new(
        2.0 * LEFT,
        2.0 * RIGHT,
        0.04,
        color
    ),
    Vec3::ZERO,
);
```

### Animated Organic Shape
```rust
let noisy_id = scene.add_tattva(
    NoisyCircle::new(1.5, color)
        .with_noise_amplitude(0.25)
        .multicolor(vec![color1, color2, color3]),
    Vec3::ZERO,
);

// Animate the morphing
timeline.call_during(0.0, 10.0, move |scene, t| {
    if let Some(circle) = scene.get_tattva_typed_mut::<NoisyCircle>(noisy_id) {
        circle.phase = t * 4.0;
    }
});
```

## Gotchas

**Circle segments too low:**
- Small circles: 32 segments is fine
- Large circles: Use 64+ for smoothness
- Very large: 128+ to avoid visible facets

**Line thickness:**
- Too thin (< 0.02): May be invisible or flicker
- Good range: 0.03 - 0.08
- Too thick (> 0.2): Looks chunky

**Procedural primitive performance:**
- NoisyCircle: Moderate cost (many line segments)
- NoisyHorizon: Moderate cost (filled mesh)
- ParticleBelt: Higher cost (many particles)
- Use sparingly in scenes with many objects

**Path points:**
- Too few points: Jagged curves
- Too many points: Performance cost
- Sweet spot: 10-50 points for most curves

## What's Next?

- **[Text](./text)** - Labels, LaTeX, Typst, code blocks
- **[Composite](./composite)** - Complex multi-part tattvas
- **[Graphs](./graphs)** - Parametric curves and surfaces
- **[Properties](./properties)** - Common tattva properties and transforms
