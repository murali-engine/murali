---
sidebar_position: 10
---

# 3D Prop Assets

Murali's `Prop3D` workflow is intended for video props: apples, balls, books, animals, tools,
furniture, robots, and other objects that can be placed in an explainer scene. It is not meant to
turn Murali into a game engine.

```rust
use murali::frontend::sangrah::primitives::prop3d::Prop3D;

let prop_id = scene.add_tattva(
    Prop3D::from_file("assets/props/apple.glb")?,
    Vec3::new(0.0, 0.0, 0.0),
);
```

`Prop3D` can load local `.glb` files and loose `.gltf` files. Prefer `.glb` assets when possible. A
`.glb` file is the bundled binary form of glTF, so geometry, materials, and often textures can
travel together as one file.

## Inspect Before Composing

From a cloned Murali repository, use the model inspector to check an asset before placing it in a
production scene:

```bash
cargo run --example model_inspector -- demo-apple
cargo run --example model_inspector -- /absolute/path/to/model.glb --rot-x -20
```

The inspector centers and frames the model automatically, rotates it continuously, and reports its
mesh count and source dimensions. Drag to orbit and use the wheel to zoom. Run it with `--help` for
scale, XYZ rotation, camera, FOV, rotation-speed, and fit controls.

`demo-apple` is repository shorthand. The inspector first checks the supplied path, then
`assets/props/<name>`. For a directory it prefers `scene.glb` or `scene.gltf`; otherwise it accepts
the directory's single `.glb` or `.gltf` file. Production `Prop3D` code still receives an explicit
asset path.

Some sites label downloads as "glTF" but provide a folder containing `.gltf`, `.bin`, and texture
images. Murali can load those with `Prop3D::from_gltf(...)` or `Prop3D::from_file(...)`, but the
referenced `.bin` and texture files must stay beside the `.gltf` file.

## Recommended Sources

Start with sources that have simple licensing and lightweight assets:

| Source | Best For | License Notes |
| --- | --- | --- |
| [Poly Haven](https://polyhaven.com/models) | High-quality realistic props | CC0 assets. Free for personal and commercial use. |
| [Quaternius](https://quaternius.com/) | Stylized low-poly props, characters, vehicles, and environments | CC0 assets. Good fit for child-friendly and educational videos. |
| [Kenney](https://kenney.nl/assets) | Simple playful game-style objects | Public domain / CC0 assets. Clean and lightweight. |
| [Sketchfab](https://sketchfab.com/3d-models) | Very large 3D model library | Check each model's license before use. Prefer CC0 or clearly compatible assets. |

For Murali examples and reusable project assets, prefer CC0 models. They are easier to redistribute,
modify, include in videos, and use commercially.

## What To Download

Use `.glb` when the source provides it:

```text
assets/props/apple.glb
assets/props/ball.glb
assets/props/tiger.glb
```

If the source only provides loose glTF, keep the whole folder together:

```text
assets/props/apple/apple.gltf
assets/props/apple/apple.bin
assets/props/apple/base_color.png
```

Avoid starting with complex assets that depend on many external texture files, rigs, skeletal
animation, physics metadata, or engine-specific materials. Murali props should be easy to place,
scale, rotate, move, bounce, and render inside a video scene.

The bundled `assets/props/demo-pyramid.glb` is intentionally tiny and faceted. Each face uses a
different base color so the object reads as 3D in Murali's current unlit renderer.

The bundled `assets/props/demo-apple` folder exists for repository examples and inspector testing.
Demo props are not embedded runtime resources; applications should keep their own models in their
project assets.

## If An Asset Fails To Load

Check these first:

- The path points to a local `.glb` or `.gltf` file.
- The file exists inside your project assets directory.
- For `.gltf`, the referenced `.bin` and texture files are still beside the `.gltf` file.
- The asset license allows your intended use.
- The model is reasonably lightweight for video rendering.
- If the asset came from Sketchfab or another marketplace, the download actually includes a glTF or
  `.glb` version.

Good fallback sources are Poly Haven, Quaternius, and Kenney. They are useful places to find clean
models when an asset URL is broken, unavailable, too heavy, or has unclear licensing.
