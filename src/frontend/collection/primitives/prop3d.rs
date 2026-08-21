use std::path::{Path, PathBuf};
use std::sync::Arc;

use glam::{Mat4, Vec3, Vec4, vec2};
use thiserror::Error;

use crate::backend::renderer::vertex::{mesh::MeshVertex, text::TextVertex};
use crate::frontend::layout::{Bounded, Bounds};
use crate::projection::{Mesh, Project, ProjectionCtx, RenderPrimitive};
use crate::resource::texture::TextureImage;

const ASSET_HELP: &str = "See https://muraliengine.com/docs/3d-prop-assets for reliable free 3D prop sources and asset troubleshooting.";

/// A static 3D prop loaded from a local `.glb` or `.gltf` file.
///
/// `Prop3D` is intended for video props such as apples, balls, books, animals,
/// and simple objects used in explainer scenes. It loads geometry, base-color
/// factors, and common base-color textures. Prefer `.glb` for portable single-file
/// assets. Loose `.gltf` files are supported when their `.bin` and texture files
/// remain beside the `.gltf` file. `Prop3D` does not currently load skeletal
/// animation, physics metadata, advanced PBR lighting, or engine-specific
/// material features.
#[derive(Debug, Clone)]
pub struct Prop3D {
    meshes: Vec<Arc<Mesh>>,
    bounds_min: Vec3,
    bounds_max: Vec3,
    source_path: Option<PathBuf>,
}

impl Prop3D {
    /// Loads a static 3D prop from a local `.glb` file.
    pub fn from_glb(path: impl AsRef<Path>) -> Result<Self, Prop3DError> {
        Self::load(path.as_ref(), &[Prop3DFormat::Glb])
    }

    /// Loads a static 3D prop from a local `.gltf` file.
    ///
    /// Keep the referenced `.bin` and texture files beside the `.gltf` file.
    /// Use `.glb` when you want the asset to travel as a single file.
    pub fn from_gltf(path: impl AsRef<Path>) -> Result<Self, Prop3DError> {
        Self::load(path.as_ref(), &[Prop3DFormat::Gltf])
    }

    /// Loads a static 3D prop from either a local `.glb` or `.gltf` file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Prop3DError> {
        Self::load(path.as_ref(), &[Prop3DFormat::Glb, Prop3DFormat::Gltf])
    }

    fn load(path: &Path, allowed_formats: &[Prop3DFormat]) -> Result<Self, Prop3DError> {
        if !path.exists() {
            return Err(Prop3DError::MissingFile {
                path: path.to_path_buf(),
            });
        }

        let format =
            Prop3DFormat::from_path(path).ok_or_else(|| Prop3DError::UnsupportedExtension {
                path: path.to_path_buf(),
                expected: expected_formats(allowed_formats),
            })?;
        if !allowed_formats.contains(&format) {
            return Err(Prop3DError::UnsupportedExtension {
                path: path.to_path_buf(),
                expected: expected_formats(allowed_formats),
            });
        }

        let (document, buffers, images) = gltf::import(path)?;
        let mut builder = Prop3DBuilder::default();
        let scene = document
            .default_scene()
            .or_else(|| document.scenes().next());
        let Some(scene) = scene else {
            return Err(Prop3DError::EmptyAsset {
                path: path.to_path_buf(),
            });
        };

        for node in scene.nodes() {
            builder.visit_node(&node, Mat4::IDENTITY, &buffers, &images)?;
        }

        builder.finish(Some(path.to_path_buf()))
    }

    /// Returns the source file path if this prop was loaded from disk.
    pub fn source_path(&self) -> Option<&Path> {
        self.source_path.as_deref()
    }

    /// Number of render meshes emitted by this prop.
    pub fn mesh_count(&self) -> usize {
        self.meshes.len()
    }

    /// Minimum corner of the loaded asset's local-space 3D bounds.
    pub fn bounds_min(&self) -> Vec3 {
        self.bounds_min
    }

    /// Maximum corner of the loaded asset's local-space 3D bounds.
    pub fn bounds_max(&self) -> Vec3 {
        self.bounds_max
    }

    /// Center of the loaded asset's local-space 3D bounds.
    pub fn center(&self) -> Vec3 {
        (self.bounds_min + self.bounds_max) * 0.5
    }

    /// Width, height, and depth of the loaded asset in local units.
    pub fn dimensions(&self) -> Vec3 {
        self.bounds_max - self.bounds_min
    }
}

impl Project for Prop3D {
    fn project(&self, ctx: &mut ProjectionCtx) {
        for mesh in &self.meshes {
            ctx.emit(RenderPrimitive::Mesh(mesh.clone()));
        }
    }
}

impl Bounded for Prop3D {
    fn local_bounds(&self) -> Bounds {
        if self.meshes.is_empty() {
            return Bounds::default();
        }

        let min = self.bounds_min.truncate();
        let max = self.bounds_max.truncate();
        let z_pad = self.bounds_min.z.abs().max(self.bounds_max.z.abs()) * 0.12;
        Bounds::new(min - vec2(z_pad, z_pad), max + vec2(z_pad, z_pad))
    }
}

#[derive(Debug, Error)]
pub enum Prop3DError {
    #[error("Prop3D could not find 3D prop asset at {path}. {ASSET_HELP}")]
    MissingFile { path: PathBuf },
    #[error("Prop3D expected {expected}, got {path}. {ASSET_HELP}")]
    UnsupportedExtension { path: PathBuf, expected: String },
    #[error("Prop3D found no renderable scene in {path}. {ASSET_HELP}")]
    EmptyAsset { path: PathBuf },
    #[error("Prop3D found a primitive without positions in {path}.")]
    MissingPositions { path: PathBuf },
    #[error("Prop3D could not load 3D prop asset: {0}. {ASSET_HELP}")]
    Gltf(#[from] gltf::Error),
    #[error("Prop3D found an embedded texture format that is not supported yet: {format:?}.")]
    UnsupportedTextureFormat { format: gltf::image::Format },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Prop3DFormat {
    Glb,
    Gltf,
}

impl Prop3DFormat {
    fn from_path(path: &Path) -> Option<Self> {
        match path.extension().and_then(|extension| extension.to_str()) {
            Some(extension) if extension.eq_ignore_ascii_case("glb") => Some(Self::Glb),
            Some(extension) if extension.eq_ignore_ascii_case("gltf") => Some(Self::Gltf),
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Glb => ".glb",
            Self::Gltf => ".gltf",
        }
    }
}

fn expected_formats(formats: &[Prop3DFormat]) -> String {
    let labels: Vec<&str> = formats.iter().map(|format| format.label()).collect();
    match labels.as_slice() {
        [one] => format!("a local {one} file"),
        [first, second] => format!("a local {first} or {second} file"),
        _ => "a supported local 3D prop file".to_string(),
    }
}

#[derive(Default)]
struct Prop3DBuilder {
    meshes: Vec<Arc<Mesh>>,
    bounds_min: Option<Vec3>,
    bounds_max: Option<Vec3>,
}

impl Prop3DBuilder {
    fn visit_node(
        &mut self,
        node: &gltf::Node<'_>,
        parent_transform: Mat4,
        buffers: &[gltf::buffer::Data],
        images: &[gltf::image::Data],
    ) -> Result<(), Prop3DError> {
        let transform = parent_transform * node_transform(node);

        if let Some(mesh) = node.mesh() {
            for primitive in mesh.primitives() {
                self.push_primitive(&primitive, transform, buffers, images)?;
            }
        }

        for child in node.children() {
            self.visit_node(&child, transform, buffers, images)?;
        }

        Ok(())
    }

    fn push_primitive(
        &mut self,
        primitive: &gltf::Primitive<'_>,
        transform: Mat4,
        buffers: &[gltf::buffer::Data],
        images: &[gltf::image::Data],
    ) -> Result<(), Prop3DError> {
        let reader =
            primitive.reader(|buffer| buffers.get(buffer.index()).map(|data| data.0.as_slice()));
        let positions: Vec<Vec3> = reader
            .read_positions()
            .ok_or_else(|| Prop3DError::MissingPositions {
                path: PathBuf::from("<embedded glb primitive>"),
            })?
            .map(|position| transform.transform_point3(Vec3::from(position)))
            .collect();

        for position in &positions {
            self.include_position(*position);
        }

        let indices: Vec<u32> = reader
            .read_indices()
            .map(|indices| indices.into_u32().collect())
            .unwrap_or_else(|| (0..positions.len() as u32).collect());

        let material = primitive.material();
        let pbr = material.pbr_metallic_roughness();
        let base_color = Vec4::from_array(pbr.base_color_factor());

        if let Some(texture_info) = pbr.base_color_texture() {
            let tex_coord = texture_info.tex_coord();
            if let (Some(tex_coords), Some(texture)) = (
                reader
                    .read_tex_coords(tex_coord)
                    .map(|coords| coords.into_f32()),
                image_texture(texture_info.texture().source().index(), images)?,
            ) {
                let vertices = positions
                    .iter()
                    .zip(tex_coords)
                    .map(|(position, uv)| TextVertex {
                        position: position.to_array(),
                        uv,
                        color: base_color.to_array(),
                    })
                    .collect();
                self.meshes
                    .push(Mesh::from_textured_vertices(vertices, indices, texture));
                return Ok(());
            }
        }

        let vertices = positions
            .iter()
            .map(|position| MeshVertex {
                position: position.to_array(),
                color: base_color.to_array(),
            })
            .collect();
        self.meshes.push(Mesh::from_tessellation(vertices, indices));
        Ok(())
    }

    fn include_position(&mut self, position: Vec3) {
        self.bounds_min = Some(self.bounds_min.map_or(position, |min| min.min(position)));
        self.bounds_max = Some(self.bounds_max.map_or(position, |max| max.max(position)));
    }

    fn finish(self, source_path: Option<PathBuf>) -> Result<Prop3D, Prop3DError> {
        if self.meshes.is_empty() {
            return Err(Prop3DError::EmptyAsset {
                path: source_path
                    .clone()
                    .unwrap_or_else(|| PathBuf::from("<glb>")),
            });
        }

        Ok(Prop3D {
            meshes: self.meshes,
            bounds_min: self.bounds_min.unwrap_or(Vec3::ZERO),
            bounds_max: self.bounds_max.unwrap_or(Vec3::ZERO),
            source_path,
        })
    }
}

fn node_transform(node: &gltf::Node<'_>) -> Mat4 {
    Mat4::from_cols_array_2d(&node.transform().matrix())
}

fn image_texture(
    image_index: usize,
    images: &[gltf::image::Data],
) -> Result<Option<Arc<TextureImage>>, Prop3DError> {
    let Some(image) = images.get(image_index) else {
        return Ok(None);
    };
    let rgba = image_to_rgba8(image)?;
    Ok(Some(Arc::new(TextureImage::from_rgba(
        rgba,
        image.width,
        image.height,
    ))))
}

fn image_to_rgba8(image: &gltf::image::Data) -> Result<Vec<u8>, Prop3DError> {
    use gltf::image::Format;

    let rgba = match image.format {
        Format::R8 => image
            .pixels
            .iter()
            .flat_map(|r| [*r, *r, *r, u8::MAX])
            .collect(),
        Format::R8G8 => image
            .pixels
            .chunks_exact(2)
            .flat_map(|pixel| [pixel[0], pixel[1], 0, u8::MAX])
            .collect(),
        Format::R8G8B8 => image
            .pixels
            .chunks_exact(3)
            .flat_map(|pixel| [pixel[0], pixel[1], pixel[2], u8::MAX])
            .collect(),
        Format::R8G8B8A8 => image.pixels.clone(),
        format => return Err(Prop3DError::UnsupportedTextureFormat { format }),
    };
    Ok(rgba)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_error_points_to_asset_help() {
        let error = Prop3D::from_glb("assets/props/does-not-exist.glb").unwrap_err();
        assert!(error.to_string().contains("/docs/3d-prop-assets"));
    }

    #[test]
    fn from_glb_rejects_gltf_extension() {
        let error = Prop3D::from_glb("assets/props/demo-apple/demo-apple.gltf").unwrap_err();

        assert!(error.to_string().contains("a local .glb file"));
    }

    #[test]
    fn local_bounds_include_depth_padding() {
        let prop = Prop3D {
            meshes: vec![Mesh::empty().into()],
            bounds_min: Vec3::new(-1.0, -0.5, -2.0),
            bounds_max: Vec3::new(1.0, 0.5, 2.0),
            source_path: None,
        };

        let bounds = prop.local_bounds();
        assert!(bounds.min.x < -1.0);
        assert!(bounds.max.x > 1.0);
    }

    #[test]
    fn loads_demo_glb_prop() {
        let prop = Prop3D::from_glb("assets/props/demo-pyramid.glb").unwrap();

        assert_eq!(prop.mesh_count(), 5);
        assert_eq!(
            prop.source_path().and_then(|path| path.file_name()),
            Some(std::ffi::OsStr::new("demo-pyramid.glb"))
        );
    }

    #[test]
    fn loads_demo_apple_gltf_prop() {
        let prop = Prop3D::from_gltf("assets/props/demo-apple/demo-apple.gltf").unwrap();

        assert_eq!(prop.mesh_count(), 3);
        assert_eq!(
            prop.source_path().and_then(|path| path.file_name()),
            Some(std::ffi::OsStr::new("demo-apple.gltf"))
        );
    }

    #[test]
    fn from_file_loads_glb_and_gltf() {
        let glb = Prop3D::from_file("assets/props/demo-pyramid.glb").unwrap();
        let gltf = Prop3D::from_file("assets/props/demo-apple/demo-apple.gltf").unwrap();

        assert_eq!(glb.mesh_count(), 5);
        assert_eq!(gltf.mesh_count(), 3);
    }

    #[test]
    fn exposes_local_3d_bounds_for_framing() {
        let prop = Prop3D {
            meshes: vec![Mesh::empty().into()],
            bounds_min: Vec3::new(-2.0, 1.0, -4.0),
            bounds_max: Vec3::new(4.0, 5.0, 2.0),
            source_path: None,
        };

        assert_eq!(prop.bounds_min(), Vec3::new(-2.0, 1.0, -4.0));
        assert_eq!(prop.bounds_max(), Vec3::new(4.0, 5.0, 2.0));
        assert_eq!(prop.center(), Vec3::new(1.0, 3.0, -1.0));
        assert_eq!(prop.dimensions(), Vec3::new(6.0, 4.0, 6.0));
    }
}
