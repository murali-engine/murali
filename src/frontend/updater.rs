// src/frontend/updater.rs
//! Updater system for dynamic tattva updates
//! Attach callbacks that run every frame for a tattva.

use crate::engine::scene::Scene;
use crate::frontend::TattvaId;
use std::sync::Arc;

/// An updater function that gets called every frame
/// Receives the scene, the tattva ID, and delta time
pub type UpdaterFn = Arc<dyn Fn(&mut Scene, TattvaId, f32) + Send + Sync>;

/// Stores an updater callback for a specific tattva
#[derive(Clone)]
pub struct Updater {
    pub tattva_id: TattvaId,
    pub callback: UpdaterFn,
    pub enabled: bool,
}

impl Updater {
    pub fn new<F>(tattva_id: TattvaId, callback: F) -> Self
    where
        F: Fn(&mut Scene, TattvaId, f32) + Send + Sync + 'static,
    {
        Self {
            tattva_id,
            callback: Arc::new(callback),
            enabled: true,
        }
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn update(&self, scene: &mut Scene, dt: f32) {
        if self.enabled {
            (self.callback)(scene, self.tattva_id, dt);
        }
    }
}

/// Manager for all updaters in the scene
pub struct UpdaterManager {
    updaters: Vec<Updater>,
}

impl UpdaterManager {
    pub fn new() -> Self {
        Self {
            updaters: Vec::new(),
        }
    }

    /// Add an updater for a tattva
    pub fn add_updater<F>(&mut self, tattva_id: TattvaId, callback: F) -> usize
    where
        F: Fn(&mut Scene, TattvaId, f32) + Send + Sync + 'static,
    {
        let updater = Updater::new(tattva_id, callback);
        self.updaters.push(updater);
        self.updaters.len() - 1
    }

    /// Remove an updater by index
    pub fn remove_updater(&mut self, index: usize) {
        if index < self.updaters.len() {
            self.updaters.remove(index);
        }
    }

    /// Remove all updaters for a specific tattva
    pub fn remove_updaters_for_tattva(&mut self, tattva_id: TattvaId) {
        self.updaters.retain(|u| u.tattva_id != tattva_id);
    }

    /// Enable/disable an updater by index
    pub fn set_enabled(&mut self, index: usize, enabled: bool) {
        if let Some(updater) = self.updaters.get_mut(index) {
            if enabled {
                updater.enable();
            } else {
                updater.disable();
            }
        }
    }

    /// Update all enabled updaters
    pub fn update_all(&self, scene: &mut Scene, dt: f32) {
        // Clone the updaters to avoid borrow checker issues
        let updaters = self.updaters.clone();
        for updater in &updaters {
            updater.update(scene, dt);
        }
    }

    /// Clear all updaters
    pub fn clear(&mut self) {
        self.updaters.clear();
    }

    /// Get the number of updaters
    pub fn len(&self) -> usize {
        self.updaters.len()
    }

    /// Check if there are no updaters
    pub fn is_empty(&self) -> bool {
        self.updaters.is_empty()
    }
}

impl Default for UpdaterManager {
    fn default() -> Self {
        Self::new()
    }
}
