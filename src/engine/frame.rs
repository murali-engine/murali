/// The logical composition frame owned by a scene.
///
/// A frame controls the camera's aspect ratio and visible world-space bounds.
/// Export settings control only the final pixel width.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Frame {
    /// A 16:9 frame with canonical world-space dimensions of 16 by 9.
    #[default]
    Landscape,
    /// A 9:16 frame with canonical world-space dimensions of 9 by 16.
    Portrait,
    /// A 1:1 frame with canonical world-space dimensions of 16 by 16.
    Square,
}

impl Frame {
    pub const fn landscape() -> Self {
        Self::Landscape
    }

    pub const fn portrait() -> Self {
        Self::Portrait
    }

    pub const fn square() -> Self {
        Self::Square
    }

    /// Returns the canonical visible world-space width and height.
    pub const fn logical_size(self) -> (f32, f32) {
        match self {
            Self::Landscape => (16.0, 9.0),
            Self::Portrait => (9.0, 16.0),
            Self::Square => (16.0, 16.0),
        }
    }

    pub fn aspect_ratio(self) -> f32 {
        let (width, height) = self.logical_size();
        width / height
    }

    /// Derives the output dimensions from a literal pixel width.
    pub fn pixel_dimensions(self, width: u32) -> (u32, u32) {
        let height = (width as f64 / self.aspect_ratio() as f64).round().max(1.0) as u32;
        (width.max(1), height)
    }

    /// A practical default export width for the frame's orientation.
    pub const fn default_export_width(self) -> u32 {
        match self {
            Self::Landscape => 1920,
            Self::Portrait | Self::Square => 1080,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Frame;

    #[test]
    fn derives_standard_pixel_dimensions_from_width() {
        assert_eq!(Frame::landscape().pixel_dimensions(1920), (1920, 1080));
        assert_eq!(Frame::portrait().pixel_dimensions(1080), (1080, 1920));
        assert_eq!(Frame::square().pixel_dimensions(1200), (1200, 1200));
    }
}
