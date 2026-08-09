// src/engine/app.rs

use anyhow::Result;
use std::sync::Arc;
use std::time::{Duration, Instant};

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

use crate::engine::Engine;
use crate::engine::camera::controller::{
    ActiveCameraController, orbit::OrbitCameraController, pan_zoom::PanZoomCameraController,
};
use crate::engine::config::RenderConfig;
use crate::engine::export::{ExportSettings, export_scene};
use crate::engine::frame::Frame;
use crate::engine::render::RenderOptions;
use crate::engine::scene::Scene;
use crate::frontend::theme::Theme;

const AUTO_CLOSE_DELAY_SECONDS: f32 = 5.0;

pub struct App {
    window: Option<Arc<Window>>,
    engine: Option<Engine>,
    pending_scene: Option<Scene>,
    explicit_export_settings: Option<ExportSettings>,
    render_options: RenderOptions,
    preview_dt: f32,
    preview_frame_duration: Duration,
    preview_start_time: Option<Instant>,
    preview_frame_count: u64,
    auto_close_preview: bool,
    camera_controller: ActiveCameraController,
    is_left_mouse_down: bool,
    last_cursor_position: Option<(f64, f64)>,
    debug_mode: bool,
}

impl App {
    pub fn new() -> Result<Self> {
        let preview_dt = 1.0 / RenderConfig::preview()?.fps.max(1) as f32;
        Ok(Self {
            preview_dt,
            preview_frame_duration: Duration::from_secs_f32(preview_dt),
            preview_start_time: None,
            preview_frame_count: 0,
            auto_close_preview: false,
            window: None,
            engine: None,
            pending_scene: None,
            explicit_export_settings: None,
            render_options: RenderOptions::default(),
            camera_controller: ActiveCameraController::Orbit(OrbitCameraController::new(10.0)),
            is_left_mouse_down: false,
            last_cursor_position: None,
            debug_mode: false,
        })
    }

    pub fn run_app(mut self) -> Result<()> {
        let args: Vec<String> = std::env::args().collect();
        self.debug_mode = args.iter().any(|arg| arg == "--debug");
        self.auto_close_preview = args.iter().any(|arg| arg == "--auto-close");

        if should_preview(&args, &self.render_options) {
            print_camera_help();
            let event_loop = EventLoop::new()?;
            return event_loop
                .run_app(&mut self)
                .map_err(|e| anyhow::anyhow!(e));
        }

        let scene = self.pending_scene.take().unwrap_or_else(Scene::new);
        let mut settings = match self.explicit_export_settings.take() {
            Some(settings) => settings,
            None => ExportSettings::from_project_config(&scene, &self.render_options)?,
        };
        if args.iter().any(|arg| arg == "--no-video") {
            settings.video_enabled = false;
            settings.preserve_frame_exports = false;
        }
        export_scene(scene, &settings)
    }

    pub fn with_render_options(mut self, options: RenderOptions) -> Self {
        self.render_options = options;
        self
    }

    pub fn with_preview(mut self) -> Self {
        self.render_options.video = Some(false);
        self
    }

    pub fn with_video_export(mut self) -> Self {
        self.render_options.video = Some(true);
        self
    }

    pub fn with_frames_export(mut self, enabled: bool) -> Self {
        self.render_options.frames = Some(enabled);
        self
    }

    pub fn with_export_settings(mut self, settings: ExportSettings) -> Self {
        self.explicit_export_settings = Some(settings);
        self
    }

    pub fn with_scene(mut self, scene: Scene) -> Self {
        self.pending_scene = Some(scene);
        self
    }
}

impl<'a> ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let scene = self.pending_scene.take().unwrap_or_else(Scene::new);
        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_title("Murali")
                    .with_inner_size(preview_window_size(scene.frame())),
            )
            .expect("Failed to create window");

        let arc_window = Arc::new(window);

        let mut engine =
            pollster::block_on(async { Engine::new_with_scene(arc_window.clone(), scene).await });

        // let bg = Theme::global().background;
        engine.set_clear_color(Theme::global().background);

        self.window = Some(arc_window.clone());
        self.engine = Some(engine);
        self.preview_start_time = Some(Instant::now());
        self.preview_frame_count = 0;
        event_loop.set_control_flow(ControlFlow::Poll);
        arc_window.request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        let Some(engine) = self.engine.as_mut() else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::RedrawRequested => {
                let Some(start_time) = self.preview_start_time else {
                    return;
                };

                // Drive the engine
                if let Err(error) = engine.update(self.preview_dt) {
                    eprintln!("Engine update error: {error}");
                    event_loop.exit();
                    return;
                }
                self.preview_frame_count += 1;

                if self.debug_mode {
                    use std::io::{Write, stdout};
                    print!(
                        "\r[DEBUG] Frame: {:>5} | Time: {:>7.3}s | DT: {:>6.2}ms",
                        self.preview_frame_count,
                        engine.scene.scene_time,
                        self.preview_dt * 1000.0
                    );
                    let _ = stdout().flush();
                }

                if let Err(e) = engine.render() {
                    eprintln!("Render error: {:?}", e);
                    event_loop.exit();
                    return;
                }

                let timeline_end = engine
                    .scene
                    .timeline
                    .as_ref()
                    .map_or(0.0, |timeline| timeline.end_time());
                if self.auto_close_preview
                    && preview_has_reached_auto_close(
                        engine.scene.scene_time,
                        timeline_end,
                        AUTO_CLOSE_DELAY_SECONDS,
                    )
                {
                    event_loop.exit();
                    return;
                }

                let target_elapsed = self
                    .preview_frame_duration
                    .mul_f64(self.preview_frame_count as f64);
                let actual_elapsed = start_time.elapsed();

                if actual_elapsed < target_elapsed {
                    event_loop
                        .set_control_flow(ControlFlow::WaitUntil(start_time + target_elapsed));
                } else {
                    event_loop.set_control_flow(ControlFlow::Poll);
                }

                window.request_redraw();
            }

            WindowEvent::Resized(size) => {
                let corrected = enforce_aspect(size, engine.scene.frame().aspect_ratio());
                if corrected != size {
                    let _ = window.request_inner_size(corrected);
                }
                // engine.backend.renderer.device_mgr.resize(corrected);
                engine.backend.renderer.resize(corrected);
            }

            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::KeyO) => {
                        self.camera_controller =
                            ActiveCameraController::Orbit(OrbitCameraController::new(10.0))
                    }
                    PhysicalKey::Code(KeyCode::KeyP) => {
                        self.camera_controller =
                            ActiveCameraController::PanZoom(PanZoomCameraController::new())
                    }
                    PhysicalKey::Code(KeyCode::Escape) => event_loop.exit(),
                    _ => {}
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                if let Some((lx, ly)) = self.last_cursor_position {
                    if self.is_left_mouse_down {
                        let delta = glam::vec2((position.x - lx) as f32, (position.y - ly) as f32);
                        self.camera_controller
                            .handle_mouse_drag(delta, engine.scene.camera_mut());
                    }
                }
                self.last_cursor_position = Some((position.x, position.y));
            }

            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left {
                    self.is_left_mouse_down = state == ElementState::Pressed;
                }
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let scroll = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y,
                    winit::event::MouseScrollDelta::PixelDelta(p) => p.y as f32 * 0.01,
                };
                self.camera_controller
                    .handle_scroll(scroll, engine.scene.camera_mut());
            }

            _ => {}
        }
    }
}

fn enforce_aspect(size: PhysicalSize<u32>, ratio: f32) -> PhysicalSize<u32> {
    let width = size.width.max(1);
    let height = size.height.max(1);
    let w = width as f32;
    let h = height as f32;
    if (w / h) > ratio {
        PhysicalSize::new((h * ratio).round().max(1.0) as u32, height)
    } else {
        PhysicalSize::new(width, (w / ratio).round().max(1.0) as u32)
    }
}

fn preview_window_size(frame: Frame) -> PhysicalSize<u32> {
    let ratio = frame.aspect_ratio();
    let max_width: f32 = 1280.0;
    let max_height: f32 = 960.0;
    let width = max_width.min(max_height * ratio);
    let height = width / ratio;
    PhysicalSize::new(width.round() as u32, height.round() as u32)
}

fn print_camera_help() {
    println!("\n🎥 Controls: [O] Orbit | [P] PanZoom | [Drag] Move | [Wheel] Zoom\n");
}

fn should_preview(args: &[String], options: &RenderOptions) -> bool {
    if args.iter().any(|arg| arg == "--preview") {
        return true;
    }
    if args.iter().any(|arg| arg == "--export") {
        return false;
    }
    !options.video_enabled()
}

fn preview_has_reached_auto_close(scene_time: f32, timeline_end: f32, delay: f32) -> bool {
    scene_time >= timeline_end + delay
}

#[cfg(test)]
mod tests {
    use super::{enforce_aspect, preview_has_reached_auto_close, preview_window_size};
    use crate::engine::frame::Frame;
    use winit::dpi::PhysicalSize;

    #[test]
    fn auto_close_waits_five_seconds_after_timeline_completion() {
        assert!(!preview_has_reached_auto_close(14.999, 10.0, 5.0));
        assert!(preview_has_reached_auto_close(15.0, 10.0, 5.0));
    }

    #[test]
    fn auto_close_handles_scenes_without_a_timeline() {
        assert!(!preview_has_reached_auto_close(4.999, 0.0, 5.0));
        assert!(preview_has_reached_auto_close(5.0, 0.0, 5.0));
    }

    #[test]
    fn preview_sizes_follow_the_scene_frame() {
        assert_eq!(
            preview_window_size(Frame::landscape()),
            PhysicalSize::new(1280, 720)
        );
        assert_eq!(
            preview_window_size(Frame::portrait()),
            PhysicalSize::new(540, 960)
        );
        assert_eq!(
            preview_window_size(Frame::square()),
            PhysicalSize::new(960, 960)
        );
    }

    #[test]
    fn resize_enforces_the_requested_aspect() {
        assert_eq!(
            enforce_aspect(
                PhysicalSize::new(800, 800),
                Frame::portrait().aspect_ratio()
            ),
            PhysicalSize::new(450, 800)
        );
        assert_eq!(
            enforce_aspect(PhysicalSize::new(1200, 800), Frame::square().aspect_ratio()),
            PhysicalSize::new(800, 800)
        );
    }
}
