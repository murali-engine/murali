use anyhow::{Context, Result};
use std::path::Path;
use std::sync::{Arc, OnceLock};

/// Texture assets embedded in Murali and available without filesystem paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinTexture {
    BlackMarble,
    WhiteMarble,
    EarthMap,
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
            Self::EarthMap => include_bytes!("assets/earthmap1k.jpg"),
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
    fn decode_builtin(texture: BuiltinTexture) -> Self {
        let image = image::load_from_memory(texture.bytes())
            .expect("Murali built-in texture bytes must contain a valid image");
        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();
        Self::from_rgba(rgba.into_raw(), width, height)
    }

    /// Decodes a texture embedded in the Murali crate.
    pub fn builtin(texture: BuiltinTexture) -> Self {
        Self::builtin_shared(texture).as_ref().clone()
    }

    /// Cached built-in texture. Cloning the returned `Arc` does not decode or copy pixels.
    pub fn builtin_shared(texture: BuiltinTexture) -> Arc<Self> {
        fn cache(slot: &OnceLock<Arc<TextureImage>>, texture: BuiltinTexture) -> Arc<TextureImage> {
            slot.get_or_init(|| Arc::new(TextureImage::decode_builtin(texture)))
                .clone()
        }
        match texture {
            BuiltinTexture::BlackMarble => {
                static CACHED: OnceLock<Arc<TextureImage>> = OnceLock::new();
                cache(&CACHED, texture)
            }
            BuiltinTexture::WhiteMarble => {
                static CACHED: OnceLock<Arc<TextureImage>> = OnceLock::new();
                cache(&CACHED, texture)
            }
            BuiltinTexture::EarthMap => {
                static CACHED: OnceLock<Arc<TextureImage>> = OnceLock::new();
                cache(&CACHED, texture)
            }
        }
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
        for texture in [
            BuiltinTexture::BlackMarble,
            BuiltinTexture::WhiteMarble,
            BuiltinTexture::EarthMap,
        ] {
            let image = TextureImage::builtin(texture);

            assert_eq!(
                image.rgba.len(),
                image.width as usize * image.height as usize * 4
            );
        }
    }
}
