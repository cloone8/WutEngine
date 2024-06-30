use std::{path::Path, time::Instant};

use loading::{
    scene::SceneLoader,
    script::{ScriptLoader, ScriptLoaders},
};

pub mod renderer;
pub use glam as math;
pub mod loading;
pub mod serialization;

use serialization::{format::SerializationFormat, scene::SerializedScene};
use thiserror::Error;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};
use wutengine_core::{
    fastmap::FastMap, object::Object, renderer::WutEngineRenderer, scene::Scene, script::ScriptData,
};

#[derive(Debug, Error)]
pub enum WutEngineError {
    #[error("No initial world set")]
    MissingWorld,
}

#[derive(Debug)]
pub struct WutEngine<R: WutEngineRenderer, F: SerializationFormat> {
    prev_frame: Instant,

    renderer: R,
    script_loaders: FastMap<ScriptLoader<F>>,
    initialization_data: Option<Box<InitData>>,
    objects: FastMap<Object>,
    scripts: FastMap<ScriptData>,
    windows: Vec<Window>,
}

#[derive(Debug)]
struct InitData {
    num_windows: usize,
}

impl<R: WutEngineRenderer, F: SerializationFormat> ApplicationHandler for WutEngine<R, F> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        log::info!("Resumed");

        if let Some(init) = self.initialization_data.take() {
            log::info!("Doing pre-first frame initialization");

            for _ in 0..init.num_windows {
                let attrs = WindowAttributes::default().with_title("WutEngine");

                let new_window = event_loop.create_window(attrs).unwrap();

                self.windows.push(new_window);
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
    }
}

impl<R: WutEngineRenderer, F: SerializationFormat> WutEngine<R, F> {
    fn add_scene(&mut self, scene: Scene) {
        for (_, obj) in scene.objects {
            self.objects.insert(obj);
        }

        for (_, script) in scene.scripts {
            self.scripts.insert(script);
        }
    }

    pub fn new(num_windows: usize, script_loaders: ScriptLoaders<F>, initial_scene: &Path) -> Self {
        log::info!("Creating new WutEngine instance with backend {}", R::NAME);

        let init = InitData { num_windows };

        let mut engine = WutEngine {
            renderer: R::default(),
            script_loaders: script_loaders.loaders,
            windows: Vec::new(),
            objects: FastMap::new(),
            scripts: FastMap::new(),
            initialization_data: Some(Box::new(init)),
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
