// murali/src/engine/render/options.rs

#[derive(Debug, Clone, Default)]
pub struct RenderOptions {
    pub video: Option<bool>,
    pub frames: Option<bool>,
    pub fps: Option<u32>,
    /// Final output width in pixels. Height is derived from the scene frame.
    pub width: Option<u32>,
}

impl RenderOptions {
    pub fn video_enabled(&self) -> bool {
        self.video.unwrap_or(true)
    }

    pub fn frames_enabled(&self) -> bool {
        self.frames.unwrap_or(true)
    }

    pub fn preserve_frames_explicitly_requested(&self) -> bool {
        self.frames == Some(true)
    }
}
