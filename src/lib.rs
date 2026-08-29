pub mod backend;
pub mod colors;
pub mod engine;
pub mod frontend;
pub mod math;
pub mod palette;
pub mod positions;
pub mod prelude;
pub mod projection;
pub mod resource;
pub mod utils;
pub mod validation;

#[cfg(feature = "python")]
mod python;

// Re-export common types for ergonomics
pub use engine::EngineError;
pub use engine::app::App;
pub use engine::frame::Frame;
pub use engine::render::RenderOptions;
pub use engine::scene::{GifCapture, Scene, ScreenshotCapture};
pub use engine::scene_view::{SceneView, SceneViewPlayback};
pub use engine::timeline::{Clip, SeekError, Timeline};
pub use frontend::Tattva;
pub use frontend::collection;
pub use frontend::props::{DepthMode, layers};
pub use palette::Palette;
pub use resource::text::register_font_path;
pub use resource::texture::{BuiltinTexture, TextureImage};
pub use validation::ValidationError;
