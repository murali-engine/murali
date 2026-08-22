use glam::{Vec2, Vec3, Vec4};

use crate::engine::scene::Scene;
use crate::frontend::TattvaId;
use crate::frontend::layout::{Bounded, Bounds};
use crate::frontend::props::DrawableProps;
use crate::projection::{Project, ProjectionCtx, RenderPrimitive};
use crate::resource::text::layout::measure_label;

#[derive(Debug, Clone)]
pub struct EquationPart {
    pub text: String,
    pub color: Vec4,
    pub key: Option<String>,
    pub opacity: f32,
    pub scale: f32,
    pub offset: Vec3,
}

impl EquationPart {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            color: Vec4::ONE,
            key: None,
            opacity: 1.0,
            scale: 1.0,
            offset: Vec3::ZERO,
        }
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }

    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn with_opacity(mut self, opacity: f32) -> Self {
        self.opacity = opacity.clamp(0.0, 1.0);
        self
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale.max(0.05);
        self
    }

    pub fn with_offset(mut self, offset: Vec3) -> Self {
        self.offset = offset;
        self
    }

    pub fn continuity_key(&self, index: usize) -> String {
        self.key
            .clone()
            .unwrap_or_else(|| format!("{}#{index}", self.text))
    }
}

#[derive(Debug, Clone)]
pub struct EquationPartLayout {
    pub index: usize,
    pub key: String,
    pub text: String,
    pub center: Vec3,
    pub width: f32,
    pub height: f32,
    pub color: Vec4,
    pub opacity: f32,
    pub scale: f32,
}

#[derive(Debug, Clone)]
pub struct EquationLayout {
    pub parts: Vec<EquationPart>,
    pub world_height: f32,
    pub gap: f32,
}

impl EquationLayout {
    pub fn new(parts: Vec<EquationPart>, world_height: f32) -> Self {
        Self {
            parts,
            world_height,
            gap: world_height * 0.35,
        }
    }

    fn part_width(&self, text: &str, scale: f32) -> f32 {
        measure_label(text, self.world_height * scale.max(0.05), None)
            .width
            .max(self.world_height * scale.max(0.05) * 0.4)
    }

    pub fn layout_snapshot(&self) -> Vec<EquationPartLayout> {
        let widths: Vec<f32> = self
            .parts
            .iter()
            .map(|p| self.part_width(&p.text, p.scale))
            .collect();
        let total_width =
            widths.iter().sum::<f32>() + self.gap * self.parts.len().saturating_sub(1) as f32;
        let mut cursor = -total_width * 0.5;
        let mut out = Vec::with_capacity(self.parts.len());

        for (index, (part, width)) in self.parts.iter().zip(widths).enumerate() {
            let base_center = Vec3::new(cursor + width * 0.5, 0.0, 0.0);
            let height = self.world_height * part.scale.max(0.05);
            out.push(EquationPartLayout {
                index,
                key: part.continuity_key(index),
                text: part.text.clone(),
                center: base_center + part.offset,
                width,
                height,
                color: part.color,
                opacity: part.opacity.clamp(0.0, 1.0),
                scale: part.scale.max(0.05),
            });
            cursor += width + self.gap;
        }

        out
    }
}

impl Project for EquationLayout {
    fn project(&self, ctx: &mut ProjectionCtx) {
        for part in self.layout_snapshot() {
            let mut color = part.color;
            color.w *= part.opacity;
            ctx.emit(RenderPrimitive::Text {
                content: part.text,
                height: part.height,
                color,
                font_name: None,
                offset: part.center,
                rotation: 0.0,
            });
        }
    }
}

impl Bounded for EquationLayout {
    fn local_bounds(&self) -> Bounds {
        let layout = self.layout_snapshot();
        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = Vec2::splat(f32::NEG_INFINITY);
        for part in layout {
            let b = Bounds::from_center_size(
                Vec2::new(part.center.x, part.center.y),
                Vec2::new(part.width, part.height),
            );
            min.x = min.x.min(b.min.x);
            min.y = min.y.min(b.min.y);
            max.x = max.x.max(b.max.x);
            max.y = max.y.max(b.max.y);
        }

        if !min.is_finite() || !max.is_finite() {
            Bounds::default()
        } else {
            Bounds::new(min, max)
        }
    }
}

#[derive(Debug, Clone)]
pub struct VectorTypstEquation {
    pub content: String,
    pub world_height: f32,
    pub color: Vec4,
}

#[derive(Debug, Clone)]
pub struct VectorEquationHandle {
    ids: Vec<TattvaId>,
}

impl VectorEquationHandle {
    pub fn new(ids: Vec<TattvaId>) -> Self {
        Self { ids }
    }

    pub fn ids(&self) -> &[TattvaId] {
        &self.ids
    }

    pub fn into_ids(self) -> Vec<TattvaId> {
        self.ids
    }
}

impl VectorTypstEquation {
    pub fn new(content: impl Into<String>, world_height: f32) -> Self {
        Self {
            content: content.into(),
            world_height,
            color: Vec4::ONE,
        }
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }

    /// Advanced insertion API: compiles the equation into individual path tattvas and
    /// inserts them into the scene, returning their raw IDs.
    ///
    /// Prefer [`add_to_scene`](Self::add_to_scene) or `scene.add_vector_typst(...)`
    /// in authored code.
    pub fn spawn(&self, scene: &mut Scene) -> Vec<usize> {
        use crate::resource::typst_resource::compiler::TypstBackend;
        use crate::resource::typst_resource::vector::parse_svg_to_paths;

        let backend = TypstBackend::new().expect("Failed to init Typst backend");

        // Use a base font size for the SVG generation, we'll scale it in world space later
        let base_size = 32.0;
        let svg = match backend.render_to_svg(&self.content, base_size) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Equation vectorization failed: {}", e);
                return Vec::new();
            }
        };

        let symbols = match parse_svg_to_paths(&svg, self.color) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("SVG parsing failed: {}", e);
                return Vec::new();
            }
        };

        spawn_vector_symbols(scene, symbols, self.world_height / base_size)
    }

    /// Preferred authored API: insert the vector equation and keep a typed handle
    /// for later morphing/animation helpers.
    pub fn add_to_scene(&self, scene: &mut Scene) -> VectorEquationHandle {
        VectorEquationHandle::new(self.spawn(scene))
    }
}

pub type VectorEquation = VectorTypstEquation;

#[derive(Debug, Clone)]
pub struct VectorLatexEquation {
    pub content: String,
    pub world_height: f32,
    pub color: Vec4,
}

impl VectorLatexEquation {
    pub fn new(content: impl Into<String>, world_height: f32) -> Self {
        Self {
            content: content.into(),
            world_height,
            color: Vec4::ONE,
        }
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }

    /// Advanced insertion API: compiles LaTeX into path tattvas and inserts them
    /// directly into the scene, returning raw IDs.
    ///
    /// Prefer [`add_to_scene`](Self::add_to_scene) or `scene.add_vector_latex(...)`
    /// in authored code.
    pub fn spawn(&self, scene: &mut Scene) -> Vec<usize> {
        use crate::resource::latex_resource::backend::compile_latex;
        use crate::resource::typst_resource::vector::parse_svg_to_paths;

        let cache_dir = std::env::temp_dir().join("murali_latex_cache");
        let latex = match compile_latex(&self.content, &cache_dir) {
            Ok(resource) => resource,
            Err(error) => {
                eprintln!("LaTeX vectorization failed: {}", error);
                return Vec::new();
            }
        };

        let symbols = match parse_svg_to_paths(&latex.svg_content, self.color) {
            Ok(symbols) => symbols,
            Err(error) => {
                eprintln!("LaTeX SVG parsing failed: {}", error);
                return Vec::new();
            }
        };

        // The LaTeX pipeline emits path-based SVG sized around the source formula.
        // Matching the world height with a 32pt authoring baseline keeps parity with
        // the Typst vector equation path.
        let base_size = 32.0;
        spawn_vector_symbols(scene, symbols, self.world_height / base_size)
    }

    /// Preferred authored API: insert the vector equation and keep a typed handle
    /// for later morphing/animation helpers.
    pub fn add_to_scene(&self, scene: &mut Scene) -> VectorEquationHandle {
        VectorEquationHandle::new(self.spawn(scene))
    }
}

fn scale_path(path: &mut crate::frontend::sangrah::primitives::path::Path, factor: f32) {
    for seg in &mut path.segments {
        match seg {
            crate::frontend::sangrah::primitives::path::PathSegment::MoveTo(p) => *p *= factor,
            crate::frontend::sangrah::primitives::path::PathSegment::LineTo(p) => *p *= factor,
            crate::frontend::sangrah::primitives::path::PathSegment::QuadTo(c, p) => {
                *c *= factor;
                *p *= factor;
            }
            crate::frontend::sangrah::primitives::path::PathSegment::CubicTo(c1, c2, p) => {
                *c1 *= factor;
                *c2 *= factor;
                *p *= factor;
            }
        }
    }
}

fn spawn_vector_symbols(
    scene: &mut Scene,
    symbols: Vec<crate::resource::typst_resource::vector::VectorSymbol>,
    world_scale: f32,
) -> Vec<usize> {
    use crate::frontend::Tattva;

    let mut ids = Vec::new();
    for symbol in symbols {
        let mut path = symbol.path;
        scale_path(&mut path, world_scale);

        let tattva = Tattva::new(0, path);
        let id = scene.add(tattva);

        if let Some(t) = scene.get_tattva_any_mut(id) {
            let mut props = DrawableProps::write(t.props());
            props.tag = Some(symbol.key);
        }

        ids.push(id);
    }

    ids
}
