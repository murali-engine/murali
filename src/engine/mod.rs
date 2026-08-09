// src/engine/mod.rs

pub mod app;
pub mod camera;
pub mod config;
pub mod doctor;
pub mod export;
pub mod render;
pub mod scene;
pub mod timeline;

use crate::backend::Backend;
use crate::backend::sync::SyncBoundary;
use crate::engine::scene::Scene;
use crate::engine::timeline::SeekError;
use crate::validation::ValidationError;

use std::sync::Arc;
use winit::window::Window;

use glam::Vec4;

/// The Engine is the top-level owner of all systems.
pub struct Engine {
    pub scene: Scene,
    pub backend: Backend,
    sync_boundary: SyncBoundary,
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
        Ok(())
    }

    /// Draw the current state of the Backend ECS World.
    pub fn render(&mut self) -> Result<(), anyhow::Error> {
        self.backend
            .renderer
            .render_scene(&self.scene, &self.backend.world)
    }

    pub async fn new_with_scene(window: Arc<winit::window::Window>, scene: Scene) -> Self {
        let backend = Backend::new(window).await.expect("Backend creation failed");

        Self {
            scene,
            backend,
            sync_boundary: SyncBoundary::new(),
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
        })
    }
}
