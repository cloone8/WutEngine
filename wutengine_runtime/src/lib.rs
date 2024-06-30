use std::{path::Path, time::Instant};

use glam::U64Vec2;
use loading::{
    scene::SceneLoader,
    script::{ScriptLoader, ScriptLoaders},
};

mod headless_renderer;
pub use glam as math;
pub use headless_renderer::HeadlessRenderer;
pub mod loading;
pub mod serialization;
pub mod settings;
pub mod window;

use serialization::format::SerializationFormat;
use settings::Settings;
use thiserror::Error;
use window::{Window, MAIN_WINDOW_ID};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::WindowId,
};
use wutengine_core::{
    lookuptable::LookupTable,
    object::Object,
    renderer::{WindowHandles, WutEngineRenderer},
    scene::Scene,
    script::ScriptData,
};

#[derive(Debug, Error)]
pub enum WutEngineError {
    #[error("No initial world set")]
    MissingWorld,
}

#[derive(Debug)]
pub struct WutEngine<R, F>
where
    R: WutEngineRenderer,
    F: SerializationFormat,
{
    prev_frame: Instant,

    renderer: R,
    script_loaders: LookupTable<ScriptLoader<F>>,
    initialization_settings: Option<Box<Settings>>,
    objects: LookupTable<Object>,
    scripts: LookupTable<ScriptData>,
    windows: Vec<Window>,
}

fn to_wutengine_window_id(id: WindowId) -> wutengine_core::renderer::WindowId {
    wutengine_core::renderer::WindowId::new(id.into())
}

impl<R, F> ApplicationHandler for WutEngine<R, F>
where
    R: WutEngineRenderer,
    F: SerializationFormat,
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        log::info!("Resumed");

        if let Some(settings) = &self.initialization_settings {
            log::info!("Doing pre-first frame initialization");

            if settings.open_window {
                Window::open(MAIN_WINDOW_ID, self, event_loop, (800, 600));
            }

            log::info!("Pre-first frame initialization done");
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        log::trace!("Window event for window {:?}: {:?}", window_id, event);

        match event {
            WindowEvent::CloseRequested => {
                log::info!("Close requested");
                event_loop.exit();
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let cur_time = Instant::now();
        let duration_secs = cur_time.duration_since(self.prev_frame).as_secs_f64();
        log::trace!(
            "About to wait, ms: {} FPS: {}",
            duration_secs * 1000.0,
            1.0 / duration_secs
        );

        self.prev_frame = cur_time;

        for window in &self.windows {
            self.renderer.render(window.id, &[]);
        }
    }
}

impl<R, F> WutEngine<R, F>
where
    R: WutEngineRenderer,
    F: SerializationFormat,
{
    fn add_scene(&mut self, scene: Scene) {
        for (_, obj) in scene.objects {
            self.objects.insert(obj);
        }

        for (_, script) in scene.scripts {
            self.scripts.insert(script);
        }
    }

    pub fn new(settings: Settings, script_loaders: ScriptLoaders<F>, initial_scene: &Path) -> Self {
        log::info!("Creating new WutEngine instance with backend {}", R::NAME);

        let mut engine = WutEngine {
            renderer: R::init(),
            script_loaders: script_loaders.loaders,
            windows: Vec::new(),
            objects: LookupTable::new(),
            scripts: LookupTable::new(),
            initialization_settings: Some(Box::new(settings)),
            prev_frame: Instant::now(),
        };

        let loaded_scene = SceneLoader::load(initial_scene, &engine.script_loaders).unwrap();

        engine.add_scene(loaded_scene);

        engine
    }

    pub fn run(mut self) -> Result<(), WutEngineError> {
        log::info!("Starting WutEngine");

        let event_loop = EventLoop::new().unwrap();

        event_loop.set_control_flow(ControlFlow::Poll);

        event_loop.run_app(&mut self).unwrap();

        log::info!("WutEngine exited succesfully");
        Ok(())
    }
}
