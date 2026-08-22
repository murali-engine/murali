use crate::engine::scene::Scene;
use crate::frontend::TattvaId;
use crate::frontend::layout::{Anchor, Bounds, Direction};

#[derive(Debug, Clone)]
pub struct Group {
    pub items: Vec<TattvaId>,
}

impl Group {
    pub fn new(items: Vec<TattvaId>) -> Self {
        Self { items }
    }

    pub fn bounds(&self, scene: &Scene) -> Option<Bounds> {
        let mut iter = self.items.iter().filter_map(|id| scene.world_bounds(*id));
        let first = iter.next()?;
        Some(iter.fold(first, |acc, bounds| acc.union(&bounds)))
    }

    pub fn move_to(&self, scene: &mut Scene, center: glam::Vec2) {
        let Some(group_bounds) = self.bounds(scene) else {
            return;
        };
        let delta = center - group_bounds.center();
        for id in &self.items {
            if let Some(bounds) = scene.world_bounds(*id) {
                scene.set_position_2d(*id, bounds.center() + delta);
            }
        }
    }

    pub fn next_to(&self, scene: &mut Scene, target: TattvaId, direction: Direction, padding: f32) {
        let Some(group_bounds) = self.bounds(scene) else {
            return;
        };
        let temp_id = self.items.first().copied();
        if temp_id.is_none() {
            return;
        }
        let Some(target_bounds) = scene.world_bounds(target) else {
            return;
        };
        let offset = match direction {
            Direction::Up => glam::vec2(0.0, padding + group_bounds.height() * 0.5),
            Direction::Down => glam::vec2(0.0, -(padding + group_bounds.height() * 0.5)),
            Direction::Left => glam::vec2(-(padding + group_bounds.width() * 0.5), 0.0),
            Direction::Right => glam::vec2(padding + group_bounds.width() * 0.5, 0.0),
        };
        self.move_to(scene, target_bounds.center() + offset);
    }

    pub fn align_to(&self, scene: &mut Scene, target: TattvaId, anchor: Anchor) {
        let Some(group_bounds) = self.bounds(scene) else {
            return;
        };
        let Some(target_bounds) = scene.world_bounds(target) else {
            return;
        };
        let delta = target_bounds.anchor(anchor) - group_bounds.anchor(anchor);
        for id in &self.items {
            if let Some(bounds) = scene.world_bounds(*id) {
                scene.set_position_2d(*id, bounds.center() + delta);
            }
        }
    }
}
