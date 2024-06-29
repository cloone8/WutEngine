use std::time::Instant;

use renderer::WutEngineRenderer;

pub mod renderer;
pub mod world;
pub use glam as math;

use thiserror::Error;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};
use world::World;

#[derive(Debug, Error)]
pub enum WutEngineError {
    #[error("No initial world set")]
    MissingWorld,
}

#[derive(Debug)]
pub struct WutEngine<R: WutEngineRenderer> {
    prev_frame: Instant,

    renderer: R,
    initialization_data: Option<Box<InitData>>,
    windows: Vec<Window>,
}

#[derive(Debug)]
struct InitData {
    num_windows: usize,
}

impl<R: WutEngineRenderer> ApplicationHandler for WutEngine<R> {
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
        log::debug!("Window event for window {:?}: {:?}", window_id, event);

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
        log::debug!(
            "About to wait, ms: {} FPS: {}",
            duration_secs * 1000.0,
            1.0 / duration_secs
        );

        self.prev_frame = cur_time;
    }
}

impl<R: WutEngineRenderer> WutEngine<R> {
    pub fn new(num_windows: usize, initial_world: World) -> Self {
        log::info!("Creating new WutEngine instance with backend {}", R::NAME);

        let init = InitData { num_windows };

        WutEngine {
            renderer: R::default(),
            windows: Vec::new(),
            initialization_data: Some(Box::new(init)),
            prev_frame: Instant::now(),
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
