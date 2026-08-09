use glam::{Mat4, Vec2, Vec3, vec2};

#[derive(Copy, Clone, Debug)]
pub enum Place {
    Center,
    Up(f32),
    Down(f32),
    Left(f32),
    Right(f32),
}

#[derive(Copy, Clone, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Anchor {
    Center,
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Bounds {
    pub min: Vec2,
    pub max: Vec2,
}

impl Bounds {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    pub fn from_center_size(center: Vec2, size: Vec2) -> Self {
        let half = size * 0.5;
        Self {
            min: center - half,
            max: center + half,
        }
    }

    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    pub fn size(&self) -> Vec2 {
        vec2(self.width(), self.height())
    }

    pub fn center(&self) -> Vec2 {
        (self.min + self.max) * 0.5
    }

    pub fn translate(&self, delta: Vec2) -> Self {
        Self {
            min: self.min + delta,
            max: self.max + delta,
        }
    }

    pub fn scale(&self, scale: Vec2) -> Self {
        let center = self.center();
        let half = self.size() * 0.5;
        let scaled_half = vec2(half.x * scale.x.abs(), half.y * scale.y.abs());
        Self {
            min: center - scaled_half,
            max: center + scaled_half,
        }
    }

    /// Returns the axis-aligned XY bounds after transforming the local corners.
    pub fn transform_3d(&self, transform: Mat4) -> Self {
        let corners = [
            Vec3::new(self.min.x, self.min.y, 0.0),
            Vec3::new(self.max.x, self.min.y, 0.0),
            Vec3::new(self.max.x, self.max.y, 0.0),
            Vec3::new(self.min.x, self.max.y, 0.0),
        ];
        let first = transform.transform_point3(corners[0]).truncate();
        corners[1..]
            .iter()
            .fold(Self::new(first, first), |bounds, corner| {
                let point = transform.transform_point3(*corner).truncate();
                Self::new(bounds.min.min(point), bounds.max.max(point))
            })
    }

    pub fn union(&self, other: &Self) -> Self {
        Self {
            min: vec2(self.min.x.min(other.min.x), self.min.y.min(other.min.y)),
            max: vec2(self.max.x.max(other.max.x), self.max.y.max(other.max.y)),
        }
    }

    pub fn anchor(&self, anchor: Anchor) -> Vec2 {
        match anchor {
            Anchor::Center => self.center(),
            Anchor::Up => vec2(self.center().x, self.max.y),
            Anchor::Down => vec2(self.center().x, self.min.y),
            Anchor::Left => vec2(self.min.x, self.center().y),
            Anchor::Right => vec2(self.max.x, self.center().y),
            Anchor::UpLeft => vec2(self.min.x, self.max.y),
            Anchor::UpRight => vec2(self.max.x, self.max.y),
            Anchor::DownLeft => vec2(self.min.x, self.min.y),
            Anchor::DownRight => vec2(self.max.x, self.min.y),
        }
    }
}

impl Default for Bounds {
    fn default() -> Self {
        Self {
            min: Vec2::ZERO,
            max: Vec2::ZERO,
        }
    }
}

pub trait Bounded {
    fn local_bounds(&self) -> Bounds;
}

pub fn opposite_anchor(direction: Direction) -> Anchor {
    match direction {
        Direction::Up => Anchor::Down,
        Direction::Down => Anchor::Up,
        Direction::Left => Anchor::Right,
        Direction::Right => Anchor::Left,
    }
}

pub fn anchor_for_direction(direction: Direction) -> Anchor {
    match direction {
        Direction::Up => Anchor::Up,
        Direction::Down => Anchor::Down,
        Direction::Left => Anchor::Left,
        Direction::Right => Anchor::Right,
    }
}
