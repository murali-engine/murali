use crate::backend::ecs::components::*;
use crate::backend::renderer::Renderer;
use crate::backend::renderer::mesh::latex_quad::build_textured_quad;
use crate::backend::renderer::mesh::{MeshInstance, MeshPipelineKind};
use crate::frontend::DirtyFlags;
use crate::frontend::TattvaId;
use crate::frontend::tattva_trait::TattvaTrait;
use crate::projection::MeshData;
use crate::projection::{ProjectionCtx, RenderPrimitive};
use crate::resource::latex_resource::backend::compile_latex;
use crate::resource::latex_resource::raster::{normalized_world_height, rasterize_svg};
use crate::resource::text::layout::layout_label;
use crate::resource::text::manager::font_asset;
use crate::resource::text::mesh::build_label_mesh;
use crate::resource::typst_resource::cache::{TypstRaster, TypstRasterCache};
use crate::resource::typst_resource::compiler::TypstBackend;
use crate::resource::typst_resource::raster::{
    normalized_world_height_from_metrics as normalized_typst_world_height, rasterize_svg_to_rgba,
};
use crate::validation::ValidationError;
use hecs::{Entity, World};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;

/// Manages the mapping between Frontend Tattvas and their materialized ECS entities.
pub struct SyncBoundary {
    /// Maps TattvaId to the list of entities representing its current geometry.
    pub entity_cache: HashMap<TattvaId, Vec<Entity>>,
    text_bind_groups: HashMap<String, Arc<wgpu::BindGroup>>,
    latex_cache_dir: PathBuf,
    typst_backend: Option<TypstBackend>,
    typst_cache: TypstRasterCache,
    reported_runtime_errors: HashSet<String>,
}

impl SyncBoundary {
    const REBUILD_FLAGS: DirtyFlags = DirtyFlags::REBUILD;

    pub fn new() -> Self {
        Self {
            entity_cache: HashMap::new(),
            text_bind_groups: HashMap::new(),
            latex_cache_dir: std::env::temp_dir().join("murali_latex_cache"),
            typst_backend: None,
            typst_cache: TypstRasterCache::new(128),
            reported_runtime_errors: HashSet::new(),
        }
    }

    /// The core sync loop. Called once per frame by the Engine.
    ///
    /// NOTE:
    /// - Projection stays CPU-only.
    /// - GPU upload happens *here*.
    pub fn sync_tattva(
        &mut self,
        world: &mut World,
        device: &wgpu::Device,
        renderer: &Renderer,
        tattva: &mut dyn TattvaTrait,
    ) -> Result<(), ValidationError> {
        let dirty = tattva.dirty_flags();
        if dirty.is_empty() {
            return Ok(());
        }

        if dirty.intersects(Self::REBUILD_FLAGS) {
            self.rebuild_render_entities(world, device, renderer, tattva)?;
            tattva.clear_all_dirty();
            return Ok(());
        }

        self.sync_runtime_only(tattva);
        Ok(())
    }

    pub fn remove_tattva(&mut self, world: &mut World, tattva_id: TattvaId) {
        self.despawn_cached_entities(world, tattva_id);
    }

    fn sync_runtime_only(&mut self, tattva: &mut dyn TattvaTrait) {
        tattva.clear_dirty(DirtyFlags::TRANSFORM | DirtyFlags::STYLE | DirtyFlags::VISIBILITY);
    }

    fn rebuild_render_entities(
        &mut self,
        world: &mut World,
        device: &wgpu::Device,
        renderer: &Renderer,
        tattva: &mut dyn TattvaTrait,
    ) -> Result<(), ValidationError> {
        self.despawn_cached_entities(world, tattva.id());
        let primitives = self.project_tattva(tattva)?;
        let entities = self.materialize_primitives(world, device, renderer, tattva, primitives);
        self.entity_cache.insert(tattva.id(), entities);
        Ok(())
    }

    fn despawn_cached_entities(&mut self, world: &mut World, tattva_id: TattvaId) {
        if let Some(old_entities) = self.entity_cache.remove(&tattva_id) {
            for entity in old_entities {
                let _ = world.despawn(entity);
            }
        }
    }

    fn project_tattva(
        &self,
        tattva: &dyn TattvaTrait,
    ) -> Result<Vec<RenderPrimitive>, ValidationError> {
        let mut ctx = ProjectionCtx::new(tattva.props().clone());
        tattva.project(&mut ctx);
        if let Some(error) = ctx.diagnostics.into_iter().next() {
            return Err(error);
        }
        Ok(ctx.primitives)
    }

    fn materialize_primitives(
        &mut self,
        world: &mut World,
        device: &wgpu::Device,
        renderer: &Renderer,
        tattva: &dyn TattvaTrait,
        primitives: Vec<RenderPrimitive>,
    ) -> Vec<Entity> {
        let mut new_entities = Vec::new();

        for (primitive_index, primitive) in primitives.into_iter().enumerate() {
            let render_order = RenderOrderComponent {
                tattva_id: tattva.id(),
                primitive_index: primitive_index as u32,
            };
            let entity = match primitive {
                RenderPrimitive::Mesh(mesh) => upload_mesh(device, renderer, mesh.as_ref(), None)
                    .map(|mesh_instance| {
                        world.spawn((
                            MeshComponent(Arc::new(mesh_instance)),
                            tattva.props().clone(),
                            render_order,
                        ))
                    }),
                RenderPrimitive::Line {
                    start,
                    end,
                    thickness,
                    color,
                    dash_length,
                    gap_length,
                    dash_offset,
                } => Some(world.spawn((
                    LineComponent {
                        start,
                        end,
                        thickness,
                        dash_length,
                        gap_length,
                        dash_offset,
                    },
                    ColorComponent(color),
                    tattva.props().clone(),
                    render_order,
                ))),
                RenderPrimitive::Text {
                    content,
                    height,
                    color,
                    font_name,
                    offset,
                    rotation,
                } => self
                    .build_label_instance(
                        device,
                        renderer,
                        &content,
                        height,
                        color,
                        font_name.as_deref(),
                        offset,
                        rotation,
                    )
                    .map(|mesh_instance| {
                        world.spawn((
                            MeshComponent(Arc::new(mesh_instance)),
                            tattva.props().clone(),
                            render_order,
                        ))
                    }),
                RenderPrimitive::Latex {
                    source,
                    height,
                    color,
                    offset,
                } => self
                    .build_latex_instance(device, renderer, &source, height, color, offset)
                    .map(|mesh_instance| {
                        world.spawn((
                            MeshComponent(Arc::new(mesh_instance)),
                            tattva.props().clone(),
                            render_order,
                        ))
                    }),
                RenderPrimitive::Typst {
                    source,
                    height,
                    color,
                    offset,
                    normalize,
                    tint,
                } => self
                    .build_typst_instance(
                        device, renderer, &source, height, color, offset, normalize, tint,
                    )
                    .map(|mesh_instance| {
                        world.spawn((
                            MeshComponent(Arc::new(mesh_instance)),
                            tattva.props().clone(),
                            render_order,
                        ))
                    }),
            };

            if let Some(entity) = entity {
                new_entities.push(entity);
            }
        }

        new_entities
    }

    fn build_label_instance(
        &mut self,
        device: &wgpu::Device,
        renderer: &Renderer,
        content: &str,
        height: f32,
        color: glam::Vec4,
        font_name: Option<&str>,
        offset: glam::Vec3,
        rotation: f32,
    ) -> Option<MeshInstance> {
        let asset = match font_asset(font_name) {
            Ok(asset) => asset,
            Err(error) => {
                let key = font_name.unwrap_or("default").to_string();
                self.report_once(
                    format!("font-load::{key}::{error}"),
                    format!("Text font load failed for `{key}`: {error}"),
                );
                return None;
            }
        };
        let key = font_name.unwrap_or("default").to_string();

        let bind_group = if let Some(existing) = self.text_bind_groups.get(&key) {
            existing.clone()
        } else {
            let created = Arc::new(renderer.create_text_bind_group_from_raster(
                &asset.atlas.rgba,
                asset.atlas.width,
                asset.atlas.height,
            ));
            self.text_bind_groups.insert(key.clone(), created.clone());
            created
        };

        let layout = layout_label(&asset.font, content, height);
        let mesh = build_label_mesh(&layout, &asset.atlas, color);
        let mesh = if rotation != 0.0 {
            rotate_mesh(mesh.as_ref(), rotation)
        } else {
            mesh.as_ref().clone()
        };
        let mesh = translate_mesh(&mesh, offset);
        upload_mesh(device, renderer, &mesh, Some(bind_group))
    }

    fn build_latex_instance(
        &mut self,
        device: &wgpu::Device,
        renderer: &Renderer,
        source: &str,
        height: f32,
        color: glam::Vec4,
        offset: glam::Vec3,
    ) -> Option<MeshInstance> {
        let latex = match compile_latex(source, &self.latex_cache_dir) {
            Ok(latex) => latex,
            Err(error) => {
                self.report_once(
                    format!("latex-compile::{error}"),
                    format!("LaTeX compile failed for `{source}`: {error}"),
                );
                return None;
            }
        };

        let raster = match rasterize_svg(
            &latex.svg_path,
            renderer.device_mgr.config.read().height as f32 / 4.0,
            renderer.device_mgr.max_texture_size(),
        ) {
            Ok(raster) => raster,
            Err(error) => {
                self.report_once(
                    format!("latex-raster::{error}"),
                    format!("LaTeX rasterization failed for `{source}`: {error}"),
                );
                return None;
            }
        };

        let bind_group =
            renderer.create_text_bind_group_from_raster(&raster.rgba, raster.width, raster.height);
        let world_height = normalized_world_height(height, &raster);
        let mesh = build_textured_quad(raster.width, raster.height, world_height, color);
        let mesh = translate_mesh(mesh.as_ref(), offset);
        upload_mesh(device, renderer, &mesh, Some(Arc::new(bind_group)))
    }

    fn build_typst_instance(
        &mut self,
        device: &wgpu::Device,
        renderer: &Renderer,
        source: &str,
        height: f32,
        color: glam::Vec4,
        offset: glam::Vec3,
        normalize: bool,
        tint: bool,
    ) -> Option<MeshInstance> {
        let (target_width, target_height) = {
            let config = renderer.device_mgr.config.read();
            (config.width, config.height)
        };
        let scale = typst_raster_scale(target_height, height);
        let cache_key = typst_raster_cache_key(
            source,
            height,
            tint,
            target_width,
            target_height,
            scale,
            renderer.device_mgr.max_texture_size(),
        );
        let raster = if let Some(existing) = self.typst_cache.get(&cache_key) {
            existing
        } else {
            if self.typst_backend.is_none() {
                self.typst_backend = TypstBackend::new().ok();
            }
            let backend = match self.typst_backend.as_ref() {
                Some(backend) => backend,
                None => {
                    self.report_once(
                        "typst-backend-init".to_string(),
                        "Typst backend initialization failed".to_string(),
                    );
                    return None;
                }
            };

            let svg = match backend.render_to_svg(source, height * 36.0) {
                Ok(svg) => svg,
                Err(error) => {
                    self.report_once(
                        format!("typst-compile::{error}"),
                        format!("Typst compilation failed for `{source}`: {error}"),
                    );
                    return None;
                }
            };

            let rasterized = match rasterize_svg_to_rgba(
                &svg,
                scale,
                renderer.device_mgr.max_texture_size(),
                tint,
            ) {
                Ok(result) => result,
                Err(error) => {
                    self.report_once(
                        format!("typst-raster::{error}"),
                        format!("Typst rasterization failed for `{source}`: {error}"),
                    );
                    return None;
                }
            };

            self.typst_cache.insert(
                cache_key.clone(),
                TypstRaster {
                    rgba: rasterized.rgba,
                    width: rasterized.width,
                    height: rasterized.height,
                    normalized_height_px: rasterized.normalized_height_px,
                    natural_height_pts: rasterized.natural_height_pts,
                    svg: Some(svg),
                },
            );
            self.typst_cache.get(&cache_key)?
        };

        let bind_group =
            renderer.create_text_bind_group_from_raster(&raster.rgba, raster.width, raster.height);

        let world_height = if normalize {
            normalized_typst_world_height(height, raster.height, raster.normalized_height_px)
        } else {
            // For block-like Typst content such as CodeBlock, the caller's
            // requested height should be treated as authoritative. Using the
            // raster's natural page height here causes the rendered quad to
            // drift away from the frontend's measured layout box.
            height
        };

        let mesh = build_textured_quad(raster.width, raster.height, world_height, color);
        let mesh = translate_mesh(mesh.as_ref(), offset);
        upload_mesh(device, renderer, &mesh, Some(Arc::new(bind_group)))
    }

    fn report_once(&mut self, key: String, message: String) {
        if self.reported_runtime_errors.insert(key) {
            eprintln!("{message}");
        }
    }
}

fn typst_raster_scale(target_height: u32, world_height: f32) -> f32 {
    (target_height as f32 / 4.0 / world_height.max(0.1)).clamp(1.0, 8.0)
}

fn typst_raster_cache_key(
    source: &str,
    world_height: f32,
    tint: bool,
    target_width: u32,
    target_height: u32,
    scale: f32,
    max_texture_size: u32,
) -> String {
    format!(
        "target={target_width}x{target_height}::scale={scale:.4}::max_texture={max_texture_size}::height={world_height:.4}::tint={tint}::{source}"
    )
}

fn upload_mesh(
    device: &wgpu::Device,
    renderer: &Renderer,
    mesh: &crate::projection::Mesh,
    bind_group: Option<Arc<wgpu::BindGroup>>,
) -> Option<MeshInstance> {
    match &mesh.data {
        MeshData::Empty => None,
        MeshData::Mesh(vertices) => {
            let vertex_bytes = bytemuck::cast_slice(vertices);
            let index_bytes = bytemuck::cast_slice(&mesh.indices);
            Some(MeshInstance::new(
                device,
                vertex_bytes,
                index_bytes,
                mesh.indices.len() as u32,
                bind_group,
                MeshPipelineKind::Mesh,
                center_of_positions(vertices.iter().map(|vertex| vertex.position)),
                vertices.iter().any(|vertex| vertex.color[3] < 1.0),
            ))
        }
        MeshData::Textured(vertices) => {
            let texture = mesh.texture.as_ref()?;
            let bind_group = if let Some(existing) = bind_group {
                existing
            } else {
                Arc::new(renderer.create_text_bind_group_from_raster(
                    &texture.rgba,
                    texture.width,
                    texture.height,
                ))
            };
            let vertex_bytes = bytemuck::cast_slice(vertices);
            let index_bytes = bytemuck::cast_slice(&mesh.indices);
            Some(MeshInstance::new(
                device,
                vertex_bytes,
                index_bytes,
                mesh.indices.len() as u32,
                Some(bind_group),
                MeshPipelineKind::Textured,
                center_of_positions(vertices.iter().map(|vertex| vertex.position)),
                vertices.iter().any(|vertex| vertex.color[3] < 1.0)
                    || texture.rgba.chunks_exact(4).any(|pixel| pixel[3] < u8::MAX),
            ))
        }
        MeshData::Text(vertices) => {
            let vertex_bytes = bytemuck::cast_slice(vertices);
            let index_bytes = bytemuck::cast_slice(&mesh.indices);
            Some(MeshInstance::new(
                device,
                vertex_bytes,
                index_bytes,
                mesh.indices.len() as u32,
                bind_group,
                MeshPipelineKind::Text,
                center_of_positions(vertices.iter().map(|vertex| vertex.position)),
                true,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::Tattva;
    use crate::frontend::sangrah::ganit::calculus::stream_lines::StreamLines;
    use glam::Vec2;

    #[test]
    fn projection_diagnostics_cross_the_sync_boundary_as_errors() {
        let tattva = Tattva::new(7, StreamLines::new(Vec::new(), |_| Vec2::X));
        let error = match SyncBoundary::new().project_tattva(&tattva) {
            Err(error) => error,
            Ok(_) => panic!("invalid projection unexpectedly succeeded"),
        };

        assert!(matches!(
            error,
            ValidationError::Empty {
                component: "StreamLines",
                field: "start_points"
            }
        ));
    }

    #[test]
    fn typst_raster_cache_key_changes_with_render_target_resolution() {
        let first_scale = typst_raster_scale(720, 0.4);
        let resized_scale = typst_raster_scale(1080, 0.4);
        let first = typst_raster_cache_key("$x$", 0.4, true, 1280, 720, first_scale, 8192);
        let resized = typst_raster_cache_key("$x$", 0.4, true, 1920, 1080, resized_scale, 8192);

        assert_ne!(first, resized);
    }

    #[test]
    fn typst_raster_cache_key_is_stable_for_an_unchanged_target() {
        let scale = typst_raster_scale(900, 0.6);
        let first = typst_raster_cache_key("hello", 0.6, false, 1440, 900, scale, 8192);
        let second = typst_raster_cache_key("hello", 0.6, false, 1440, 900, scale, 8192);

        assert_eq!(first, second);
    }
}

fn center_of_positions(positions: impl IntoIterator<Item = [f32; 3]>) -> glam::Vec3 {
    let mut positions = positions.into_iter();
    let Some(first) = positions.next() else {
        return glam::Vec3::ZERO;
    };
    let mut min = glam::Vec3::from(first);
    let mut max = min;
    for position in positions {
        let position = glam::Vec3::from(position);
        min = min.min(position);
        max = max.max(position);
    }
    (min + max) * 0.5
}

fn translate_mesh(mesh: &crate::projection::Mesh, offset: glam::Vec3) -> crate::projection::Mesh {
    match &mesh.data {
        MeshData::Empty => mesh.clone(),
        MeshData::Mesh(vertices) => {
            let translated = vertices
                .iter()
                .map(|v| {
                    let mut v = *v;
                    v.position[0] += offset.x;
                    v.position[1] += offset.y;
                    v.position[2] += offset.z;
                    v
                })
                .collect();
            crate::projection::Mesh {
                data: MeshData::Mesh(translated),
                indices: mesh.indices.clone(),
                texture: mesh.texture.clone(),
            }
        }
        MeshData::Textured(vertices) => {
            let translated = vertices
                .iter()
                .map(|v| {
                    let mut v = *v;
                    v.position[0] += offset.x;
                    v.position[1] += offset.y;
                    v.position[2] += offset.z;
                    v
                })
                .collect();
            crate::projection::Mesh {
                data: MeshData::Textured(translated),
                indices: mesh.indices.clone(),
                texture: mesh.texture.clone(),
            }
        }
        MeshData::Text(vertices) => {
            let translated = vertices
                .iter()
                .map(|v| {
                    let mut v = *v;
                    v.position[0] += offset.x;
                    v.position[1] += offset.y;
                    v.position[2] += offset.z;
                    v
                })
                .collect();
            crate::projection::Mesh {
                data: MeshData::Text(translated),
                indices: mesh.indices.clone(),
                texture: mesh.texture.clone(),
            }
        }
    }
}
fn rotate_mesh(mesh: &crate::projection::Mesh, angle: f32) -> crate::projection::Mesh {
    let cos = angle.cos();
    let sin = angle.sin();
    match &mesh.data {
        MeshData::Empty => mesh.clone(),
        MeshData::Mesh(vertices) => {
            let rotated = vertices
                .iter()
                .map(|v| {
                    let mut v = *v;
                    let x = v.position[0];
                    let y = v.position[1];
                    v.position[0] = x * cos - y * sin;
                    v.position[1] = x * sin + y * cos;
                    v
                })
                .collect();
            crate::projection::Mesh {
                data: MeshData::Mesh(rotated),
                indices: mesh.indices.clone(),
                texture: mesh.texture.clone(),
            }
        }
        MeshData::Textured(vertices) => {
            let rotated = vertices
                .iter()
                .map(|v| {
                    let mut v = *v;
                    let x = v.position[0];
                    let y = v.position[1];
                    v.position[0] = x * cos - y * sin;
                    v.position[1] = x * sin + y * cos;
                    v
                })
                .collect();
            crate::projection::Mesh {
                data: MeshData::Textured(rotated),
                indices: mesh.indices.clone(),
                texture: mesh.texture.clone(),
            }
        }
        MeshData::Text(vertices) => {
            let rotated = vertices
                .iter()
                .map(|v| {
                    let mut v = *v;
                    let x = v.position[0];
                    let y = v.position[1];
                    v.position[0] = x * cos - y * sin;
                    v.position[1] = x * sin + y * cos;
                    v
                })
                .collect();
            crate::projection::Mesh {
                data: MeshData::Text(rotated),
                indices: mesh.indices.clone(),
                texture: mesh.texture.clone(),
            }
        }
    }
}
