use std::any::Any;

use crate::frontend::layout::Bounds;
use crate::frontend::props::SharedProps;
use crate::frontend::{DirtyFlags, Tattva, TattvaId};
use crate::projection::ProjectionCtx;

/// Object-safe interface for all Tattvas.
/// Scene, Animation, and SyncBoundary talk ONLY through this.
pub trait TattvaTrait: Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Borrow shared transform / drawable properties
    fn props(&self) -> &SharedProps;
    fn local_bounds(&self) -> Bounds;

    /// Dirty domains tracked for this object.
    fn dirty_flags(&self) -> DirtyFlags;

    fn mark_dirty(&mut self, flags: DirtyFlags);
    fn clear_dirty(&mut self, flags: DirtyFlags);
    fn clear_all_dirty(&mut self);

    /// Identity
    fn set_id(&mut self, id: TattvaId);
    fn id(&self) -> TattvaId;

    /// Projection hook (virtual dispatch)
    fn project(&self, ctx: &mut ProjectionCtx);
}

/// Blanket implementation for the generic `Tattva<T>` struct.
/// This bridges concrete math types → dyn TattvaTrait.
impl<T> TattvaTrait for Tattva<T>
where
    T: crate::projection::Project + crate::frontend::layout::Bounded + Send + Sync + 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn props(&self) -> &SharedProps {
        &self.props
    }

    fn local_bounds(&self) -> Bounds {
        self.state.local_bounds()
    }

    fn dirty_flags(&self) -> DirtyFlags {
        self.dirty
    }

    fn mark_dirty(&mut self, flags: DirtyFlags) {
        self.dirty |= flags;
    }

    fn clear_dirty(&mut self, flags: DirtyFlags) {
        self.dirty = self.dirty.without(flags);
    }

    fn clear_all_dirty(&mut self) {
        self.dirty = DirtyFlags::NONE;
    }

    fn set_id(&mut self, id: TattvaId) {
        self.id = id;
    }

    fn id(&self) -> TattvaId {
        self.id
    }

    fn project(&self, ctx: &mut ProjectionCtx) {
        self.state.project(ctx);
    }
}
