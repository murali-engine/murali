use anyhow::{Context, Result};
use std::path::Path;

/// Texture assets embedded in Murali and available without filesystem paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinTexture {
    BlackMarble,
    WhiteMarble,
}

impl BuiltinTexture {
    fn bytes(self) -> &'static [u8] {
        match self {
            Self::BlackMarble => {
                include_bytes!("../../assets/textures/kavriq-black-marble-texture.png")
            }
            Self::WhiteMarble => {
                include_bytes!("../../assets/textures/kavriq-white-marble-texture.png")
            }
        }
    }
}

/// CPU-side RGBA texture data that can be uploaded onto a mesh.
#[derive(Debug, Clone)]
pub struct TextureImage {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl TextureImage {
    /// Decodes a texture embedded in the Murali crate.
    pub fn builtin(texture: BuiltinTexture) -> Self {
        let image = image::load_from_memory(texture.bytes())
            .expect("Murali built-in texture bytes must contain a valid image");
        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();
        Self::from_rgba(rgba.into_raw(), width, height)
    }

    pub fn from_rgba(rgba: Vec<u8>, width: u32, height: u32) -> Self {
        Self {
            rgba,
            width,
            height,
        }
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let image = image::open(path)
            .with_context(|| format!("Failed to open texture image at {}", path.display()))?;
        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();
        Ok(Self::from_rgba(rgba.into_raw(), width, height))
    }
}

#[cfg(test)]
mod tests {
    use super::{BuiltinTexture, TextureImage};

    #[test]
    fn built_in_marble_textures_decode_without_filesystem_paths() {
        for texture in [BuiltinTexture::BlackMarble, BuiltinTexture::WhiteMarble] {
            let image = TextureImage::builtin(texture);

            assert_eq!((image.width, image.height), (1254, 1254));
            assert_eq!(image.rgba.len(), 1254 * 1254 * 4);
        }
    }
}
