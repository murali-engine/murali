// src/frontend/mod.rs

pub mod animation;
pub mod collection;
pub mod layout;
pub mod props;
pub mod style;
pub mod tattva_trait;
pub mod theme;
pub mod updater;

use crate::frontend::props::SharedProps;
use std::ops::{BitOr, BitOrAssign};

pub type TattvaId = usize;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct DirtyFlags(u8);

impl DirtyFlags {
    pub const NONE: Self = Self(0);
    pub const TRANSFORM: Self = Self(1 << 0);
    pub const GEOMETRY: Self = Self(1 << 1);
    pub const STYLE: Self = Self(1 << 2);
    pub const TEXT_LAYOUT: Self = Self(1 << 3);
    pub const RASTER: Self = Self(1 << 4);
    pub const BOUNDS: Self = Self(1 << 5);
    pub const VISIBILITY: Self = Self(1 << 6);
    pub const REBUILD: Self =
        Self(Self::GEOMETRY.0 | Self::TEXT_LAYOUT.0 | Self::RASTER.0 | Self::BOUNDS.0);
    pub const ALL: Self = Self(0xFF);

    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    pub const fn intersects(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub const fn without(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }
}

impl Default for DirtyFlags {
    fn default() -> Self {
        Self::NONE
    }
}

impl BitOr for DirtyFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for DirtyFlags {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

pub struct Tattva<T> {
    id: TattvaId,
    pub state: T,
    pub props: SharedProps,
    dirty: DirtyFlags,
}

impl<T> Tattva<T> {
    pub fn new(id: TattvaId, state: T) -> Self {
        Self {
            id,
            state,
            props: SharedProps::default(),
            dirty: DirtyFlags::GEOMETRY,
        }
    }

    pub fn mark_dirty(&mut self, flags: DirtyFlags) {
        self.dirty |= flags;
    }

    pub fn id(&self) -> TattvaId {
        self.id
    }

    pub fn dirty_flags(&self) -> DirtyFlags {
        self.dirty
    }

    pub fn has_any_dirty(&self) -> bool {
        !self.dirty.is_empty()
    }

    pub fn clear_dirty(&mut self, flags: DirtyFlags) {
        self.dirty = self.dirty.without(flags);
    }
}

pub trait IntoTattva {
    fn into_tattva(self) -> Tattva<Self>
    where
        Self: Sized;
}

impl<T> IntoTattva for T
where
    T: crate::projection::Project + crate::frontend::layout::Bounded + Send + Sync + 'static,
{
    fn into_tattva(self) -> Tattva<Self> {
        Tattva::new(0, self)
    }
}
