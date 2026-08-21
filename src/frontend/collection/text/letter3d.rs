use std::path::Path as FsPath;
use std::sync::Arc;

use glam::{Vec2, Vec3, Vec4, vec2, vec3};
use lyon_tessellation::path::iterator::PathIterator;
use lyon_tessellation::path::{Path as LyonPath, PathEvent};
use lyon_tessellation::{FillOptions, FillRule, FillTessellator, VertexBuffers};
use thiserror::Error;
use ttf_parser::OutlineBuilder;

use crate::backend::renderer::vertex::mesh::MeshVertex;
use crate::backend::renderer::vertex::text::TextVertex;
use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Mesh, Project, ProjectionCtx, RenderPrimitive};
use crate::resource::texture::TextureImage;

const DEFAULT_FONT: &[u8] = include_bytes!("../../../resource/assets/fonts/Inter-Regular.ttf");

#[derive(Debug, Error)]
pub enum Letter3DError {
    #[error("Letter3D currently supports only ASCII capital letters A-Z, got {0:?}")]
    UnsupportedCharacter(char),
    #[error("Letter3D height and depth must be finite and greater than zero")]
    InvalidDimensions,
    #[error("failed to read font at {path}: {source}")]
    FontRead {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse the selected font")]
    FontParse,
    #[error("the selected font does not contain {0:?}")]
    MissingGlyph(char),
    #[error("the selected font has no usable capital-height metrics")]
    MissingCapitalMetrics,
    #[error("failed to tessellate glyph {0:?}")]
    Tessellation(char),
    #[error(transparent)]
    Texture(#[from] anyhow::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GlyphFace {
    Front,
    Back,
    Side,
}

#[derive(Debug, Clone, Copy)]
struct GlyphVertex {
    position: Vec3,
    uv: Vec2,
    face: GlyphFace,
}

#[derive(Debug, Clone)]
struct GlyphGeometry {
    vertices: Vec<GlyphVertex>,
    indices: Vec<u32>,
    front_indices: Vec<u32>,
    bounds: Bounds,
}

/// An extruded, depth-tested capital letter built from a font outline.
///
/// `Letter3D` currently accepts ASCII `A` through `Z`. Add each character as a
/// separate tattva when letters need independent motion.
#[derive(Debug, Clone)]
pub struct Letter3D {
    pub character: char,
    pub height: f32,
    pub depth: f32,
    pub front_color: Vec4,
    pub back_color: Vec4,
    pub side_color: Vec4,
    geometry: Arc<GlyphGeometry>,
    texture: Option<Arc<TextureImage>>,
}

impl Letter3D {
    pub fn new(character: char, height: f32, depth: f32) -> Result<Self, Letter3DError> {
        Self::from_font_bytes(character, height, depth, DEFAULT_FONT)
    }

    pub fn from_font_path(
        character: char,
        height: f32,
        depth: f32,
        path: impl AsRef<FsPath>,
    ) -> Result<Self, Letter3DError> {
        let path = path.as_ref();
        let bytes = std::fs::read(path).map_err(|source| Letter3DError::FontRead {
            path: path.display().to_string(),
            source,
        })?;
        Self::from_font_bytes(character, height, depth, &bytes)
    }

    fn from_font_bytes(
        character: char,
        height: f32,
        depth: f32,
        bytes: &[u8],
    ) -> Result<Self, Letter3DError> {
        validate_inputs(character, height, depth)?;
        let geometry = build_glyph_geometry(bytes, character, height, depth)?;
        Ok(Self {
            character,
            height,
            depth,
            front_color: Vec4::new(0.96, 0.94, 0.87, 1.0),
            back_color: Vec4::new(0.68, 0.65, 0.57, 1.0),
            side_color: Vec4::new(0.48, 0.45, 0.39, 1.0),
            geometry: Arc::new(geometry),
            texture: None,
        })
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.front_color = color;
        self.back_color = shade(color, 0.72);
        self.side_color = shade(color, 0.5);
        self
    }

    pub fn with_face_colors(mut self, front: Vec4, back: Vec4, side: Vec4) -> Self {
        self.front_color = front;
        self.back_color = back;
        self.side_color = side;
        self
    }

    pub fn with_texture(mut self, texture: TextureImage) -> Self {
        self.texture = Some(Arc::new(texture));
        self
    }

    pub(crate) fn with_shared_texture(mut self, texture: Arc<TextureImage>) -> Self {
        self.texture = Some(texture);
        self
    }

    pub fn with_texture_path(mut self, path: impl AsRef<FsPath>) -> Result<Self, Letter3DError> {
        self.texture = Some(Arc::new(TextureImage::from_path(path)?));
        Ok(self)
    }

    pub fn width(&self) -> f32 {
        self.geometry.bounds.size().x
    }

    pub fn texture(&self) -> Option<&TextureImage> {
        self.texture.as_deref()
    }
}

impl Project for Letter3D {
    fn project(&self, ctx: &mut ProjectionCtx) {
        if let Some(texture) = &self.texture {
            let vertices = self
                .geometry
                .vertices
                .iter()
                .map(|vertex| TextVertex {
                    position: vertex.position.into(),
                    uv: vertex.uv.into(),
                    color: self.color_for(vertex.face).into(),
                })
                .collect();
            ctx.emit(RenderPrimitive::Mesh(Mesh::from_textured_vertices(
                vertices,
                self.geometry.indices.clone(),
                texture.clone(),
            )));
        } else {
            let vertices = self
                .geometry
                .vertices
                .iter()
                .map(|vertex| MeshVertex {
                    position: vertex.position.into(),
                    color: self.color_for(vertex.face).into(),
                })
                .collect();
            ctx.emit(RenderPrimitive::Mesh(Mesh::from_tessellation(
                vertices,
                self.geometry.indices.clone(),
            )));
        }
    }
}

impl Letter3D {
    fn color_for(&self, face: GlyphFace) -> Vec4 {
        match face {
            GlyphFace::Front => self.front_color,
            GlyphFace::Back => self.back_color,
            GlyphFace::Side => self.side_color,
        }
    }
}

impl Bounded for Letter3D {
    fn local_bounds(&self) -> Bounds {
        self.geometry.bounds
    }
}

/// A particle volume sampled from the same capital-letter outline as `Letter3D`.
/// Animate `scatter` from `0.0` to `1.0` to dissolve the silhouette.
#[derive(Debug, Clone)]
pub struct LetterParticles3D {
    pub character: char,
    pub scatter: f32,
    pub phase: f32,
    pub distance: f32,
    pub rise: f32,
    pub curl: f32,
    pub particle_size: f32,
    pub color: Vec4,
    pub seed: f32,
    palette: Arc<Vec<Vec4>>,
    origins: Arc<Vec<Vec3>>,
    bounds: Bounds,
}

impl LetterParticles3D {
    pub fn new(
        character: char,
        height: f32,
        depth: f32,
        particle_count: usize,
    ) -> Result<Self, Letter3DError> {
        Self::from_font_bytes(character, height, depth, particle_count, DEFAULT_FONT)
    }

    pub fn from_font_path(
        character: char,
        height: f32,
        depth: f32,
        particle_count: usize,
        path: impl AsRef<FsPath>,
    ) -> Result<Self, Letter3DError> {
        let path = path.as_ref();
        let bytes = std::fs::read(path).map_err(|source| Letter3DError::FontRead {
            path: path.display().to_string(),
            source,
        })?;
        Self::from_font_bytes(character, height, depth, particle_count, &bytes)
    }

    fn from_font_bytes(
        character: char,
        height: f32,
        depth: f32,
        particle_count: usize,
        bytes: &[u8],
    ) -> Result<Self, Letter3DError> {
        validate_inputs(character, height, depth)?;
        let geometry = build_glyph_geometry(bytes, character, height, depth)?;
        let origins = sample_glyph_volume(&geometry, depth, particle_count.max(1));
        Ok(Self {
            character,
            scatter: 0.0,
            phase: 0.0,
            distance: 4.0,
            rise: 2.2,
            curl: 0.8,
            particle_size: height * 0.018,
            color: Vec4::new(0.83, 0.82, 0.78, 0.86),
            seed: character as u32 as f32 * 0.173,
            palette: Arc::new(default_particle_palette()),
            origins: Arc::new(origins),
            bounds: geometry.bounds,
        })
    }

    pub fn with_motion(mut self, distance: f32, rise: f32, curl: f32) -> Self {
        self.distance = distance.max(0.0);
        self.rise = rise;
        self.curl = curl.max(0.0);
        self
    }

    pub fn with_particle_size(mut self, size: f32) -> Self {
        self.particle_size = size.max(0.001);
        self
    }

    pub fn with_color(mut self, color: Vec4) -> Self {
        self.color = color;
        self
    }

    /// Sets the colors particles transition toward as the glyph scatters.
    /// An empty palette leaves the current palette unchanged.
    pub fn with_palette<I>(mut self, palette: I) -> Self
    where
        I: IntoIterator<Item = Vec4>,
    {
        let palette = palette.into_iter().collect::<Vec<_>>();
        if !palette.is_empty() {
            self.palette = Arc::new(palette);
        }
        self
    }

    pub fn with_seed(mut self, seed: f32) -> Self {
        self.seed = seed;
        self
    }

    pub fn particle_count(&self) -> usize {
        self.origins.len()
    }

    pub fn palette(&self) -> &[Vec4] {
        &self.palette
    }
}

impl Project for LetterParticles3D {
    fn project(&self, ctx: &mut ProjectionCtx) {
        let progress = self.scatter.clamp(0.0, 1.0);
        let eased = 1.0 - (1.0 - progress).powi(3);
        const DISC_SEGMENTS: usize = 12;
        let mut vertices = Vec::with_capacity(self.origins.len() * (DISC_SEGMENTS + 1));
        let mut indices = Vec::with_capacity(self.origins.len() * DISC_SEGMENTS * 3);

        for (index, origin) in self.origins.iter().enumerate() {
            let h0 = hash01(self.seed + index as f32 * 17.31);
            let h1 = hash01(self.seed + index as f32 * 41.73 + 0.7);
            let h2 = hash01(self.seed + index as f32 * 83.19 + 1.9);
            let angle = h0 * std::f32::consts::TAU;
            let radial = self.distance * (0.35 + 0.9 * h1) * eased;
            let curl =
                self.curl * (self.phase * (0.8 + h2) + angle + progress * 4.0).sin() * progress;
            let center = *origin
                + vec3(
                    angle.cos() * radial + curl,
                    angle.sin() * radial * 0.48 + self.rise * eased,
                    (h2 - 0.5) * radial * 0.75 + curl * 0.35,
                );
            let color_start = 0.06 + h1 * 0.12;
            let color_end = 0.52 + h1 * 0.14;
            let color_mix = smoothstep(color_start, color_end, progress);
            let color = self.color.lerp(
                particle_color(index, self.color.w, &self.palette),
                color_mix,
            );

            let shrink_start = 0.68 + h0 * 0.16;
            let shrink = 1.0 - smoothstep(shrink_start, 1.0, progress);
            let size = self.particle_size
                * (0.65 + h2 * 0.8)
                * (1.0 + smoothstep(0.0, 0.45, progress) * 0.15)
                * shrink;
            append_disc(
                &mut vertices,
                &mut indices,
                center,
                size,
                shade(color, 0.82 + h1 * 0.18),
                DISC_SEGMENTS,
            );
        }

        ctx.emit(RenderPrimitive::Mesh(Mesh::from_tessellation(
            vertices, indices,
        )));
    }
}

impl Bounded for LetterParticles3D {
    fn local_bounds(&self) -> Bounds {
        let expansion = self.distance + self.curl + self.particle_size;
        Bounds::new(
            self.bounds.min - vec2(expansion, expansion),
            self.bounds.max + vec2(expansion, expansion + self.rise.abs()),
        )
    }
}

#[derive(Debug, Clone, Copy)]
enum OutlineCommand {
    Move(Vec2),
    Line(Vec2),
    Quad(Vec2, Vec2),
    Cubic(Vec2, Vec2, Vec2),
    Close,
}

#[derive(Default)]
struct CommandCollector {
    commands: Vec<OutlineCommand>,
}

impl OutlineBuilder for CommandCollector {
    fn move_to(&mut self, x: f32, y: f32) {
        self.commands.push(OutlineCommand::Move(vec2(x, y)));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.commands.push(OutlineCommand::Line(vec2(x, y)));
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.commands
            .push(OutlineCommand::Quad(vec2(x1, y1), vec2(x, y)));
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.commands.push(OutlineCommand::Cubic(
            vec2(x1, y1),
            vec2(x2, y2),
            vec2(x, y),
        ));
    }

    fn close(&mut self) {
        self.commands.push(OutlineCommand::Close);
    }
}

fn validate_inputs(character: char, height: f32, depth: f32) -> Result<(), Letter3DError> {
    if !character.is_ascii_uppercase() {
        return Err(Letter3DError::UnsupportedCharacter(character));
    }
    if !height.is_finite() || !depth.is_finite() || height <= 0.0 || depth <= 0.0 {
        return Err(Letter3DError::InvalidDimensions);
    }
    Ok(())
}

fn build_glyph_geometry(
    font_bytes: &[u8],
    character: char,
    height: f32,
    depth: f32,
) -> Result<GlyphGeometry, Letter3DError> {
    let face = ttf_parser::Face::parse(font_bytes, 0).map_err(|_| Letter3DError::FontParse)?;
    let glyph_id = face
        .glyph_index(character)
        .ok_or(Letter3DError::MissingGlyph(character))?;
    let capital_id = face
        .glyph_index('H')
        .ok_or(Letter3DError::MissingCapitalMetrics)?;
    let capital_bounds = face
        .glyph_bounding_box(capital_id)
        .ok_or(Letter3DError::MissingCapitalMetrics)?;
    let glyph_bounds = face
        .glyph_bounding_box(glyph_id)
        .ok_or(Letter3DError::MissingGlyph(character))?;
    let capital_height = (capital_bounds.y_max - capital_bounds.y_min) as f32;
    if capital_height <= f32::EPSILON {
        return Err(Letter3DError::MissingCapitalMetrics);
    }

    let scale = height / capital_height;
    let center_x = (glyph_bounds.x_min as f32 + glyph_bounds.x_max as f32) * 0.5;
    let center_y = (capital_bounds.y_min as f32 + capital_bounds.y_max as f32) * 0.5;
    let transform = |point: Vec2| vec2((point.x - center_x) * scale, (point.y - center_y) * scale);

    let mut collector = CommandCollector::default();
    face.outline_glyph(glyph_id, &mut collector)
        .ok_or(Letter3DError::MissingGlyph(character))?;
    let path = build_lyon_path(&collector.commands, transform);
    let tolerance = (height * 0.0025).max(0.001);

    let mut tessellator = FillTessellator::new();
    let mut fill: VertexBuffers<lyon_tessellation::math::Point, u16> = VertexBuffers::new();
    tessellator
        .tessellate_path(
            &path,
            &FillOptions::default()
                .with_fill_rule(FillRule::NonZero)
                .with_tolerance(tolerance),
            &mut lyon_tessellation::geometry_builder::simple_builder(&mut fill),
        )
        .map_err(|_| Letter3DError::Tessellation(character))?;

    let min = transform(vec2(glyph_bounds.x_min as f32, glyph_bounds.y_min as f32));
    let max = transform(vec2(glyph_bounds.x_max as f32, glyph_bounds.y_max as f32));
    let size = (max - min).max(Vec2::splat(f32::EPSILON));
    let uv_for = |point: Vec2| vec2((point.x - min.x) / size.x, 1.0 - (point.y - min.y) / size.y);
    let half_depth = depth * 0.5;
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for point in &fill.vertices {
        let point = vec2(point.x, point.y);
        vertices.push(GlyphVertex {
            position: point.extend(half_depth),
            uv: uv_for(point),
            face: GlyphFace::Front,
        });
    }
    let front_indices = fill
        .indices
        .iter()
        .copied()
        .map(u32::from)
        .collect::<Vec<_>>();
    indices.extend(front_indices.iter().copied());

    let back_offset = vertices.len() as u32;
    for point in &fill.vertices {
        let point = vec2(point.x, point.y);
        vertices.push(GlyphVertex {
            position: point.extend(-half_depth),
            uv: uv_for(point),
            face: GlyphFace::Back,
        });
    }
    for triangle in front_indices.chunks_exact(3) {
        indices.extend([
            back_offset + triangle[2],
            back_offset + triangle[1],
            back_offset + triangle[0],
        ]);
    }

    for contour in flattened_contours(&path, tolerance) {
        let perimeter = contour_perimeter(&contour).max(f32::EPSILON);
        let mut distance = 0.0;
        for index in 0..contour.len() {
            let a = contour[index];
            let b = contour[(index + 1) % contour.len()];
            let edge_length = a.distance(b);
            if edge_length <= f32::EPSILON {
                continue;
            }
            let u0 = distance / perimeter;
            distance += edge_length;
            let u1 = distance / perimeter;
            let base = vertices.len() as u32;
            vertices.extend([
                GlyphVertex {
                    position: a.extend(half_depth),
                    uv: vec2(u0, 0.0),
                    face: GlyphFace::Side,
                },
                GlyphVertex {
                    position: b.extend(half_depth),
                    uv: vec2(u1, 0.0),
                    face: GlyphFace::Side,
                },
                GlyphVertex {
                    position: b.extend(-half_depth),
                    uv: vec2(u1, 1.0),
                    face: GlyphFace::Side,
                },
                GlyphVertex {
                    position: a.extend(-half_depth),
                    uv: vec2(u0, 1.0),
                    face: GlyphFace::Side,
                },
            ]);
            indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }

    Ok(GlyphGeometry {
        vertices,
        indices,
        front_indices,
        bounds: Bounds::new(min, max),
    })
}

fn build_lyon_path(commands: &[OutlineCommand], transform: impl Fn(Vec2) -> Vec2) -> LyonPath {
    let mut builder = LyonPath::builder();
    let mut open = false;
    for command in commands {
        match *command {
            OutlineCommand::Move(point) => {
                if open {
                    builder.end(false);
                }
                let point = transform(point);
                builder.begin(lyon_tessellation::math::point(point.x, point.y));
                open = true;
            }
            OutlineCommand::Line(point) if open => {
                let point = transform(point);
                builder.line_to(lyon_tessellation::math::point(point.x, point.y));
            }
            OutlineCommand::Quad(control, point) if open => {
                let control = transform(control);
                let point = transform(point);
                builder.quadratic_bezier_to(
                    lyon_tessellation::math::point(control.x, control.y),
                    lyon_tessellation::math::point(point.x, point.y),
                );
            }
            OutlineCommand::Cubic(control_a, control_b, point) if open => {
                let control_a = transform(control_a);
                let control_b = transform(control_b);
                let point = transform(point);
                builder.cubic_bezier_to(
                    lyon_tessellation::math::point(control_a.x, control_a.y),
                    lyon_tessellation::math::point(control_b.x, control_b.y),
                    lyon_tessellation::math::point(point.x, point.y),
                );
            }
            OutlineCommand::Close if open => {
                builder.end(true);
                open = false;
            }
            _ => {}
        }
    }
    if open {
        builder.end(false);
    }
    builder.build()
}

fn flattened_contours(path: &LyonPath, tolerance: f32) -> Vec<Vec<Vec2>> {
    let mut contours = Vec::new();
    let mut current = Vec::new();
    for event in path.iter().flattened(tolerance) {
        match event {
            PathEvent::Begin { at } => {
                current.clear();
                current.push(vec2(at.x, at.y));
            }
            PathEvent::Line { to, .. } => current.push(vec2(to.x, to.y)),
            PathEvent::End { close, .. } => {
                if close && current.len() >= 3 {
                    contours.push(std::mem::take(&mut current));
                } else {
                    current.clear();
                }
            }
            _ => {}
        }
    }
    contours
}

fn contour_perimeter(contour: &[Vec2]) -> f32 {
    (0..contour.len())
        .map(|index| contour[index].distance(contour[(index + 1) % contour.len()]))
        .sum()
}

fn sample_glyph_volume(geometry: &GlyphGeometry, depth: f32, count: usize) -> Vec<Vec3> {
    let triangles = geometry
        .front_indices
        .chunks_exact(3)
        .filter_map(|indices| {
            let a = geometry.vertices[indices[0] as usize].position.truncate();
            let b = geometry.vertices[indices[1] as usize].position.truncate();
            let c = geometry.vertices[indices[2] as usize].position.truncate();
            let area = (b - a).perp_dot(c - a).abs() * 0.5;
            (area > f32::EPSILON).then_some((a, b, c, area))
        })
        .collect::<Vec<_>>();
    let total_area: f32 = triangles.iter().map(|triangle| triangle.3).sum();
    if total_area <= f32::EPSILON {
        return Vec::new();
    }

    (0..count)
        .map(|index| {
            let target = hash01(index as f32 * 31.17 + 0.31) * total_area;
            let mut accumulated = 0.0;
            let &(a, b, c, _) = triangles
                .iter()
                .find(|triangle| {
                    accumulated += triangle.3;
                    accumulated >= target
                })
                .unwrap_or_else(|| triangles.last().expect("glyph has a triangle"));
            let r1 = hash01(index as f32 * 73.91 + 1.7).sqrt();
            let r2 = hash01(index as f32 * 19.37 + 4.1);
            let point = a * (1.0 - r1) + b * (r1 * (1.0 - r2)) + c * (r1 * r2);
            let z = (hash01(index as f32 * 47.11 + 8.3) - 0.5) * depth;
            point.extend(z)
        })
        .collect()
}

fn append_disc(
    vertices: &mut Vec<MeshVertex>,
    indices: &mut Vec<u32>,
    center: Vec3,
    radius: f32,
    color: Vec4,
    segments: usize,
) {
    let base = vertices.len() as u32;
    vertices.push(MeshVertex {
        position: center.into(),
        color: color.into(),
    });
    for index in 0..segments {
        let angle = index as f32 / segments as f32 * std::f32::consts::TAU;
        let offset = vec3(angle.cos() * radius, angle.sin() * radius, 0.0);
        vertices.push(MeshVertex {
            position: (center + offset).into(),
            color: color.into(),
        });
    }
    for index in 0..segments {
        indices.extend([
            base,
            base + 1 + index as u32,
            base + 1 + ((index + 1) % segments) as u32,
        ]);
    }
}

fn default_particle_palette() -> Vec<Vec4> {
    vec![
        Vec4::new(0.18, 0.82, 0.78, 1.0),
        Vec4::new(0.32, 0.58, 1.0, 1.0),
        Vec4::new(0.92, 0.38, 0.68, 1.0),
        Vec4::new(1.0, 0.72, 0.22, 1.0),
        Vec4::new(0.48, 0.86, 0.38, 1.0),
        Vec4::new(1.0, 0.4, 0.3, 1.0),
    ]
}

fn particle_color(index: usize, alpha: f32, palette: &[Vec4]) -> Vec4 {
    let selected = palette[index % palette.len()];
    Vec4::new(selected.x, selected.y, selected.z, selected.w * alpha)
}

fn smoothstep(edge0: f32, edge1: f32, value: f32) -> f32 {
    let t = ((value - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn shade(color: Vec4, amount: f32) -> Vec4 {
    Vec4::new(
        color.x * amount,
        color.y * amount,
        color.z * amount,
        color.w,
    )
}

fn hash01(value: f32) -> f32 {
    (value.sin() * 43_758.547).fract().abs()
}

#[cfg(test)]
mod tests {
    use super::{Letter3D, Letter3DError, LetterParticles3D};
    use crate::engine::scene::Scene;
    use crate::engine::timeline::Timeline;
    use crate::frontend::animation::Ease;
    use glam::Vec3;

    #[test]
    fn supports_all_ascii_capitals() {
        for character in 'A'..='Z' {
            let letter = Letter3D::new(character, 2.0, 0.4).unwrap();
            assert!(letter.width() > 0.0, "{character} should have width");
        }
    }

    #[test]
    fn preserves_holes_during_front_face_tessellation() {
        for character in ['A', 'P', 'Q', 'R'] {
            let letter = Letter3D::new(character, 2.0, 0.4).unwrap();
            assert!(!letter.geometry.front_indices.is_empty());
        }
    }

    #[test]
    fn rejects_non_capital_characters() {
        assert!(matches!(
            Letter3D::new('a', 2.0, 0.4),
            Err(Letter3DError::UnsupportedCharacter('a'))
        ));
    }

    #[test]
    fn particle_cloud_samples_the_glyph_volume() {
        let particles = LetterParticles3D::new('K', 2.0, 0.4, 128).unwrap();
        assert_eq!(particles.particle_count(), 128);
    }

    #[test]
    fn particle_scatter_animation_is_seekable() {
        let mut scene = Scene::new();
        let id = scene.add_tattva(
            LetterParticles3D::new('K', 2.0, 0.4, 32).unwrap(),
            Vec3::ZERO,
        );
        let mut timeline = Timeline::new();
        timeline
            .animate(id)
            .at(1.0)
            .for_duration(2.0)
            .ease(Ease::Linear)
            .scatter_letter_particles()
            .spawn();
        scene.play(timeline).unwrap();

        scene.seek_to(2.0).unwrap();
        let particles = scene.get_tattva_typed::<LetterParticles3D>(id).unwrap();
        assert!((particles.state.scatter - 0.5).abs() < 1e-6);

        scene.seek_to(0.0).unwrap();
        let particles = scene.get_tattva_typed::<LetterParticles3D>(id).unwrap();
        assert!(particles.state.scatter.abs() < 1e-6);
    }
}
