use std::collections::HashMap;

use thiserror::Error;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder, WindowId},
};

#[derive(Debug)]
pub struct WutEngine {
    event_loop: EventLoop<()>,
    windows: HashMap<WindowId, Window>
}

#[derive(Debug)]
pub struct WutEngineStarter {
    initial_windows: usize
}

impl WutEngineStarter {
    pub fn new() -> Self {
        Self::default()   
    }

    pub fn with_initial_windows(self, num: usize) -> Self {
        WutEngineStarter {
            initial_windows: num,
        }
    }

    fn build_window<T>(&self, event_loop: &EventLoop<T>) -> (WindowId, Window) {
        let window = WindowBuilder::new()
            .build(event_loop).unwrap();

        (window.id(), window)
    }

    pub fn init(self) -> WutEngine {
        let event_loop = EventLoop::new().unwrap();

        let windows: HashMap<WindowId, Window> = (0..self.initial_windows)
            .map(|_| self.build_window(&event_loop))
            .collect();

        event_loop.set_control_flow(ControlFlow::Poll);

        WutEngine {
            event_loop,
            windows
        }
    }
}

impl Default for WutEngineStarter {
    fn default() -> Self {
        WutEngineStarter {
            initial_windows: 1
        }
    }
}

#[derive(Debug, Error)]
pub enum WutEngineError {
    #[error("Unknown WutEngine error")]
    Unknown
}

#[derive(Debug)]
enum WutEngineWindowAction {
    CloseGame,
}

fn handle_window_event(w_id: WindowId, event: &WindowEvent) -> Option<WutEngineWindowAction> {
    match event {
        WindowEvent::CloseRequested => Some(WutEngineWindowAction::CloseGame),
        _ => None
    }
}

impl WutEngine {
    pub fn run(self) -> Result<(), WutEngineError> {

        let eloop_result = self.event_loop.run(move |event, elwt| {
            match event {
                Event::WindowEvent { window_id, event } => {
                    let requested_action = handle_window_event(window_id, &event);

                    if let Some(action) = requested_action {
                        match action {
                            WutEngineWindowAction::CloseGame => elwt.exit()
                        }
                    }
                },
                _ => ()
            }
        });

        match eloop_result {
            Ok(_) => Ok(()),
            Err(_) => Err(WutEngineError::Unknown),
        }
    }
}
