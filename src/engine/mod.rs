// src/engine/mod.rs

pub mod app;
pub mod camera;
pub mod config;
pub mod doctor;
pub mod export;
pub mod frame;
pub mod render;
pub mod scene;
pub mod scene_view;
pub mod timeline;

use crate::backend::Backend;
use crate::backend::renderer::renderer::{SceneRenderTarget, SceneViewComposite, SceneViewPass};
use crate::backend::sync::SyncBoundary;
use crate::engine::scene::Scene;
use crate::engine::timeline::SeekError;
use crate::validation::ValidationError;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use winit::window::Window;

use glam::Vec4;

/// The Engine is the top-level owner of all systems.
pub struct Engine {
    pub scene: Scene,
    pub backend: Backend,
    sync_boundary: SyncBoundary,
    scene_view_runtimes: HashMap<crate::frontend::TattvaId, SceneViewRuntime>,
}

struct SceneViewRuntime {
    world: hecs::World,
    sync_boundary: SyncBoundary,
    target: SceneRenderTarget,
}

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error(transparent)]
    Seek(#[from] SeekError),
    #[error(transparent)]
    Validation(#[from] ValidationError),
}

impl Engine {
    pub async fn new(window: Arc<Window>) -> Self {
        let backend = Backend::new(window).await.expect("Backend init failed");

        Self {
            scene: Scene::new(),
            backend,
            sync_boundary: SyncBoundary::new(),
            scene_view_runtimes: HashMap::new(),
        }
    }

    pub fn set_clear_color(&mut self, color: Vec4) {
        self.backend.renderer.clear_color = wgpu::Color {
            r: color.x as f64,
            g: color.y as f64,
            b: color.z as f64,
            a: color.w as f64,
        };
    }

    /// The Heartbeat: This moves time forward and syncs the layers.
    pub fn update(&mut self, dt: f32) -> Result<(), EngineError> {
        // 1. Advance the Frontend (Animations & Timelines)
        self.scene.update(dt)?;
        self.scene.enforce_camera_frame_aspect();

        self.sync_scene()?;
        Ok(())
    }

    /// Reconstructs the scene at an absolute timeline time and immediately
    /// synchronizes the resulting frontend state to the renderer backend.
    pub fn seek_to(&mut self, scene_time: f32) -> Result<(), EngineError> {
        self.scene.seek_to(scene_time)?;
        self.sync_scene()?;
        Ok(())
    }

    fn sync_scene(&mut self) -> Result<(), ValidationError> {
        // 2. Perform the Sync Boundary pass
        // Project dirty tattvas and materialize GPU resources
        let device = &self.backend.renderer.device_mgr.device;

        for tattva_id in self.scene.take_removed_tattva_ids() {
            self.sync_boundary
                .remove_tattva(&mut self.backend.world, tattva_id);
        }

        for (_id, tattva) in self.scene.tattvas_iter_mut() {
            self.sync_boundary.sync_tattva(
                &mut self.backend.world,
                device,
                &self.backend.renderer,
                tattva.as_mut(),
            )?;
        }

        let active_ids = self
            .scene
            .scene_views
            .keys()
            .copied()
            .collect::<HashSet<_>>();
        self.scene_view_runtimes
            .retain(|id, _| active_ids.contains(id));

        let config = self.backend.renderer.device_mgr.config.read().clone();
        let view_ids = self.scene.scene_views.keys().copied().collect::<Vec<_>>();
        for id in view_ids {
            let view = self
                .scene
                .scene_views
                .get(&id)
                .expect("SceneView disappeared");
            let (width, height) = scene_view_target_dimensions(
                view,
                config.width,
                config.height,
                self.backend.renderer.device_mgr.max_texture_size(),
            );
            if let Some(runtime) = self.scene_view_runtimes.get(&id) {
                if runtime.target.width != width || runtime.target.height != height {
                    let target = self
                        .backend
                        .renderer
                        .create_scene_render_target(width, height);
                    self.scene_view_runtimes
                        .get_mut(&id)
                        .expect("SceneView runtime disappeared")
                        .target = target;
                }
            } else {
                let target = self
                    .backend
                    .renderer
                    .create_scene_render_target(width, height);
                self.scene_view_runtimes.insert(
                    id,
                    SceneViewRuntime {
                        world: hecs::World::new(),
                        sync_boundary: SyncBoundary::new(),
                        target,
                    },
                );
            }

            let view = self
                .scene
                .scene_views
                .get_mut(&id)
                .expect("SceneView disappeared");
            let runtime = self
                .scene_view_runtimes
                .get_mut(&id)
                .expect("SceneView runtime was not created");
            sync_scene_state(
                &mut view.scene,
                &mut runtime.world,
                &mut runtime.sync_boundary,
                &self.backend.renderer,
            )?;
        }
        Ok(())
    }

    /// Draw the current state of the Backend ECS World.
    pub fn render(&mut self) -> Result<(), anyhow::Error> {
        self.scene.enforce_camera_frame_aspect();
        let Engine {
            scene,
            backend,
            scene_view_runtimes,
            ..
        } = self;
        let (child_passes, composites) = scene_view_render_data(scene, scene_view_runtimes);
        backend
            .renderer
            .render_scene_with_views(scene, &backend.world, &child_passes, &composites)
    }

    pub fn render_to_image(&mut self) -> Result<image::RgbaImage, anyhow::Error> {
        self.scene.enforce_camera_frame_aspect();
        let Engine {
            scene,
            backend,
            scene_view_runtimes,
            ..
        } = self;
        let (child_passes, composites) = scene_view_render_data(scene, scene_view_runtimes);
        backend.renderer.render_to_image_with_views(
            scene,
            &backend.world,
            &child_passes,
            &composites,
        )
    }

    pub async fn new_with_scene(window: Arc<winit::window::Window>, scene: Scene) -> Self {
        let backend = Backend::new(window).await.expect("Backend creation failed");

        Self {
            scene,
            backend,
            sync_boundary: SyncBoundary::new(),
            scene_view_runtimes: HashMap::new(),
        }
    }

    pub async fn new_headless_with_scene(
        scene: Scene,
        width: u32,
        height: u32,
    ) -> Result<Self, anyhow::Error> {
        let backend = Backend::new_headless(width, height).await?;

        Ok(Self {
            scene,
            backend,
            sync_boundary: SyncBoundary::new(),
            scene_view_runtimes: HashMap::new(),
        })
    }
}

fn scene_view_render_data<'a>(
    scene: &'a Scene,
    runtimes: &'a HashMap<crate::frontend::TattvaId, SceneViewRuntime>,
) -> (Vec<SceneViewPass<'a>>, Vec<SceneViewComposite>) {
    let mut passes = Vec::new();
    let mut composites = Vec::new();
    for (id, view) in &scene.scene_views {
        let Some(runtime) = runtimes.get(id) else {
            continue;
        };
        let Some(tattva) = scene.get_tattva_any(*id) else {
            continue;
        };
        passes.push(SceneViewPass {
            scene: &view.scene,
            world: &runtime.world,
            target: &runtime.target,
        });
        composites.push(SceneViewComposite {
            tattva_id: *id,
            props: tattva.props().clone(),
            texture_bind_group: runtime.target.texture_bind_group.clone(),
            size: view.size,
            background: view.background,
            corner_radius: view.corner_radius,
            border_width: view.border_width,
            border_color: view.border_color,
        });
    }
    (passes, composites)
}

fn sync_scene_state(
    scene: &mut Scene,
    world: &mut hecs::World,
    sync_boundary: &mut SyncBoundary,
    renderer: &crate::backend::renderer::Renderer,
) -> Result<(), ValidationError> {
    for tattva_id in scene.take_removed_tattva_ids() {
        sync_boundary.remove_tattva(world, tattva_id);
    }
    let device = &renderer.device_mgr.device;
    for (_id, tattva) in scene.tattvas_iter_mut() {
        sync_boundary.sync_tattva(world, device, renderer, tattva.as_mut())?;
    }
    Ok(())
}

fn scene_view_target_dimensions(
    view: &crate::engine::scene_view::SceneView,
    parent_width: u32,
    parent_height: u32,
    maximum: u32,
) -> (u32, u32) {
    if let Some((width, height)) = view.resolution {
        return (width.min(maximum).max(1), height.min(maximum).max(1));
    }

    let aspect = view.scene.frame().aspect_ratio();
    let mut width = parent_width.max(1) as f32;
    let mut height = width / aspect;
    if height > parent_height.max(1) as f32 {
        height = parent_height.max(1) as f32;
        width = height * aspect;
    }
    let limit_scale = (maximum as f32 / width)
        .min(maximum as f32 / height)
        .min(1.0);
    (
        (width * limit_scale).round().max(1.0) as u32,
        (height * limit_scale).round().max(1.0) as u32,
    )
}
