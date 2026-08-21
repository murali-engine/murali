use anyhow::Result;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2, Vec3, Vec4};
use image::{ImageBuffer, RgbaImage};
use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::num::NonZeroU64;
use std::sync::Arc;
use std::time::{Duration, Instant};
use wgpu::util::DeviceExt;

use crate::backend::ecs::components::*;
use crate::backend::renderer::device::{DepthTexture, DeviceManager};
use crate::backend::renderer::mesh::{Drawable, MeshInstance, MeshPipelineKind};
use crate::backend::renderer::vertex::mesh::MeshVertex;
use crate::backend::renderer::vertex::text::TextVertex;
use crate::engine::camera::Projection;
use crate::engine::scene::Scene;
use crate::frontend::props::SharedProps;
use crate::frontend::props::{DepthMode, DrawableProps};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Uniforms {
    pub mvp: [[f32; 4]; 4],
    pub alpha: f32,
    pub _padding: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct SceneViewUniforms {
    mvp: [[f32; 4]; 4],
    size: [f32; 2],
    opacity: f32,
    corner_radius: f32,
    background: [f32; 4],
    border_color: [f32; 4],
    border_width: f32,
    _padding: [f32; 3],
}

pub struct SceneRenderTarget {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub depth: DepthTexture,
    pub texture_bind_group: Arc<wgpu::BindGroup>,
    pub width: u32,
    pub height: u32,
}

pub struct SceneViewPass<'a> {
    pub scene: &'a Scene,
    pub world: &'a hecs::World,
    pub target: &'a SceneRenderTarget,
}

pub struct SceneViewComposite {
    pub tattva_id: usize,
    pub props: SharedProps,
    pub texture_bind_group: Arc<wgpu::BindGroup>,
    pub size: Vec2,
    pub background: Option<Vec4>,
    pub corner_radius: f32,
    pub border_width: f32,
    pub border_color: Vec4,
}

impl Uniforms {
    pub fn from_mat4_alpha(mat: Mat4, alpha: f32) -> Self {
        Self {
            mvp: mat.to_cols_array_2d(),
            alpha,
            _padding: [0.0; 3],
        }
    }
}

const INITIAL_UNIFORM_CAPACITY: u64 = 1024;

fn next_uniform_capacity(current: u64, required: u64, maximum: u64) -> Option<u64> {
    if required > maximum {
        return None;
    }
    let mut capacity = current.max(1).min(maximum);
    while capacity < required {
        let grown = capacity.saturating_mul(2).min(maximum);
        if grown == capacity {
            return None;
        }
        capacity = grown;
    }
    Some(capacity)
}

fn maximum_uniform_capacity(max_buffer_size: u64, slot_size: u64) -> u64 {
    let buffer_capacity = max_buffer_size / slot_size;
    let dynamic_offset_capacity = (u32::MAX as u64 / slot_size).saturating_add(1);
    buffer_capacity.min(dynamic_offset_capacity)
}

fn create_uniform_resources(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    slot_size: u64,
    capacity: u64,
) -> (wgpu::Buffer, wgpu::BindGroup) {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Uniform Buffer"),
        size: slot_size * capacity,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("uniform-bind-group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: Some(NonZeroU64::new(slot_size).unwrap()),
                }),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    });
    (buffer, bind_group)
}

fn create_scene_view_uniform_resources(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    slot_size: u64,
    capacity: u64,
) -> (wgpu::Buffer, wgpu::BindGroup) {
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("SceneView Uniform Buffer"),
        size: slot_size * capacity,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("scene-view-uniform-bind-group"),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer: &buffer,
                offset: 0,
                size: Some(NonZeroU64::new(slot_size).unwrap()),
            }),
        }],
    });
    (buffer, bind_group)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct RenderSortKey {
    layer: i32,
    tattva_id: usize,
    primitive_index: u32,
}

enum DrawCommand {
    Line {
        key: RenderSortKey,
        data: [f32; 16],
        world_center: Vec3,
        transparent: bool,
        depth_mode: DepthMode,
    },
    Mesh {
        key: RenderSortKey,
        mesh: Arc<MeshInstance>,
        model: Mat4,
        bind_group: Option<Arc<wgpu::BindGroup>>,
        alpha: f32,
        world_center: Vec3,
        transparent: bool,
        depth_mode: DepthMode,
    },
    SceneView {
        key: RenderSortKey,
        model: Mat4,
        bind_group: Arc<wgpu::BindGroup>,
        uniforms: SceneViewUniforms,
        world_center: Vec3,
        depth_mode: DepthMode,
    },
}

impl DrawCommand {
    fn key(&self) -> RenderSortKey {
        match self {
            Self::Line { key, .. } | Self::Mesh { key, .. } | Self::SceneView { key, .. } => *key,
        }
    }

    fn world_center(&self) -> Vec3 {
        match self {
            Self::Line { world_center, .. }
            | Self::Mesh { world_center, .. }
            | Self::SceneView { world_center, .. } => *world_center,
        }
    }

    fn is_transparent(&self) -> bool {
        match self {
            Self::Line { transparent, .. } | Self::Mesh { transparent, .. } => *transparent,
            Self::SceneView { .. } => true,
        }
    }

    fn depth_mode(&self) -> DepthMode {
        match self {
            Self::Line { depth_mode, .. }
            | Self::Mesh { depth_mode, .. }
            | Self::SceneView { depth_mode, .. } => *depth_mode,
        }
    }

    fn phase_3d(&self) -> u8 {
        match (self.depth_mode(), self.is_transparent()) {
            (DepthMode::Overlay, _) => 2,
            (DepthMode::World, false) => 0,
            (DepthMode::World, true) => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DepthPipelineMode {
    Overlay,
    OpaqueWorld,
    TransparentWorld,
}

fn depth_pipeline_mode(command: &DrawCommand, perspective: bool) -> DepthPipelineMode {
    if !perspective || command.depth_mode() == DepthMode::Overlay {
        DepthPipelineMode::Overlay
    } else if command.is_transparent() {
        DepthPipelineMode::TransparentWorld
    } else {
        DepthPipelineMode::OpaqueWorld
    }
}

fn compare_3d_commands(a: &DrawCommand, b: &DrawCommand, view: Mat4) -> Ordering {
    let phase_order = a.phase_3d().cmp(&b.phase_3d());
    if phase_order != Ordering::Equal {
        return phase_order;
    }

    if a.phase_3d() == 1 {
        let layer_order = a.key().layer.cmp(&b.key().layer);
        if layer_order != Ordering::Equal {
            return layer_order;
        }
        let a_depth = -view.transform_point3(a.world_center()).z;
        let b_depth = -view.transform_point3(b.world_center()).z;
        let depth_order = b_depth.total_cmp(&a_depth);
        if depth_order != Ordering::Equal {
            return depth_order;
        }
    }

    a.key().cmp(&b.key())
}

pub struct Renderer {
    pub device_mgr: Arc<DeviceManager>, // Changed to Arc
    pub clear_color: wgpu::Color,

    mesh_pipeline: wgpu::RenderPipeline,
    mesh_depth_pipeline: wgpu::RenderPipeline,
    mesh_transparent_depth_pipeline: wgpu::RenderPipeline,
    text_pipeline: wgpu::RenderPipeline,
    text_depth_write_pipeline: wgpu::RenderPipeline,
    text_overlay_pipeline: wgpu::RenderPipeline,
    line_pipeline: wgpu::RenderPipeline,
    line_depth_write_pipeline: wgpu::RenderPipeline,
    line_overlay_pipeline: wgpu::RenderPipeline,
    scene_view_pipeline: wgpu::RenderPipeline,
    scene_view_overlay_pipeline: wgpu::RenderPipeline,

    pub depth: DepthTexture,

    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    uniform_bind_group_layout: wgpu::BindGroupLayout,
    uniform_slot_size: u64,
    uniform_capacity: u64,

    scene_view_uniform_buffer: wgpu::Buffer,
    scene_view_uniform_bind_group: wgpu::BindGroup,
    scene_view_uniform_bind_group_layout: wgpu::BindGroupLayout,
    scene_view_uniform_slot_size: u64,
    scene_view_uniform_capacity: u64,

    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    pub line_bind_group_layout: wgpu::BindGroupLayout,

    default_sampler: wgpu::Sampler,
    default_texture_bind_group: wgpu::BindGroup,
    scene_view_quad: MeshInstance,

    mesh_cache: HashMap<usize, Arc<MeshInstance>>,
}

impl Renderer {
    /// Initializer for the 2.1 Renderer.
    /// Note: You will need to move your pipeline creation logic (Shaders, Layouts) into here.
    pub fn new(device_mgr: Arc<DeviceManager>) -> Self {
        let device = &device_mgr.device;
        // let config = &device_mgr.config;
        let config = device_mgr.config.read();

        // 1. Create Depth Texture
        let depth = DepthTexture::create(device, &config);

        // 2. Setup Uniforms (MVP)
        let min_alignment = device.limits().min_uniform_buffer_offset_alignment as u64;
        let uniform_slot_size =
            (std::mem::size_of::<Uniforms>() as u64 + min_alignment - 1) & !(min_alignment - 1);
        let scene_view_uniform_slot_size =
            (std::mem::size_of::<SceneViewUniforms>() as u64 + min_alignment - 1)
                & !(min_alignment - 1);

        // ===============================
        // 🔑 MISSING FIELD 1: Camera Buffer
        // ===============================
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: 64, // Mat4 size
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // ===============================
        // Shaders
        // ===============================
        let mesh_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("mesh-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/mesh.wgsl"))),
        });

        let text_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("text-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/text.wgsl"))),
        });

        let line_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("line-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/line.wgsl"))),
        });

        let scene_view_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("scene-view-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!(
                "shaders/scene_view.wgsl"
            ))),
        });

        // ===============================
        // Bind group layouts
        // ===============================
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("uniform-bind-group-layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: true,
                            min_binding_size: Some(NonZeroU64::new(uniform_slot_size).unwrap()),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture-bind-group-layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let scene_view_uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("scene-view-uniform-bind-group-layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: Some(
                            NonZeroU64::new(scene_view_uniform_slot_size).unwrap(),
                        ),
                    },
                    count: None,
                }],
            });

        // 🔑 MISSING FIELD 2: Camera Bind Group Layout
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("camera_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        // 🔑 MISSING FIELD 3: Line Bind Group Layout (Storage for lines)
        let line_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("line_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        // let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        //     label: Some("pipeline-layout"),
        //     bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind_group_layout],
        //     push_constant_ranges: &[],
        // });

        let mesh_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mesh-pipeline-layout"),
            bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let text_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("text-pipeline-layout"),
            bind_group_layouts: &[&uniform_bind_group_layout, &texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        let line_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("line-pipeline-layout"),
            bind_group_layouts: &[&line_bind_group_layout, &camera_bind_group_layout],
            push_constant_ranges: &[],
        });

        let scene_view_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("scene-view-pipeline-layout"),
                bind_group_layouts: &[
                    &scene_view_uniform_bind_group_layout,
                    &texture_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let create_mesh_pipeline = |label, depth_write_enabled, depth_compare, blend| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&mesh_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &mesh_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[MeshVertex::desc()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &mesh_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: depth.format,
                    depth_write_enabled,
                    depth_compare,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                multiview: None,
                cache: None,
            })
        };
        let create_text_pipeline = |label, depth_write_enabled, depth_compare, blend| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&text_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &text_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[TextVertex::desc()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &text_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: depth.format,
                    depth_write_enabled,
                    depth_compare,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                multiview: None,
                cache: None,
            })
        };
        let create_line_pipeline = |label, depth_write_enabled, depth_compare, blend| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&line_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &line_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &line_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: depth.format,
                    depth_write_enabled,
                    depth_compare,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                multiview: None,
                cache: None,
            })
        };
        let create_scene_view_pipeline = |label, depth_compare| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&scene_view_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &scene_view_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[TextVertex::desc()],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &scene_view_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: depth.format,
                    depth_write_enabled: false,
                    depth_compare,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                multiview: None,
                cache: None,
            })
        };

        let alpha_blending = Some(wgpu::BlendState::ALPHA_BLENDING);
        let mesh_pipeline = create_mesh_pipeline(
            "mesh-overlay-pipeline",
            false,
            wgpu::CompareFunction::Always,
            alpha_blending,
        );
        let mesh_depth_pipeline = create_mesh_pipeline(
            "mesh-depth-write-pipeline",
            true,
            wgpu::CompareFunction::LessEqual,
            None,
        );
        let mesh_transparent_depth_pipeline = create_mesh_pipeline(
            "mesh-transparent-depth-pipeline",
            false,
            wgpu::CompareFunction::LessEqual,
            alpha_blending,
        );
        let text_pipeline = create_text_pipeline(
            "text-transparent-depth-pipeline",
            false,
            wgpu::CompareFunction::LessEqual,
            alpha_blending,
        );
        let text_depth_write_pipeline = create_text_pipeline(
            "text-depth-write-pipeline",
            true,
            wgpu::CompareFunction::LessEqual,
            None,
        );
        let text_overlay_pipeline = create_text_pipeline(
            "text-overlay-pipeline",
            false,
            wgpu::CompareFunction::Always,
            alpha_blending,
        );
        let line_pipeline = create_line_pipeline(
            "line-transparent-depth-pipeline",
            false,
            wgpu::CompareFunction::LessEqual,
            alpha_blending,
        );
        let line_depth_write_pipeline = create_line_pipeline(
            "line-depth-write-pipeline",
            true,
            wgpu::CompareFunction::LessEqual,
            None,
        );
        let line_overlay_pipeline = create_line_pipeline(
            "line-overlay-pipeline",
            false,
            wgpu::CompareFunction::Always,
            alpha_blending,
        );
        let scene_view_pipeline = create_scene_view_pipeline(
            "scene-view-depth-pipeline",
            wgpu::CompareFunction::LessEqual,
        );
        let scene_view_overlay_pipeline = create_scene_view_pipeline(
            "scene-view-overlay-pipeline",
            wgpu::CompareFunction::Always,
        );

        let default_sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

        let maximum_mesh_uniform_capacity =
            maximum_uniform_capacity(device.limits().max_buffer_size, uniform_slot_size);
        assert!(
            maximum_mesh_uniform_capacity > 0,
            "GPU cannot allocate even one {uniform_slot_size}-byte mesh uniform slot"
        );
        let uniform_capacity = INITIAL_UNIFORM_CAPACITY.min(maximum_mesh_uniform_capacity);
        let (uniform_buffer, uniform_bind_group) = create_uniform_resources(
            device,
            &uniform_bind_group_layout,
            &default_sampler,
            uniform_slot_size,
            uniform_capacity,
        );
        let maximum_scene_view_uniform_capacity = maximum_uniform_capacity(
            device.limits().max_buffer_size,
            scene_view_uniform_slot_size,
        );
        let scene_view_uniform_capacity =
            INITIAL_UNIFORM_CAPACITY.min(maximum_scene_view_uniform_capacity);
        let (scene_view_uniform_buffer, scene_view_uniform_bind_group) =
            create_scene_view_uniform_resources(
                device,
                &scene_view_uniform_bind_group_layout,
                scene_view_uniform_slot_size,
                scene_view_uniform_capacity,
            );

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let (_, _, default_texture_bind_group) = Self::create_texture_bind_group_from_rgba(
            device,
            &device_mgr.queue,
            &texture_bind_group_layout,
            &default_sampler,
            &[255, 255, 255, 255],
            1,
            1,
        );

        let scene_view_vertices = [
            TextVertex {
                position: [-0.5, -0.5, 0.0],
                uv: [0.0, 1.0],
                color: [1.0; 4],
            },
            TextVertex {
                position: [0.5, -0.5, 0.0],
                uv: [1.0, 1.0],
                color: [1.0; 4],
            },
            TextVertex {
                position: [0.5, 0.5, 0.0],
                uv: [1.0, 0.0],
                color: [1.0; 4],
            },
            TextVertex {
                position: [-0.5, 0.5, 0.0],
                uv: [0.0, 0.0],
                color: [1.0; 4],
            },
        ];
        let scene_view_indices = [0_u32, 1, 2, 0, 2, 3];
        let scene_view_quad = MeshInstance::new(
            device,
            bytemuck::cast_slice(&scene_view_vertices),
            bytemuck::cast_slice(&scene_view_indices),
            scene_view_indices.len() as u32,
            None,
            MeshPipelineKind::Textured,
            Vec3::ZERO,
            true,
        );

        Self {
            device_mgr: device_mgr.clone(),
            clear_color: wgpu::Color {
                r: 0.05,
                g: 0.1,
                b: 0.15,
                a: 1.0,
            },
            mesh_pipeline,
            mesh_depth_pipeline,
            mesh_transparent_depth_pipeline,
            text_pipeline,
            text_depth_write_pipeline,
            text_overlay_pipeline,
            scene_view_pipeline,
            scene_view_overlay_pipeline,
            depth,
            uniform_buffer,
            uniform_bind_group,
            uniform_bind_group_layout,
            uniform_slot_size,
            uniform_capacity,
            scene_view_uniform_buffer,
            scene_view_uniform_bind_group,
            scene_view_uniform_bind_group_layout,
            scene_view_uniform_slot_size,
            scene_view_uniform_capacity,
            mesh_cache: HashMap::new(),
            texture_bind_group_layout,
            default_sampler,
            default_texture_bind_group,
            scene_view_quad,
            line_pipeline,
            line_depth_write_pipeline,
            line_overlay_pipeline,
            camera_buffer,
            camera_bind_group,
            line_bind_group_layout, // start_time: Instant::now(),
        }
    }

    pub fn render_scene(&mut self, scene: &Scene, world: &hecs::World) -> Result<()> {
        self.render_scene_with_views(scene, world, &[], &[])
    }

    pub fn render_scene_with_views(
        &mut self,
        scene: &Scene,
        world: &hecs::World,
        child_passes: &[SceneViewPass<'_>],
        composites: &[SceneViewComposite],
    ) -> Result<()> {
        let (frame, view) = self.device_mgr.acquire_frame()?;
        let view_proj = scene.camera.view_proj_matrix();
        for child in child_passes {
            let mut child_encoder =
                self.device_mgr
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Murali SceneView Render Encoder"),
                    });
            self.encode_scene_pass(
                child.scene,
                child.world,
                &child.target.view,
                &child.target.depth.view,
                &mut child_encoder,
                child.scene.camera.view_proj_matrix(),
                wgpu::Color::TRANSPARENT,
                &[],
            )?;
            self.device_mgr.queue.submit(Some(child_encoder.finish()));
        }
        let mut encoder =
            self.device_mgr
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Murali Render Encoder"),
                });
        let parent_depth_view = self
            .depth
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.encode_scene_pass(
            scene,
            world,
            &view,
            &parent_depth_view,
            &mut encoder,
            view_proj,
            self.clear_color,
            composites,
        )?;
        self.device_mgr.queue.submit(Some(encoder.finish()));
        frame.present();
        Ok(())
    }

    pub fn render_to_image(&mut self, scene: &Scene, world: &hecs::World) -> Result<RgbaImage> {
        self.render_to_image_with_views(scene, world, &[], &[])
    }

    pub fn render_to_image_with_views(
        &mut self,
        scene: &Scene,
        world: &hecs::World,
        child_passes: &[SceneViewPass<'_>],
        composites: &[SceneViewComposite],
    ) -> Result<RgbaImage> {
        let config = self.device_mgr.config.read().clone();
        let width = config.width.max(1);
        let height = config.height.max(1);
        let device = self.device_mgr.device.clone();

        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("export-target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());

        let padded_bytes_per_row = ((width * 4 + 255) / 256) * 256;
        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("export-readback"),
            size: padded_bytes_per_row as u64 * height as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        for child in child_passes {
            let mut child_encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Murali SceneView Export Encoder"),
                });
            self.encode_scene_pass(
                child.scene,
                child.world,
                &child.target.view,
                &child.target.depth.view,
                &mut child_encoder,
                child.scene.camera.view_proj_matrix(),
                wgpu::Color::TRANSPARENT,
                &[],
            )?;
            self.device_mgr.queue.submit(Some(child_encoder.finish()));
        }
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Murali Export Encoder"),
        });
        let parent_depth_view = self
            .depth
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.encode_scene_pass(
            scene,
            world,
            &target_view,
            &parent_depth_view,
            &mut encoder,
            scene.camera.view_proj_matrix(),
            self.clear_color,
            composites,
        )?;
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &target,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &output_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        let submission = self.device_mgr.queue.submit(Some(encoder.finish()));
        self.device_mgr
            .device
            .poll(wgpu::PollType::wait_for(submission))?;

        let slice = output_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });

        let timeout = Duration::from_secs(10);
        let start = Instant::now();
        loop {
            self.device_mgr.device.poll(wgpu::PollType::Poll)?;

            match rx.try_recv() {
                Ok(result) => {
                    result.map_err(|e| anyhow::anyhow!("Export readback failed: {e:?}"))?;
                    break;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    if start.elapsed() >= timeout {
                        return Err(anyhow::anyhow!(
                            "Export readback timed out after {}s",
                            timeout.as_secs()
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(1));
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    return Err(anyhow::anyhow!("Export readback channel closed"));
                }
            }
        }

        let mapped = slice.get_mapped_range();
        let mut rgba = vec![0_u8; (width * height * 4) as usize];
        for row in 0..height as usize {
            let src_offset = row * padded_bytes_per_row as usize;
            let dst_offset = row * width as usize * 4;
            rgba[dst_offset..dst_offset + width as usize * 4]
                .copy_from_slice(&mapped[src_offset..src_offset + width as usize * 4]);
        }
        drop(mapped);
        output_buffer.unmap();

        ImageBuffer::from_raw(width, height, rgba)
            .ok_or_else(|| anyhow::anyhow!("Failed to assemble export image buffer"))
    }

    pub fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.device_mgr.resize(size);
        //
        let config = self.device_mgr.config.read();
        self.depth = DepthTexture::create(&self.device_mgr.device, &config);
    }

    //helpers
    pub fn create_texture_bind_group_from_rgba(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        rgba: &[u8],
        width: u32,
        height: u32,
    ) -> (wgpu::Texture, wgpu::TextureView, wgpu::BindGroup) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("rgba-texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture-bind-group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        });

        (texture, view, bind_group)
    }

    pub fn create_text_bind_group_from_raster(
        &self,
        rgba: &[u8],
        width: u32,
        height: u32,
    ) -> wgpu::BindGroup {
        let (_t, _v, bg) = Self::create_texture_bind_group_from_rgba(
            &self.device_mgr.device,
            &self.device_mgr.queue,
            &self.texture_bind_group_layout,
            &self.default_sampler,
            rgba,
            width,
            height,
        );
        bg
    }

    pub fn create_scene_render_target(&self, width: u32, height: u32) -> SceneRenderTarget {
        let width = width.max(1);
        let height = height.max(1);
        let device = &self.device_mgr.device;
        let format = self.device_mgr.config.read().format;
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("scene-view-target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let texture_bind_group = Arc::new(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("scene-view-texture-bind-group"),
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.default_sampler),
                },
            ],
        }));
        SceneRenderTarget {
            texture,
            view,
            depth: DepthTexture::create_sized(device, width, height),
            texture_bind_group,
            width,
            height,
        }
    }

    fn ensure_uniform_capacity(&mut self, required: u64) -> Result<()> {
        if required <= self.uniform_capacity {
            return Ok(());
        }

        let maximum = maximum_uniform_capacity(
            self.device_mgr.device.limits().max_buffer_size,
            self.uniform_slot_size,
        );
        let new_capacity = next_uniform_capacity(self.uniform_capacity, required, maximum)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "scene requires {required} mesh uniforms, but this GPU supports at most \
                     {maximum} with {}-byte aligned slots",
                    self.uniform_slot_size
                )
            })?;
        let (buffer, bind_group) = create_uniform_resources(
            &self.device_mgr.device,
            &self.uniform_bind_group_layout,
            &self.default_sampler,
            self.uniform_slot_size,
            new_capacity,
        );
        self.uniform_buffer = buffer;
        self.uniform_bind_group = bind_group;
        self.uniform_capacity = new_capacity;
        Ok(())
    }

    fn ensure_scene_view_uniform_capacity(&mut self, required: u64) -> Result<()> {
        if required <= self.scene_view_uniform_capacity {
            return Ok(());
        }
        let maximum = maximum_uniform_capacity(
            self.device_mgr.device.limits().max_buffer_size,
            self.scene_view_uniform_slot_size,
        );
        let new_capacity =
            next_uniform_capacity(self.scene_view_uniform_capacity, required, maximum)
                .ok_or_else(|| anyhow::anyhow!("scene requires too many SceneView uniforms"))?;
        let (buffer, bind_group) = create_scene_view_uniform_resources(
            &self.device_mgr.device,
            &self.scene_view_uniform_bind_group_layout,
            self.scene_view_uniform_slot_size,
            new_capacity,
        );
        self.scene_view_uniform_buffer = buffer;
        self.scene_view_uniform_bind_group = bind_group;
        self.scene_view_uniform_capacity = new_capacity;
        Ok(())
    }

    fn encode_scene_pass(
        &mut self,
        scene: &Scene,
        world: &hecs::World,
        view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
        view_proj: Mat4,
        clear_color: wgpu::Color,
        composites: &[SceneViewComposite],
    ) -> Result<()> {
        let mut draw_commands = Vec::new();
        {
            let mut query = world.query::<(
                &LineComponent,
                &ColorComponent,
                &SharedProps,
                &RenderOrderComponent,
            )>();
            for (line, color, props, order) in query.iter() {
                let props = DrawableProps::read(props);
                if !props.visible || props.opacity <= 0.0 {
                    continue;
                }

                let model = props.model_matrix();
                let start = model.transform_point3(line.start);
                let end = model.transform_point3(line.end);
                let alpha = color.0.w * props.opacity;
                draw_commands.push(DrawCommand::Line {
                    key: RenderSortKey {
                        layer: props.layer,
                        tattva_id: order.tattva_id,
                        primitive_index: order.primitive_index,
                    },
                    data: [
                        start.x,
                        start.y,
                        start.z,
                        0.0,
                        end.x,
                        end.y,
                        end.z,
                        0.0,
                        color.0.x,
                        color.0.y,
                        color.0.z,
                        alpha,
                        line.thickness,
                        line.dash_length,
                        line.gap_length,
                        line.dash_offset,
                    ],
                    world_center: (start + end) * 0.5,
                    transparent: alpha < 1.0,
                    depth_mode: props.depth_mode,
                });
            }
        }

        {
            let mut query = world.query::<(&MeshComponent, &SharedProps, &RenderOrderComponent)>();
            for (mesh_comp, props, order) in query.iter() {
                let props = DrawableProps::read(props);
                if !props.visible || props.opacity <= 0.0 {
                    continue;
                }
                draw_commands.push(DrawCommand::Mesh {
                    key: RenderSortKey {
                        layer: props.layer,
                        tattva_id: order.tattva_id,
                        primitive_index: order.primitive_index,
                    },
                    mesh: mesh_comp.0.clone(),
                    model: props.model_matrix(),
                    bind_group: mesh_comp.0.bind_group.clone(),
                    alpha: props.opacity,
                    world_center: props
                        .model_matrix()
                        .transform_point3(mesh_comp.0.local_center),
                    transparent: mesh_comp.0.has_transparency || props.opacity < 1.0,
                    depth_mode: props.depth_mode,
                });
            }
        }

        for composite in composites {
            let props = DrawableProps::read(&composite.props);
            if !props.visible || props.opacity <= 0.0 {
                continue;
            }
            let model = props.model_matrix()
                * Mat4::from_scale(Vec3::new(composite.size.x, composite.size.y, 1.0));
            draw_commands.push(DrawCommand::SceneView {
                key: RenderSortKey {
                    layer: props.layer,
                    tattva_id: composite.tattva_id,
                    primitive_index: 0,
                },
                model,
                bind_group: composite.texture_bind_group.clone(),
                uniforms: SceneViewUniforms {
                    mvp: Mat4::IDENTITY.to_cols_array_2d(),
                    size: composite.size.to_array(),
                    opacity: props.opacity,
                    corner_radius: composite.corner_radius,
                    background: composite.background.unwrap_or(Vec4::ZERO).to_array(),
                    border_color: composite.border_color.to_array(),
                    border_width: composite.border_width,
                    _padding: [0.0; 3],
                },
                world_center: props.position,
                depth_mode: props.depth_mode,
            });
        }

        let perspective = matches!(scene.camera.projection, Projection::Perspective { .. });
        if perspective {
            let view = scene.camera.view_matrix();
            draw_commands.sort_by(|a, b| compare_3d_commands(a, b, view));
        } else {
            draw_commands.sort_by_key(DrawCommand::key);
        }

        let mesh_count = draw_commands
            .iter()
            .filter(|command| matches!(command, DrawCommand::Mesh { .. }))
            .count() as u64;
        self.ensure_uniform_capacity(mesh_count)?;
        let scene_view_count = draw_commands
            .iter()
            .filter(|command| matches!(command, DrawCommand::SceneView { .. }))
            .count() as u64;
        self.ensure_scene_view_uniform_capacity(scene_view_count)?;

        let line_count = draw_commands
            .iter()
            .filter(|command| matches!(command, DrawCommand::Line { .. }))
            .count();
        let mut line_data = Vec::with_capacity(line_count * 16 * std::mem::size_of::<f32>());
        for command in &draw_commands {
            if let DrawCommand::Line { data, .. } = command {
                line_data.extend_from_slice(bytemuck::cast_slice(data));
            }
        }

        let line_resources = if line_count > 0 {
            let buffer =
                self.device_mgr
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Line Storage Buffer"),
                        contents: &line_data,
                        usage: wgpu::BufferUsages::STORAGE,
                    });
            let bind_group = self
                .device_mgr
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.line_bind_group_layout,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    }],
                    label: None,
                });
            Some((buffer, bind_group))
        } else {
            None
        };

        if line_resources.is_some() {
            self.device_mgr.queue.write_buffer(
                &self.camera_buffer,
                0,
                bytemuck::cast_slice(view_proj.as_ref()),
            );
        }
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Primary Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });

        let mut line_index = 0_u32;
        let mut mesh_index = 0_u64;
        let mut scene_view_index = 0_u64;
        let mut command_index = 0;
        while command_index < draw_commands.len() {
            match &draw_commands[command_index] {
                DrawCommand::Line { .. } => {
                    let Some((_, bind_group)) = &line_resources else {
                        break;
                    };
                    let pipeline_mode =
                        depth_pipeline_mode(&draw_commands[command_index], perspective);
                    let first_line = line_index;
                    while command_index < draw_commands.len()
                        && matches!(&draw_commands[command_index], DrawCommand::Line { .. })
                        && depth_pipeline_mode(&draw_commands[command_index], perspective)
                            == pipeline_mode
                    {
                        line_index += 1;
                        command_index += 1;
                    }
                    let pipeline = match pipeline_mode {
                        DepthPipelineMode::Overlay => &self.line_overlay_pipeline,
                        DepthPipelineMode::OpaqueWorld => &self.line_depth_write_pipeline,
                        DepthPipelineMode::TransparentWorld => &self.line_pipeline,
                    };
                    rpass.set_pipeline(pipeline);
                    rpass.set_bind_group(0, bind_group, &[]);
                    rpass.set_bind_group(1, &self.camera_bind_group, &[]);
                    rpass.draw(0..6, first_line..line_index);
                }
                DrawCommand::Mesh {
                    mesh,
                    model,
                    bind_group,
                    alpha,
                    transparent,
                    depth_mode,
                    ..
                } => {
                    let mvp = view_proj * *model;
                    let offset = (mesh_index * self.uniform_slot_size) as u32;
                    self.device_mgr.queue.write_buffer(
                        &self.uniform_buffer,
                        offset as u64,
                        bytemuck::cast_slice(&[Uniforms::from_mat4_alpha(mvp, *alpha)]),
                    );

                    let pipeline_mode = if !perspective || *depth_mode == DepthMode::Overlay {
                        DepthPipelineMode::Overlay
                    } else if *transparent {
                        DepthPipelineMode::TransparentWorld
                    } else {
                        DepthPipelineMode::OpaqueWorld
                    };
                    let pipeline = match (mesh.pipeline_kind, pipeline_mode) {
                        (MeshPipelineKind::Mesh, DepthPipelineMode::Overlay) => &self.mesh_pipeline,
                        (MeshPipelineKind::Mesh, DepthPipelineMode::OpaqueWorld) => {
                            &self.mesh_depth_pipeline
                        }
                        (MeshPipelineKind::Mesh, DepthPipelineMode::TransparentWorld) => {
                            &self.mesh_transparent_depth_pipeline
                        }
                        (
                            MeshPipelineKind::Textured | MeshPipelineKind::Text,
                            DepthPipelineMode::Overlay,
                        ) => &self.text_overlay_pipeline,
                        (
                            MeshPipelineKind::Textured | MeshPipelineKind::Text,
                            DepthPipelineMode::OpaqueWorld,
                        ) => &self.text_depth_write_pipeline,
                        (
                            MeshPipelineKind::Textured | MeshPipelineKind::Text,
                            DepthPipelineMode::TransparentWorld,
                        ) => &self.text_pipeline,
                    };
                    rpass.set_pipeline(pipeline);
                    rpass.set_bind_group(0, &self.uniform_bind_group, &[offset]);
                    if let Some(bind_group) = bind_group.as_ref() {
                        rpass.set_bind_group(1, bind_group.as_ref(), &[]);
                    } else {
                        rpass.set_bind_group(1, &self.default_texture_bind_group, &[]);
                    }
                    mesh.draw(&mut rpass);
                    mesh_index += 1;
                    command_index += 1;
                }
                DrawCommand::SceneView {
                    model,
                    bind_group,
                    uniforms,
                    depth_mode,
                    ..
                } => {
                    let mut uniforms = *uniforms;
                    uniforms.mvp = (view_proj * *model).to_cols_array_2d();
                    let offset = (scene_view_index * self.scene_view_uniform_slot_size) as u32;
                    self.device_mgr.queue.write_buffer(
                        &self.scene_view_uniform_buffer,
                        offset as u64,
                        bytemuck::cast_slice(&[uniforms]),
                    );
                    let pipeline = if !perspective || *depth_mode == DepthMode::Overlay {
                        &self.scene_view_overlay_pipeline
                    } else {
                        &self.scene_view_pipeline
                    };
                    rpass.set_pipeline(pipeline);
                    rpass.set_bind_group(0, &self.scene_view_uniform_bind_group, &[offset]);
                    rpass.set_bind_group(1, bind_group.as_ref(), &[]);
                    self.scene_view_quad.draw(&mut rpass);
                    scene_view_index += 1;
                    command_index += 1;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DepthMode, DepthPipelineMode, DrawCommand, Mat4, RenderSortKey, SceneViewUniforms, Vec3,
        compare_3d_commands, depth_pipeline_mode, maximum_uniform_capacity, next_uniform_capacity,
    };

    #[test]
    fn scene_view_shader_is_valid_wgsl() {
        let source = include_str!("shaders/scene_view.wgsl");
        let module = naga::front::wgsl::parse_str(source).expect("SceneView shader should parse");
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .expect("SceneView shader should validate");
    }

    #[test]
    fn scene_view_uniform_layout_matches_wgsl_offsets() {
        assert_eq!(std::mem::size_of::<SceneViewUniforms>(), 128);
        assert_eq!(std::mem::offset_of!(SceneViewUniforms, background), 80);
        assert_eq!(std::mem::offset_of!(SceneViewUniforms, border_color), 96);
        assert_eq!(std::mem::offset_of!(SceneViewUniforms, border_width), 112);
    }

    fn line_command(
        tattva_id: usize,
        world_center: Vec3,
        transparent: bool,
        depth_mode: DepthMode,
    ) -> DrawCommand {
        DrawCommand::Line {
            key: RenderSortKey {
                layer: 0,
                tattva_id,
                primitive_index: 0,
            },
            data: [0.0; 16],
            world_center,
            transparent,
            depth_mode,
        }
    }

    #[test]
    fn painter_order_is_stable_across_primitive_kinds() {
        let mut commands = vec![
            (
                "text",
                RenderSortKey {
                    layer: 0,
                    tattva_id: 2,
                    primitive_index: 1,
                },
            ),
            (
                "newer mesh",
                RenderSortKey {
                    layer: 0,
                    tattva_id: 3,
                    primitive_index: 0,
                },
            ),
            (
                "background mesh",
                RenderSortKey {
                    layer: -100,
                    tattva_id: 9,
                    primitive_index: 0,
                },
            ),
            (
                "line",
                RenderSortKey {
                    layer: 0,
                    tattva_id: 2,
                    primitive_index: 0,
                },
            ),
        ];

        commands.sort_by_key(|(_, key)| *key);

        assert_eq!(
            commands
                .into_iter()
                .map(|(kind, _)| kind)
                .collect::<Vec<_>>(),
            vec!["background mesh", "line", "text", "newer mesh"]
        );
    }

    #[test]
    fn perspective_order_draws_opaque_then_far_to_near_transparency_then_overlay() {
        let mut commands = vec![
            line_command(2, Vec3::new(0.0, 0.0, -2.0), true, DepthMode::World),
            line_command(4, Vec3::ZERO, true, DepthMode::Overlay),
            line_command(1, Vec3::new(0.0, 0.0, -3.0), false, DepthMode::World),
            line_command(3, Vec3::new(0.0, 0.0, -10.0), true, DepthMode::World),
        ];

        commands.sort_by(|a, b| compare_3d_commands(a, b, Mat4::IDENTITY));

        assert_eq!(
            commands
                .iter()
                .map(|command| command.key().tattva_id)
                .collect::<Vec<_>>(),
            vec![1, 3, 2, 4]
        );
    }

    #[test]
    fn orthographic_commands_use_overlay_pipelines() {
        let opaque_world = line_command(1, Vec3::ZERO, false, DepthMode::World);
        let transparent_world = line_command(2, Vec3::ZERO, true, DepthMode::World);

        assert_eq!(
            depth_pipeline_mode(&opaque_world, false),
            DepthPipelineMode::Overlay
        );
        assert_eq!(
            depth_pipeline_mode(&transparent_world, true),
            DepthPipelineMode::TransparentWorld
        );
    }

    #[test]
    fn uniform_capacity_grows_geometrically_without_a_fixed_mesh_limit() {
        assert_eq!(next_uniform_capacity(1024, 1024, 16_384), Some(1024));
        assert_eq!(next_uniform_capacity(1024, 1025, 16_384), Some(2048));
        assert_eq!(next_uniform_capacity(1024, 5000, 16_384), Some(8192));
        assert_eq!(next_uniform_capacity(8192, 12_000, 12_000), Some(12_000));
        assert_eq!(next_uniform_capacity(8192, 12_001, 12_000), None);
    }

    #[test]
    fn uniform_capacity_respects_buffer_and_dynamic_offset_limits() {
        assert_eq!(maximum_uniform_capacity(4096, 256), 16);
        assert_eq!(
            maximum_uniform_capacity(u64::MAX, 256),
            (u32::MAX as u64 / 256) + 1
        );
    }
}
