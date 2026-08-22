use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Mesh, Project, ProjectionCtx, RenderPrimitive};
use crate::resource::typst_resource::compiler::TypstBackend;
use glam::{Vec2, Vec4};
use parking_lot::Mutex;
use resvg::usvg;
use std::collections::HashMap;
use std::sync::LazyLock;

static CODEBLOCK_MEASURE_CACHE: LazyLock<Mutex<HashMap<String, glam::Vec2>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Clone)]
pub enum CodeBlockTheme {
    Dark,
    Light,
}

#[derive(Debug, Clone)]
pub enum CodeBlockSurface {
    Dark,
    Light,
}

/// A Tattva for displaying syntax-highlighted code blocks.
/// Leverages the Typst backend for high-quality rendering and highlighting.
#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub code: String,
    pub language: String,
    pub font_size: f32,
    pub color: Vec4,
    pub theme: CodeBlockTheme,
    pub surface: CodeBlockSurface,
    pub panel_inset_pt: f32,
    pub panel_radius_pt: f32,
    /// Character reveal progress: 0.0 = no characters, 1.0 = all characters
    pub char_reveal: f32,
    /// Reveal mode: true = typewriter (fixed position), false = reveal (shifting)
    pub typewriter_mode: bool,
    /// Window style controls (red, yellow, green dots)
    pub show_controls: bool,
    /// Line numbers on the left gutter
    pub show_line_numbers: bool,
    /// Optional title (e.g. filename) displayed in the center of the top bar
    pub title: Option<String>,
    /// Optional fixed content area size (excluding padding/title bar).
    /// When set, the panel uses this size directly.
    pub content_box_size: Option<Vec2>,
    /// Optional manual offset for the code content inside the panel's content area.
    pub content_offset: Vec2,
}

impl CodeBlock {
    pub fn new(code: impl Into<String>, language: impl Into<String>, font_size: f32) -> Self {
        Self {
            code: code.into(),
            language: language.into(),
            font_size,
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            theme: CodeBlockTheme::Dark,
            surface: CodeBlockSurface::Dark,
            panel_inset_pt: 16.0,
            panel_radius_pt: 10.0,
            char_reveal: 1.0,
            typewriter_mode: false,
            show_controls: true,
            show_line_numbers: true,
            title: None,
            content_box_size: None,
            content_offset: Vec2::ZERO,
        }
    }

    /// Builder-style color setter
    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }

    pub fn with_theme(mut self, theme: CodeBlockTheme) -> Self {
        self.theme = theme;
        self
    }

    pub fn with_surface(mut self, surface: CodeBlockSurface) -> Self {
        self.surface = surface;
        self
    }

    pub fn with_panel_padding(mut self, inset_pt: f32) -> Self {
        self.panel_inset_pt = inset_pt.max(0.0);
        self
    }

    pub fn with_controls(mut self, show: bool) -> Self {
        self.show_controls = show;
        self
    }

    pub fn with_line_numbers(mut self, show: bool) -> Self {
        self.show_line_numbers = show;
        self
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_content_box_size(mut self, width: f32, height: f32) -> Self {
        self.content_box_size = Some(glam::vec2(width.max(0.01), height.max(0.01)));
        self
    }

    pub fn with_content_offset(mut self, x: f32, y: f32) -> Self {
        self.content_offset = glam::vec2(x, y);
        self
    }

    fn escape_typst_string(value: &str) -> String {
        value
            .replace("\\", "\\\\")
            .replace("\"", "\\\"")
            .replace("\n", "\\n")
    }

    fn typst_color(color: Vec4) -> String {
        let r = (color.x.clamp(0.0, 1.0) * 255.0).round() as u8;
        let g = (color.y.clamp(0.0, 1.0) * 255.0).round() as u8;
        let b = (color.z.clamp(0.0, 1.0) * 255.0).round() as u8;
        format!("rgb(\"#{r:02X}{g:02X}{b:02X}\")")
    }

    fn theme_expr(&self) -> Option<String> {
        match &self.theme {
            CodeBlockTheme::Dark => Some(format!(
                "theme: bytes(\"{}\")",
                Self::escape_typst_string(include_str!(
                    "../../../../assets/code_themes/murali_dark.tmTheme"
                ))
            )),
            CodeBlockTheme::Light => Some(format!(
                "theme: bytes(\"{}\")",
                Self::escape_typst_string(include_str!(
                    "../../../../assets/code_themes/murali_light.tmTheme"
                ))
            )),
        }
    }

    fn surface_style(&self) -> (Vec4, Vec4, Vec4) {
        match self.surface {
            CodeBlockSurface::Dark => (
                Vec4::new(0.06, 0.09, 0.16, 1.0), // Deep Slate
                Vec4::new(0.80, 0.84, 0.90, 1.0), // Muted White
                Vec4::new(0.12, 0.16, 0.24, 1.0), // Title Bar slightly lighter
            ),
            CodeBlockSurface::Light => (
                Vec4::new(0.97, 0.98, 1.00, 1.0), // Ghost White
                Vec4::new(0.20, 0.25, 0.35, 1.0), // Slate Text
                Vec4::new(0.92, 0.94, 0.98, 1.0), // Title Bar slightly darker
            ),
        }
    }

    fn panel_inset_world(&self) -> f32 {
        self.font_size * (self.panel_inset_pt / 36.0)
    }

    fn title_bar_height(&self) -> f32 {
        if self.show_controls || self.title.is_some() {
            self.font_size * 0.9
        } else {
            0.0
        }
    }

    fn get_revealed_code(&self) -> String {
        if self.char_reveal >= 0.999 {
            return self.code.clone();
        }
        let char_count = self.code.chars().count();
        let reveal_count = (char_count as f32 * self.char_reveal.clamp(0.0, 1.0)).ceil() as usize;
        self.code.chars().take(reveal_count).collect()
    }

    fn line_count_for(&self, code: &str) -> f32 {
        code.lines().count().max(1) as f32
    }

    fn authored_code_height(&self, code: &str) -> f32 {
        self.line_count_for(code) * self.font_size * 1.22
    }

    fn fallback_measured_size(&self, code: &str) -> glam::Vec2 {
        let longest_row_chars = code
            .lines()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0) as f32;
        let glyph_width = self.font_size * 0.62;
        glam::vec2(
            longest_row_chars * glyph_width,
            self.authored_code_height(code),
        )
    }

    fn line_number_gutter_width(&self, code: &str) -> f32 {
        if !self.show_line_numbers {
            return 0.0;
        }
        let line_count = self.line_count_for(code);
        let digits = line_count.max(1.0).log10().floor() + 1.0;
        let glyph_width = self.font_size * 0.62;
        digits * glyph_width + self.font_size * 1.1
    }

    fn build_typst_source(&self, code: &str, text_fallback: Vec4) -> String {
        let escaped_code = Self::escape_typst_string(code);
        let font_size_pt = (self.font_size * 36.0).max(1.0);
        let mut raw_args = vec![
            format!("lang: \"{}\"", Self::escape_typst_string(&self.language)),
            "block: true".to_string(),
        ];
        if let Some(theme_expr) = self.theme_expr() {
            raw_args.push(theme_expr);
        }

        let body_fn = format!("raw(\"{}\", {})", escaped_code, raw_args.join(", "));

        format!(
            "#set text(font: \"Courier New\", size: {font_size_pt:.2}pt, fill: {})\n#show raw: it => block(fill: none, inset: 0pt, radius: 0pt, it)\n#{}",
            Self::typst_color(text_fallback),
            body_fn
        )
    }

    fn build_line_number_source(&self, code: &str, color: Vec4) -> String {
        let line_count = code.lines().count().max(1);
        let mut numbers = String::new();
        for i in 1..=line_count {
            numbers.push_str(&format!("{i}\n"));
        }
        let font_size_pt = (self.font_size * 36.0).max(1.0);
        format!(
            "#set text(font: \"Courier New\", size: {font_size_pt:.2}pt, fill: {})\n#text(fill: {}, \"{}\")",
            Self::typst_color(color),
            Self::typst_color(color),
            Self::escape_typst_string(&numbers)
        )
    }

    fn measured_typst_size(&self, typst_source: &str) -> Option<glam::Vec2> {
        let cache_key = format!("{:.4}::{typst_source}", self.font_size);
        if let Some(size) = CODEBLOCK_MEASURE_CACHE.lock().get(&cache_key).copied() {
            return Some(size);
        }

        let backend = TypstBackend::new().ok()?;
        let svg = backend
            .render_to_svg(typst_source, self.font_size * 36.0)
            .ok()?;
        let tree = usvg::Tree::from_str(&svg, &usvg::Options::default()).ok()?;
        let svg_size = tree.size();
        if svg_size.height() <= 0.0 {
            return None;
        }

        // Return the natural size in world units. 1 unit = 36 pts.
        let size = glam::vec2(svg_size.width() / 36.0, svg_size.height() / 36.0);
        CODEBLOCK_MEASURE_CACHE.lock().insert(cache_key, size);
        Some(size)
    }
}

impl Project for CodeBlock {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let revealed_code = self.get_revealed_code();
        let (panel_fill, text_fallback, title_bar_fill) = self.surface_style();
        let gutter_color = text_fallback * 0.4;

        let revealed_typst = self.build_typst_source(&revealed_code, text_fallback);
        let line_numbers_typst = if self.show_line_numbers {
            Some(self.build_line_number_source(&revealed_code, gutter_color))
        } else {
            None
        };

        let authored_code_height = if self.typewriter_mode {
            self.authored_code_height(&self.code)
        } else {
            self.authored_code_height(&revealed_code)
        };

        let (panel_size, code_render_height) = if let Some(box_size) = self.content_box_size {
            (box_size, authored_code_height)
        } else {
            let full_typst = self.build_typst_source(&self.code, text_fallback);
            let full_size = self
                .measured_typst_size(&full_typst)
                .unwrap_or_else(|| self.fallback_measured_size(&self.code));
            let revealed_size = self
                .measured_typst_size(&revealed_typst)
                .unwrap_or_else(|| self.fallback_measured_size(&revealed_code));
            let panel_size = if self.typewriter_mode {
                full_size
            } else {
                revealed_size
            };
            let code_render_height = panel_size.y;
            (panel_size, code_render_height)
        };

        let inset = self.panel_inset_world();
        let bar_h = self.title_bar_height();
        let gutter_w = self.line_number_gutter_width(if self.typewriter_mode {
            &self.code
        } else {
            &revealed_code
        });
        let gutter_gap = if self.show_line_numbers {
            self.font_size * 0.7
        } else {
            0.0
        };

        let total_h = panel_size.y + inset * 2.0 + bar_h;
        let total_w = panel_size.x + inset * 2.0 + gutter_w + gutter_gap;

        // 1. Base Panel
        let panel_mesh = Mesh::rectangle(total_w, total_h, panel_fill);
        ctx.emit(RenderPrimitive::Mesh(
            panel_mesh
                .as_ref()
                .translated(glam::Vec3::new(0.0, 0.0, -0.002)),
        ));

        // 2. Title Bar
        if bar_h > 0.0 {
            let bar_mesh = Mesh::rectangle(total_w, bar_h, title_bar_fill);
            let bar_y = (total_h - bar_h) * 0.5;
            ctx.emit(RenderPrimitive::Mesh(
                bar_mesh
                    .as_ref()
                    .translated(glam::Vec3::new(0.0, bar_y, -0.001)),
            ));

            // Window Buttons
            if self.show_controls {
                let btn_radius = bar_h * 0.18;
                let btn_y = bar_y;
                let btn_x_start = -total_w * 0.5 + inset * 0.8;

                let colors = [
                    Vec4::new(1.0, 0.37, 0.34, 1.0),  // Red
                    Vec4::new(1.0, 0.74, 0.18, 1.0),  // Yellow
                    Vec4::new(0.15, 0.79, 0.25, 1.0), // Green
                ];

                for (i, color) in colors.iter().enumerate() {
                    let dot = Mesh::circle(btn_radius, 16, *color);
                    let x = btn_x_start + i as f32 * (btn_radius * 2.8);
                    ctx.emit(RenderPrimitive::Mesh(
                        dot.as_ref().translated(glam::Vec3::new(x, btn_y, 0.0)),
                    ));
                }
            }

            // Title Text
            if let Some(ref title) = self.title {
                ctx.emit(RenderPrimitive::Text {
                    content: title.clone(),
                    height: self.font_size * 0.32,
                    color: text_fallback * 0.7,
                    font_name: None,
                    offset: glam::Vec3::new(0.0, bar_y, 0.0),
                    rotation: 0.0,
                });
            }
        }

        // 3. Code content and optional line numbers
        let typst_source = revealed_typst;
        let content_center_x = -total_w * 0.5
            + inset
            + gutter_w
            + gutter_gap
            + panel_size.x * 0.5
            + self.content_offset.x;
        let y_offset = -bar_h * 0.5 + self.content_offset.y;

        ctx.emit(RenderPrimitive::Typst {
            source: typst_source,
            height: code_render_height,
            color: Vec4::ONE, // Use pure white to preserve syntax colors when tint is false
            offset: glam::Vec3::new(content_center_x, y_offset, 0.0),
            normalize: false, // Don't normalize blocks
            tint: false,      // Preserves .tmTheme colors
        });

        if let Some(line_number_source) = line_numbers_typst {
            let gutter_center_x = -total_w * 0.5 + inset + gutter_w * 0.5;
            ctx.emit(RenderPrimitive::Typst {
                source: line_number_source,
                height: authored_code_height,
                color: Vec4::ONE,
                offset: glam::Vec3::new(gutter_center_x, y_offset, 0.0),
                normalize: false,
                tint: false,
            });
        }
    }
}

impl Bounded for CodeBlock {
    fn local_bounds(&self) -> Bounds {
        let size = if let Some(box_size) = self.content_box_size {
            box_size
        } else {
            let (_, text_fallback, _) = self.surface_style();
            let measured = if self.typewriter_mode {
                let typst = self.build_typst_source(&self.code, text_fallback);
                self.measured_typst_size(&typst)
                    .unwrap_or_else(|| self.fallback_measured_size(&self.code))
            } else {
                let revealed = self.get_revealed_code();
                let typst = self.build_typst_source(&revealed, text_fallback);
                self.measured_typst_size(&typst)
                    .unwrap_or_else(|| self.fallback_measured_size(&revealed))
            };
            measured
        };

        let inset = self.panel_inset_world();
        let bar_h = self.title_bar_height();

        Bounds::from_center_size(
            glam::Vec2::ZERO,
            glam::vec2(size.x + inset * 2.0, size.y + inset * 2.0 + bar_h),
        )
    }
}
