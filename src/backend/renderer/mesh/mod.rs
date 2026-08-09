// src/renderer/mesh/mod.rs

pub mod latex_quad;
pub mod typst_quad;

use glam::Vec3;
use std::sync::Arc;
use wgpu::util::DeviceExt;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MeshPipelineKind {
    Mesh,
    Textured,
    Text,
}

/// GPU-side mesh buffers.
pub struct MeshInstance {
    pub vertex_buffer: Arc<wgpu::Buffer>, // Wrap in Arc
    pub index_buffer: Arc<wgpu::Buffer>,  // Wrap in Arc
    pub index_count: u32,
    pub bind_group: Option<Arc<wgpu::BindGroup>>, // Use Arc for sharing
    pub pipeline_kind: MeshPipelineKind,
    pub local_center: Vec3,
    pub has_transparency: bool,
}

// Manually implement Clone so Arc::make_mut works
impl Clone for MeshInstance {
    fn clone(&self) -> Self {
        Self {
            vertex_buffer: self.vertex_buffer.clone(),
            index_buffer: self.index_buffer.clone(),
            index_count: self.index_count,
            bind_group: self.bind_group.clone(),
            pipeline_kind: self.pipeline_kind,
            local_center: self.local_center,
            has_transparency: self.has_transparency,
        }
    }
}

impl MeshInstance {
    pub fn new(
        device: &wgpu::Device,
        vertices: &[u8],
        indices: &[u8],
        index_count: u32,
        bind_group: Option<Arc<wgpu::BindGroup>>,
        pipeline_kind: MeshPipelineKind,
        local_center: Vec3,
        has_transparency: bool,
    ) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mesh-vertex-buffer"),
            contents: vertices,
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mesh-index-buffer"),
            contents: indices,
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buffer: Arc::new(vertex_buffer),
            index_buffer: Arc::new(index_buffer),
            index_count,
            bind_group,
            pipeline_kind,
            local_center,
            has_transparency,
        }
    }
}

pub trait Drawable {
    fn draw<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>);
}

impl Drawable for MeshInstance {
    fn draw<'a>(&'a self, rpass: &mut wgpu::RenderPass<'a>) {
        if !has_drawable_geometry(
            self.index_count,
            self.vertex_buffer.size(),
            self.index_buffer.size(),
        ) {
            return;
        }
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        rpass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}

fn has_drawable_geometry(
    index_count: u32,
    vertex_buffer_size: u64,
    index_buffer_size: u64,
) -> bool {
    index_count > 0 && vertex_buffer_size > 0 && index_buffer_size > 0
}

#[cfg(test)]
mod tests {
    use super::has_drawable_geometry;

    #[test]
    fn empty_mesh_buffers_are_not_submitted_for_drawing() {
        assert!(!has_drawable_geometry(0, 0, 0));
        assert!(!has_drawable_geometry(3, 0, 12));
        assert!(!has_drawable_geometry(3, 48, 0));
        assert!(has_drawable_geometry(3, 48, 12));
    }
}
