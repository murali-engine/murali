//! Seekable Kavriq splash ident: the drop fills, falls, and breaks into a puddle.
//!
//! ```text
//! cargo run --release --example kavriq_splash
//! ```

use glam::Vec3;
use murali::App;
use murali::engine::scene::Scene;
use murali::engine::timeline::Timeline;
use murali::frontend::collection::composite::{KAVRIQ_SPLASH_DURATION, KavriqSplashLogo};

fn main() -> anyhow::Result<()> {
    let mut scene = Scene::new();
    scene.set_background(glam::Vec4::new(
        7.0 / 255.0,
        10.0 / 255.0,
        22.0 / 255.0,
        1.0,
    ));

    let logo = scene.add_tattva(KavriqSplashLogo::new(), Vec3::ZERO);
    let mut timeline = Timeline::new();
    timeline.call_during_reversible(
        0.0,
        KAVRIQ_SPLASH_DURATION,
        move |scene, progress| {
            if let Some(mark) = scene.get_tattva_typed_mut::<KavriqSplashLogo>(logo) {
                mark.state.time = progress * mark.state.duration;
            }
        },
        move |scene| {
            if let Some(mark) = scene.get_tattva_typed_mut::<KavriqSplashLogo>(logo) {
                mark.state.time = 0.0;
            }
        },
    );
    scene.play(timeline)?;
    App::new()?.with_scene(scene).run_app()
}
