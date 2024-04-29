use entity::Entity;
use renderer::WutEngineRenderer;

pub mod entity;
pub mod renderer;
pub mod world;
pub use glam as math;

use thiserror::Error;
use winit::{application::ApplicationHandler, event::WindowEvent, event_loop::{ActiveEventLoop, ControlFlow, EventLoop}, window::{Window, WindowId}};
use world::World;

#[derive(Debug, Error)]
pub enum WutEngineError {
    #[error("No initial world set")]
    MissingWorld
}

#[derive(Debug)]
pub struct WutEngine<R: WutEngineRenderer> {
    /// Used only for initialization
    initial_world: Option<World>,

    renderer: R,
    windows: Vec<Window>,
    entities: Vec<Entity>
}

impl<R: WutEngineRenderer> ApplicationHandler for WutEngine<R> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        log::info!("Resumed");

        if let Some(initial_world) = &mut self.initial_world {
            log::info!("Initializing initial world");
            log::info!("Spawning {} entities", initial_world.get_entities().len());

            initial_world
                .get_entities()
                .drain(0..)
                .for_each(|entity| self.entities.push(entity));

            self.initial_world = None;
            log::info!("World initialization done");

            log::info!("Starting test window");

            let test_window = event_loop.create_window(Window::default_attributes()).unwrap();

            self.windows.push(test_window);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                log::info!("Close requested");
                event_loop.exit();
            },
            _ => ()
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
         
    }
}

impl<R: WutEngineRenderer> WutEngine<R> {
    pub fn new(initial_world: World) -> Self {
        log::info!("Creating new WutEngine instance with backend {}", R::NAME);

        WutEngine {
            renderer: R::default(),
            windows: Vec::new(),
            initial_world: Some(initial_world),
            entities: Vec::new()
        }
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
