---
sidebar_position: 5
---

# Coordinate system

Murali uses a right-handed world-space coordinate system with the origin at the center of the screen.

## Axes

- X — horizontal, positive to the right
- Y — vertical, positive upward
- Z — depth, positive toward the viewer

## Scene frames

Every scene owns its logical composition frame. Landscape is the default:

```rust
let landscape = Scene::new();
let portrait = Scene::new().with_frame(Frame::portrait());
let square = Scene::new().with_frame(Frame::square());
```

The built-in frames have exact, symmetric world-space bounds:

| Frame | Logical size | X bounds | Y bounds |
| --- | --- | --- | --- |
| `Frame::landscape()` | 16:9 | `-8` to `8` | `-4.5` to `4.5` |
| `Frame::portrait()` | 9:16 | `-4.5` to `4.5` | `-8` to `8` |
| `Frame::square()` | 1:1 | `-8` to `8` | `-8` to `8` |

Coordinates retain their ordinary world-space meaning in every frame. Selecting portrait does not rearrange or scale scene content automatically; compose deliberately within its `9:16` bounds.

All sizes (font sizes, shape radii, line thickness) are in these world units. The scene frame initializes the camera before composition, so layout helpers such as `to_edge` use the selected aspect immediately.

For complete portrait, landscape, and square workflows, see [Video Formats](./video-formats.md).

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
