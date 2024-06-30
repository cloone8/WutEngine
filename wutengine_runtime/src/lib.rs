use std::{path::Path, time::Instant};

use glam::{U64Vec2, U64Vec4};
use loading::{
    scene::SceneLoader,
    script::{ScriptLoader, ScriptLoaders},
};

mod headless_renderer;
pub use glam as math;
pub use headless_renderer::HeadlessRenderer;
pub mod loading;
pub mod serialization;

use serialization::format::SerializationFormat;
use thiserror::Error;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};
use wutengine_core::{
    fastmap::FastMap,
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
pub struct WutEngine<const W: usize, R, F>
where
    R: WutEngineRenderer,
    F: SerializationFormat,
{
    prev_frame: Instant,

    renderer: R,
    script_loaders: FastMap<ScriptLoader<F>>,
    initialized: bool,
    objects: FastMap<Object>,
    scripts: FastMap<ScriptData>,
    windows: Vec<Window>,
}

fn to_wutengine_window_id(id: WindowId) -> wutengine_core::renderer::WindowId {
    wutengine_core::renderer::WindowId::new(id.into())
}

impl<const W: usize, R, F> ApplicationHandler for WutEngine<W, R, F>
where
    R: WutEngineRenderer,
    F: SerializationFormat,
{
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        log::info!("Resumed");

        if !self.initialized {
            self.initialized = true;

            log::info!("Doing pre-first frame initialization");

            for i in 0..W {
                let attrs = WindowAttributes::default().with_title(format!("WutEngine - {}", i));

                let new_window = event_loop.create_window(attrs).unwrap();
                let window_size: U64Vec2 = {
                    let phys_size = new_window.inner_size();

                    U64Vec2 {
                        x: phys_size.width as u64,
                        y: phys_size.height as u64,
                    }
                };

                self.renderer.init_window(
                    to_wutengine_window_id(new_window.id()),
                    WindowHandles::from_window(&new_window).unwrap(),
                    window_size,
                );

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

        for window in &self.windows {
            let id = to_wutengine_window_id(window.id());

            self.renderer.render(id, &[]);
        }
    }
}

impl<const W: usize, R, F> WutEngine<W, R, F>
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

    pub fn new(script_loaders: ScriptLoaders<F>, initial_scene: &Path) -> Self {
        log::info!("Creating new WutEngine instance with backend {}", R::NAME);

        let mut engine = WutEngine {
            renderer: R::init(),
            script_loaders: script_loaders.loaders,
            windows: Vec::new(),
            objects: FastMap::new(),
            scripts: FastMap::new(),
            initialized: false,
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
