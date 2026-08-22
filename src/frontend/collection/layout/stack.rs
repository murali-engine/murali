use crate::engine::scene::Scene;
use crate::frontend::TattvaId;
use crate::frontend::layout::Anchor;

#[derive(Debug, Clone)]
pub struct HStack {
    pub items: Vec<TattvaId>,
    pub gap: f32,
}

impl HStack {
    pub fn new(items: Vec<TattvaId>, gap: f32) -> Self {
        Self { items, gap }
    }

    pub fn apply(&self, scene: &mut Scene) {
        let Some(first_id) = self.items.first().copied() else {
            return;
        };

        for pair in self.items.windows(2) {
            let prev = pair[0];
            let current = pair[1];
            scene.next_to(
                current,
                prev,
                crate::frontend::layout::Direction::Right,
                self.gap,
            );
            scene.align_to(current, first_id, Anchor::Down);
        }
    }
}

#[derive(Debug, Clone)]
pub struct VStack {
    pub items: Vec<TattvaId>,
    pub gap: f32,
}

impl VStack {
    pub fn new(items: Vec<TattvaId>, gap: f32) -> Self {
        Self { items, gap }
    }

    pub fn apply(&self, scene: &mut Scene) {
        let Some(first_id) = self.items.first().copied() else {
            return;
        };

        for pair in self.items.windows(2) {
            let prev = pair[0];
            let current = pair[1];
            scene.next_to(
                current,
                prev,
                crate::frontend::layout::Direction::Down,
                self.gap,
            );
            scene.align_to(current, first_id, Anchor::Left);
        }
    }
}
