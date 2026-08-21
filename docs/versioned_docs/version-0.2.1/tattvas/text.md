---
sidebar_position: 2
---

# Text

Text is essential for mathematical animation, teaching visuals, and explanatory content. Murali provides several text tattvas, each optimized for different use cases.

All text tattvas live under `murali::frontend::collection::text`.

If you are choosing between text systems for a real scene, start here and then read [Math](./math), [Animations](../animations), and [Examples](../examples/showcase).

## Quick Decision Guide

| Need | Use | Why |
|---|---|---|
| Simple text, titles, labels | `Label` | Fast, lightweight, no dependencies |
| Mathematical equations | `Latex` or `Typst` | Industry standard (Latex) or modern (Typst) |
| Rich formatted text | `Typst` | Modern typesetting, no external tools |
| Syntax-highlighted code | `CodeBlock` | Built-in highlighting, monospace |
| Extruded capital letters | `Letter3D` | True depth, texture mapping, and independent 3D transforms |
| Letter-shaped dissolve | `LetterParticles3D` | Deterministic particles sampled from the glyph interior |

## Which Text Tattva Should I Pick?

Use `Label` when the text is short, direct, and scene-native: titles, axis labels, captions, counters, callouts, and UI-like annotations.

Use `Latex` when you want standard academic math rendering and are comfortable depending on a local LaTeX toolchain.

Use `Typst` when you want richer formatting, modern typesetting, or math/text content without leaning on a full LaTeX install.

Use `CodeBlock` when the code itself is part of the visual explanation and syntax highlighting matters.

Use `Letter3D` when a capital letter needs to behave as a real 3D object. Pair it with
`LetterParticles3D` when the solid glyph should dissolve into a seekable particle effect.

The most common beginner mistake is overusing `Latex` for everything. For titles, annotations, and short labels, `Label` is usually the better tool.

## Label

Lightweight glyph-based text using fontdue. Best for short strings, numbers, titles, and UI text.

```rust
use murali::frontend::collection::text::label::Label;

Label::new(text: impl Into<String>, world_height: f32)
```

**Parameters:**
- `text` - The text content
- `world_height` - Font size in world units (not pixels!)

**Basic usage:**
```rust
let title = Label::new("Hello, World!", 0.36)
    .with_color(Vec4::new(0.9, 0.9, 0.9, 1.0));

scene.add_tattva(title, Vec3::new(0.0, 3.0, 0.0));
```

### Styling

```rust
// Set color
Label::new("Colored Text", 0.32)
    .with_color(Vec4::new(0.2, 0.6, 0.9, 1.0))

// Common colors
let white = Vec4::new(1.0, 1.0, 1.0, 1.0);
let black = Vec4::new(0.0, 0.0, 0.0, 1.0);
let red = Vec4::new(0.9, 0.3, 0.3, 1.0);
let blue = Vec4::new(0.2, 0.6, 0.9, 1.0);
```

### Custom Fonts

`Label` uses Murali's lightweight glyph text pipeline. By default it renders with the built-in font, but you can now register additional fonts from file paths and opt into them per label.

Register a font once before preview or export:

```rust
use murali::register_font_path;

register_font_path(
    "Satoshi",
    "/absolute/path/to/Satoshi-Regular.otf",
)?;
```

Then select it on any label:

```rust
let title = Label::new("KAVRIQ", 0.8)
    .with_font("Satoshi")
    .with_color(Vec4::new(0.9, 0.9, 0.9, 1.0));
```

Notes:
- Font selection is optional. If you do not call `.with_font(...)`, Murali uses the default embedded font.
- The font name passed to `.with_font(...)` must match the name used in `register_font_path(...)`.
- Path-based registration is the recommended workflow for reproducible renders across machines.
- Register fonts before the scene is rendered so the text backend can load and atlas them lazily on first use.

### Text Animations

Label supports two reveal modes for character-by-character animations:

#### Reveal Mode (Default)

Text reveals from center, characters shift into place:

```rust
let label = Label::new("Hello, World!", 0.32)
    .with_char_reveal(0.0);  // Start hidden

scene.add_tattva(label, Vec3::ZERO);

// Animate the reveal
timeline.call_during(0.0, 2.0, move |scene, t| {
    if let Some(label) = scene.get_tattva_typed_mut::<Label>(label_id) {
        label.char_reveal = t;  // 0.0 to 1.0
    }
});
```

Or use the built-in animation:
```rust
timeline
    .animate(label_id)
    .at(0.0)
    .for_duration(2.0)
    .ease(Ease::OutCubic)
    .reveal_text()
    .spawn();
```

#### Typewriter Mode

Text appears left-to-right, position stays fixed:

```rust
let label = Label::new("Hello, World!", 0.32)
    .with_char_reveal(0.0);

scene.add_tattva(label, Vec3::ZERO);

// Enable typewriter mode
if let Some(label) = scene.get_tattva_typed_mut::<Label>(label_id) {
    label.typewriter_mode = true;
}

// Animate
timeline
    .animate(label_id)
    .at(0.0)
    .for_duration(2.0)
    .ease(Ease::Linear)  // Usually linear for typing
    .typewrite_text()
    .spawn();
```

### Font Sizing

**Typical sizes:**
- **0.24-0.28** - Body text, annotations
- **0.32-0.36** - Titles, headings
- **0.40-0.50** - Large titles, emphasis
- **0.18-0.22** - Small text, subscripts

**Rule of thumb:** Start with 0.32 and adjust based on camera view width.

### When to Use Label

✅ **Use Label for:**
- Titles and headings
- UI text and labels
- Numbers and simple annotations
- Axis labels
- Short text (< 100 characters)
- When you need fast rendering

❌ **Don't use Label for:**
- Mathematical equations (use Latex or Typst)
- Rich formatted text (use Typst)
- Code with syntax highlighting (use CodeBlock)
- Very long paragraphs (performance)

### Performance

Label is the fastest text option:
- No external tools required
- Direct glyph rendering
- Minimal overhead
- Good for many labels in one scene

**Best animation pairings:**
- `appear()` for titles and short labels
- `fade_to(...)` for subtle emphasis/de-emphasis
- `reveal_text()` for teaching-style reveals
- `typewrite_text()` when a left-to-right typing feel matters

---

## Letter3D

`Letter3D` extrudes a font outline into front, back, and side faces. Holes in letters such as `A`,
`P`, `Q`, and `R` remain open. The initial API intentionally accepts only ASCII capital letters
`A` through `Z`.

```rust
use glam::{Quat, Vec3, Vec4};
use murali::frontend::collection::text::letter3d::Letter3D;
use murali::{BuiltinTexture, TextureImage};

let letter = Letter3D::new('K', 2.6, 0.72)?
    .with_texture(TextureImage::builtin(BuiltinTexture::WhiteMarble))
    .with_face_colors(
        Vec4::new(1.0, 0.98, 0.91, 1.0),
        Vec4::new(0.58, 0.55, 0.49, 1.0),
        Vec4::new(0.42, 0.39, 0.34, 1.0),
    );

let letter_id = scene.add_tattva(letter, Vec3::new(0.0, 4.0, 12.0));
scene.set_rotation(letter_id, Quat::from_rotation_y(0.8));
```

The constructor parameters are `(character, world_height, extrusion_depth)`. Add letters
separately when each character needs its own fall, tumble, impact timing, or final position.

The embedded Inter font is used by `Letter3D::new`. Use `Letter3D::from_font_path` to build a
capital from another TrueType or OpenType font:

```rust
let letter = Letter3D::from_font_path('K', 2.6, 0.72, "assets/fonts/brand.ttf")?;
```

Use a perspective camera and `DepthMode::World` when the extrusion and rotations should read as
3D. Image textures use planar UVs on the front and back and perimeter UVs on the side walls.
Without a texture, the configured face colors render directly.

### Falling Letters

`Letter3D` uses ordinary Tattva transforms, so no special motion API is required:

```rust
timeline
    .animate(letter_id)
    .at(0.4)
    .for_duration(1.4)
    .ease(Ease::InCubic)
    .move_to(Vec3::new(0.0, -0.4, 0.0))
    .spawn();
timeline
    .animate(letter_id)
    .at(0.4)
    .for_duration(1.4)
    .ease(Ease::InOutCubic)
    .rotate_to(Quat::IDENTITY)
    .spawn();
```

Starting beyond the camera position lets the letter enter from behind the viewer. Keep the camera
far plane large enough to include the initial position.

### Letter Particles

`LetterParticles3D` samples a deterministic particle volume from the same glyph silhouette:

```rust
use murali::frontend::collection::text::letter3d::LetterParticles3D;

let particles = LetterParticles3D::new('K', 2.6, 0.72, 260)?
    .with_motion(4.8, 2.35, 1.0)
    .with_particle_size(0.028)
    .with_color(Vec4::new(0.78, 0.79, 0.77, 0.88))
    .with_palette([TEAL, BLUE, PINK, GOLD]);
let particles_id = scene.add_tattva(particles, final_position);

timeline
    .animate(particles_id)
    .at(4.5)
    .for_duration(2.8)
    .ease(Ease::InOutCubic)
    .scatter_letter_particles()
    .spawn();
```

At scatter `0.0`, the particles occupy the solid glyph silhouette. A common transition is to show
the particle tattva and quickly fade the solid `Letter3D` at the same timestamp. The scatter verb
is deterministic and supports seeking and export. Use `.letter_particle_scatter_to(value)` for a
partial or reversible-looking state.

For the reusable drop, shake, particle, and tagline choreography, see the beta
[`Opening`](./composite#opening) composite and the `kavriq_opening` example. Place its scene in a
`SceneView` only when the complete opening needs an independent clock or must transform as one
object inside a parent scene.

## Latex

Renders mathematical equations using LaTeX. Industry standard for academic and scientific content.

```rust
use murali::frontend::collection::text::latex::Latex;

Latex::new(source: impl Into<String>, world_height: f32)
```

**Parameters:**
- `source` - LaTeX source code
- `world_height` - Height of the rendered equation in world units

### Basic Usage

```rust
// Simple equation
let eq = Latex::new(r"\frac{a}{b} + \sqrt{c}", 0.32)
    .with_color(Vec4::new(0.9, 0.9, 0.9, 1.0));

scene.add_tattva(eq, Vec3::ZERO);
```

### Common LaTeX Patterns

```rust
// Fractions
Latex::new(r"\frac{numerator}{denominator}", 0.32)

// Square root
Latex::new(r"\sqrt{x}", 0.32)

// Superscript and subscript
Latex::new(r"x^2 + y_i", 0.32)

// Greek letters
Latex::new(r"\alpha + \beta = \gamma", 0.32)

// Integrals
Latex::new(r"\int_0^\infty e^{-x} dx", 0.32)

// Summation
Latex::new(r"\sum_{i=1}^n i^2", 0.32)

// Matrices
Latex::new(r"\begin{bmatrix} a & b \\ c & d \end{bmatrix}", 0.32)

// Aligned equations
Latex::new(r"\begin{aligned} x &= 2 \\ y &= 3 \end{aligned}", 0.32)
```

### Text Reveal Animation

Latex supports character reveal like Label:

```rust
let eq = Latex::new(r"E = mc^2", 0.36)
    .with_char_reveal(0.0);

scene.add_tattva(eq, Vec3::ZERO);

timeline
    .animate(eq_id)
    .at(0.0)
    .for_duration(2.0)
    .reveal_text()
    .spawn();
```

### System Requirements

**Required tools:**
- `latex` - LaTeX compiler
- `dvisvgm` - DVI to SVG converter

**Installation:**
```bash
# macOS
brew install --cask mactex

# Ubuntu/Debian
sudo apt-get install texlive-latex-base texlive-latex-extra dvisvgm

# Arch Linux
sudo pacman -S texlive-core texlive-bin
```

### Caching

Latex results are cached to disk by source hash:
- First render: Slow (compiles LaTeX)
- Subsequent renders: Fast (loads from cache)
- Cache location: System temp directory

### When to Use Latex

✅ **Use Latex for:**
- Mathematical equations
- Academic/scientific notation
- Complex formulas
- When you need LaTeX-specific packages
- Industry-standard math rendering

❌ **Don't use Latex for:**
- Simple text (use Label)
- When you can't install LaTeX
- Real-time editing (slow first render)
- Code (use CodeBlock)

### Gotchas

**Escaping backslashes:**
```rust
// Use raw strings (r"...") to avoid double-escaping
Latex::new(r"\frac{a}{b}", 0.32)  // ✅ Good

// Without raw string, you need double backslashes
Latex::new("\\frac{a}{b}", 0.32)  // ✅ Also works but ugly
```

**Compilation errors:**
- Invalid LaTeX syntax will cause render failure
- Check terminal output for LaTeX error messages
- Test equations in a LaTeX editor first

**Best animation pairings:**
- `reveal_text()` for equation reveals
- `fade_to(...)` for staging supporting formulas
- vector-formula morph workflows when continuity matters more than static raster output

---

## Typst

Modern typesetting system with built-in math support. No external tools required!

Typst is a strong middle ground between simple labels and full LaTeX. It works well for formatted teaching text, math-heavy notes, and scenes where you want cleaner authoring than raw LaTeX strings.

```rust
use murali::frontend::collection::text::typst::Typst;

Typst::new(source: impl Into<String>, world_height: f32)
```

**Parameters:**
- `source` - Typst markup
- `world_height` - Height in world units

### Basic Usage

```rust
// Math equation
let eq = Typst::new(r"$\frac{a}{b} + \sqrt{c}$", 0.32)
    .with_color(Vec4::new(0.9, 0.9, 0.9, 1.0));

scene.add_tattva(eq, Vec3::ZERO);
```

### Typst Math Syntax

```rust
// Fractions
Typst::new(r"$a/b$", 0.32)  // Inline
Typst::new(r"$frac(a, b)$", 0.32)  // Function style

// Square root
Typst::new(r"$sqrt(x)$", 0.32)

// Superscript and subscript
Typst::new(r"$x^2 + y_i$", 0.32)

// Greek letters
Typst::new(r"$alpha + beta = gamma$", 0.32)

// Integrals
Typst::new(r"$integral_0^infinity e^(-x) dif x$", 0.32)

// Summation
Typst::new(r"$sum_(i=1)^n i^2$", 0.32)

// Matrices
Typst::new(r"$mat(a, b; c, d)$", 0.32)
```

### Rich Text

Typst supports formatted text, not just math:

```rust
// Bold and italic
Typst::new("*Bold* and _italic_ text", 0.28)

// Mixed text and math
Typst::new("The equation $E = mc^2$ is famous", 0.28)

// Lists
Typst::new("- Item 1\n- Item 2\n- Item 3", 0.24)
```

### Caching

Typst results are cached in an LRU cache:
- Cache size: 128 entries
- Key: source + height
- Much faster than Latex (no external process)

### When to Use Typst

✅ **Use Typst for:**
- Mathematical equations (no LaTeX installation needed)
- Rich formatted text
- Modern, clean syntax
- When you want fast compilation
- Mixed text and math

❌ **Don't use Typst for:**
- Simple text (use Label - it's faster)
- When you need specific LaTeX packages
- Code with syntax highlighting (use CodeBlock)

### Typst vs Latex

| Feature | Typst | Latex |
|---|---|---|
| Installation | None (built-in) | Requires LaTeX + dvisvgm |
| Speed | Fast | Slow (first render) |
| Syntax | Modern, clean | Traditional, verbose |
| Math support | Excellent | Excellent |
| Packages | Limited | Extensive |
| Learning curve | Easier | Steeper |

**Recommendation:** Use Typst unless you specifically need LaTeX packages or have existing LaTeX code.

---

## CodeBlock

Syntax-highlighted code using Typst's `#raw` block.

```rust
use murali::frontend::collection::text::code_block::{
    CodeBlock, CodeBlockSurface, CodeBlockTheme,
};

CodeBlock::new(
    code: impl Into<String>,
    language: impl Into<String>,
    world_height: f32
)
```

**Parameters:**
- `code` - Source code
- `language` - Language identifier (e.g., "rust", "python", "javascript")
- `world_height` - Visible code text size in world units

### Basic Usage

```rust
let code = CodeBlock::new(
    "fn main() {\n    println!(\"Hello, world!\");\n}",
    "rust",
    0.24
)
.with_theme(CodeBlockTheme::Dark)
.with_surface(CodeBlockSurface::Dark)
.with_title("main.rs")
.with_content_box_size(4.8, 1.6)
.with_content_offset(0.0, -0.04);

scene.add_tattva(code, Vec3::new(1.5, -0.4, 0.0));
```

### Supported Languages

Common languages:
- `"rust"` - Rust
- `"python"` - Python
- `"javascript"` / `"js"` - JavaScript
- `"typescript"` / `"ts"` - TypeScript
- `"c"` - C
- `"cpp"` / `"c++"` - C++
- `"java"` - Java
- `"go"` - Go
- `"bash"` / `"sh"` - Shell scripts
- `"json"` - JSON
- `"yaml"` - YAML
- `"toml"` - TOML
- `"sql"` - SQL

### Multi-line Code

```rust
let code = r#"
fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}
"#;

let code_block = CodeBlock::new(code, "rust", 0.22);
scene.add_tattva(code_block, Vec3::ZERO);
```

### Sizing

**Typical per-line heights:**
- **0.20-0.22** - Small code snippets
- **0.24-0.26** - Standard code blocks
- **0.28-0.30** - Large, emphasized code

**Note:** `world_height` controls the code text size. If you want a deterministic panel footprint, pair it with `.with_content_box_size(width, height)`.

### Layout Controls

- Move the whole block by changing the position passed to `scene.add_tattva(...)`
- Move only the code inside the panel with `.with_content_offset(x, y)`
- Use `.with_content_box_size(width, height)` when you want explicit authored block sizing

### When to Use CodeBlock

✅ **Use CodeBlock for:**
- Code snippets
- Algorithm demonstrations
- Programming tutorials
- Terminal output
- Configuration files

❌ **Don't use CodeBlock for:**
- Plain text (use Label)
- Math equations (use Latex or Typst)
- Prose (use Typst)

### Gotchas

**Sizing behavior:**
- If you provide `.with_content_box_size(...)`, that explicit authored size drives the block layout
- Without an explicit content box, Murali falls back to a measured or estimated natural size
- For production scenes, prefer explicit sizing when the exact panel footprint matters

**Syntax highlighting:**
- Powered by Typst's built-in highlighter
- This API currently exposes two built-in palettes: dark and light
- The code panel surface is also chosen from built-in dark and light presets
- Custom themes are intentionally deferred until the API matures

**Large code blocks:**
- CodeBlock is currently rasterized to a GPU texture before rendering
- Very large snippets can hit the device texture-size limit on some hardware
- For now, prefer camera framing, shorter snippets, or smaller `world_height` if a block gets too large
- A renderer-level fix for oversized blocks is planned for a future iteration

---

## Comparison Table

| Feature | Label | Latex | Typst | CodeBlock |
|---|---|---|---|---|
| **Speed** | Fastest | Slow (first) | Fast | Fast |
| **Dependencies** | None | latex, dvisvgm | None | None |
| **Math support** | None | Excellent | Excellent | None |
| **Syntax highlighting** | No | No | No | Yes |
| **Rich text** | No | Limited | Yes | No |
| **Best for** | Simple text | Math equations | Math + text | Code |
| **Reveal animation** | Yes | Yes | No | Yes |
| **Caching** | N/A | Disk | LRU (128) | LRU (128) |

## Tooling And Performance Notes

- `Label` is the lightest-weight option and scales best when you have many text objects.
- `Latex` has the heaviest setup cost because it depends on external tooling and a compile step.
- `Typst` is a good default when you need richer formatting but want a smoother authoring story than LaTeX.
- `CodeBlock` is specialized: excellent for code visuals, not a replacement for general-purpose text.

## Animation Patterns

### Fade In Text

```rust
// Works for all text types
timeline
    .animate(text_id)
    .at(0.0)
    .for_duration(1.0)
    .ease(Ease::OutCubic)
    .appear()
    .spawn();
```

### Typewriter Effect (Label, Latex, CodeBlock)

```rust
let code_id = scene.add_tattva(
    CodeBlock::new("fn main() {}", "rust", 0.24),
    Vec3::ZERO,
);

timeline
    .animate(code_id)
    .at(0.0)
    .for_duration(2.0)
    .ease(Ease::Linear)
    .typewrite_text()
    .spawn();
```

### Reveal Effect (Label, Latex)

```rust
timeline
    .animate(text_id)
    .at(0.0)
    .for_duration(1.5)
    .ease(Ease::OutCubic)
    .reveal_text()
    .spawn();
```

### Sequential Text Appearance

```rust
let texts = vec![
    "First line",
    "Second line",
    "Third line",
];

let mut current_time = 0.0;
for text in texts {
    let id = scene.add_tattva(
        Label::new(text, 0.28),
        Vec3::new(0.0, current_y, 0.0),
    );

    timeline
        .animate(id)
        .at(current_time)
        .for_duration(0.8)
        .appear()
        .spawn();

    current_time += 1.0;
    current_y -= 0.5;
}
```

## Best Practices

### Choosing Text Type

1. **Start with Label** - Use for 90% of text needs
2. **Use Typst for math** - If you don't have LaTeX installed
3. **Use Latex for math** - If you have existing LaTeX code or need specific packages
4. **Use CodeBlock for code** - Always, for syntax highlighting

### Font Sizing

```rust
// Establish a size hierarchy
const TITLE_SIZE: f32 = 0.40;
const HEADING_SIZE: f32 = 0.32;
const BODY_SIZE: f32 = 0.24;
const SMALL_SIZE: f32 = 0.20;
```

### Color Consistency

```rust
// Define a color palette
const TEXT_PRIMARY: Vec4 = Vec4::new(0.95, 0.95, 0.95, 1.0);
const TEXT_SECONDARY: Vec4 = Vec4::new(0.70, 0.75, 0.80, 1.0);
const TEXT_ACCENT: Vec4 = Vec4::new(0.20, 0.60, 0.90, 1.0);
```

### Performance

- **Many labels?** Label is fastest
- **Complex equations?** Cache is your friend (both Latex and Typst)
- **Real-time editing?** Use Typst over Latex
- **Hundreds of text objects?** Consider Label over rendered text

## Common Patterns

### Title and Subtitle

```rust
let title = Label::new("Main Title", 0.40)
    .with_color(Vec4::new(0.95, 0.95, 0.95, 1.0));
let title_id = scene.add_tattva(title, Vec3::ZERO);
scene.to_edge(title_id, Direction::Up, 0.35);

let subtitle = Label::new("Subtitle text", 0.24)
    .with_color(Vec4::new(0.75, 0.80, 0.85, 1.0));
scene.add_tattva(subtitle, Vec3::new(0.0, 2.8, 0.0));
```

## Related Docs

- [Math](./math)
- [Animations](../animations)
- [Which API Should I Use?](../which-api-should-i-use)
- [Examples](../examples/text_and_math)

### Equation with Label

```rust
// Equation
let eq_id = scene.add_tattva(
    Typst::new(r"$E = mc^2$", 0.36),
    Vec3::ZERO,
);

// Label below
let label_id = scene.add_tattva(
    Label::new("Einstein's famous equation", 0.22),
    Vec3::new(0.0, -1.0, 0.0),
);
```

### Code with Title

```rust
let title_id = scene.add_tattva(
    Label::new("Fibonacci Function", 0.32),
    Vec3::new(0.0, 2.5, 0.0),
);

let code_id = scene.add_tattva(
    CodeBlock::new(
        "fn fib(n: u32) -> u32 {\n    // ...\n}",
        "rust",
        0.24
    ),
    Vec3::new(0.0, 0.0, 0.0),
);
```

## Troubleshooting

**Text not visible:**
- Check color alpha (should be 1.0 for opaque)
- Verify camera position and view width
- Check if text is within frame bounds

**Latex compilation fails:**
- Verify latex and dvisvgm are installed
- Check terminal for LaTeX error messages
- Test equation in a LaTeX editor first
- Use raw strings: `r"\frac{a}{b}"`

**Text too small/large:**
- Adjust `world_height` parameter
- Remember: world units, not pixels
- Check camera view width
- Typical range: 0.20 to 0.50

**Typewriter mode not working:**
- Must set `typewriter_mode = true` on Label struct
- Use `.typewrite_text()` animation
- Start with `char_reveal = 0.0`

## What's Next?

- **[Math](./math)** - Equation layout and vector formulas
- **[Tables](./tables)** - Structured data display
- **[Primitives](./primitives)** - Shapes to combine with text
- **[Animations](../animations)** - Text animation techniques
