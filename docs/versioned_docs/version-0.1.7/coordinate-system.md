---
sidebar_position: 5
---

# Coordinate system

Murali uses a right-handed world-space coordinate system with the origin at the center of the screen.

## Axes

- X — horizontal, positive to the right
- Y — vertical, positive upward
- Z — depth, positive toward the viewer

## Aspect ratio

Murali uses a canonical 16:9 coordinate system. The default visible world space is:

- X: `-8` to `8`
- Y: `-4.5` to `4.5`

This is exact and symmetric — not derived or approximate. `x = 4` is exactly one quarter of the screen width. `x = 8` is the right edge.

All sizes (font sizes, shape radii, line thickness) are in these world units. The camera is the single source of truth for these bounds — layout helpers like `to_edge` derive from it automatically.

## Camera

The camera uses orthographic projection by default. Moving the camera position does **not** change what's visible — only `set_view_width` does. For 2D scenes the Z position is irrelevant to the visible area:

```rust
scene.camera_mut().position = Vec3::new(0.0, 0.0, 10.0); // Z doesn't affect ortho bounds
```

For 3D scenes, you can orbit freely in the preview window.

## Positioning tattvas

Positions are passed as `Vec3` to `add_tattva`:

```rust
// Center of screen
scene.add_tattva(shape, Vec3::new(0.0, 0.0, 0.0));

// Upper left area
scene.add_tattva(shape, Vec3::new(-4.0, 2.5, 0.0));
```

## Colors

Colors are `Vec4` in linear RGBA, values from `0.0` to `1.0`:

```rust
Vec4::new(r, g, b, a)

// White
Vec4::new(1.0, 1.0, 1.0, 1.0)

// Semi-transparent red
Vec4::new(1.0, 0.0, 0.0, 0.5)
```
