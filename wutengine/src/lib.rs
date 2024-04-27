pub use wutengine_core as core;

use core::Scene;
use std::{collections::HashMap, marker::PhantomData, time::Instant};

use thiserror::Error;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
    window::{Window, WindowBuilder, WindowId},
};

pub enum WutEngineWindowEvent {

}

pub struct HeadlessBackend;

impl Default for HeadlessBackend {
    fn default() -> Self {
        HeadlessBackend {}
    }
}
impl WutEngineRenderer for HeadlessBackend {
    fn init(&mut self) {
        log::info!("Initialized headless backend") 
    }

    fn render(&mut self, engine: &WutEngine) {
        log::info!("Rendering engine state");
    }
}

trait WutEngineStarterState {}

pub struct Building<B: WutEngineRenderer> {
    windows_to_open: usize,
    backend: PhantomData<B>
}

impl<B: WutEngineRenderer> WutEngineStarterState for Building<B> {}

pub struct Uninitialized<B: WutEngineRenderer> {
    window_builders: Vec<WindowBuilder>,
    backend: B
}

impl<B: WutEngineRenderer> WutEngineStarterState for Uninitialized<B> {}

pub trait WutEngineRenderer: Default {
    fn init(&mut self);
    fn render(&mut self, engine: &WutEngine);
}

pub struct Initialized<B: WutEngineRenderer> {
    event_loop: EventLoop<WutEngineWindowEvent>,
    windows: HashMap<WindowId, Window>,
    backend: B
}

impl<B: WutEngineRenderer> WutEngineStarterState for Initialized<B> {}


#[derive(Debug)]
pub struct WutEngineStarter<S: WutEngineStarterState = Building<HeadlessBackend>> {
    state: S
}

impl WutEngineStarter {
    pub fn new() -> WutEngineStarter<Building<HeadlessBackend>> {
        WutEngineStarter::<Building<HeadlessBackend>>::default() 
    }
}

impl Default for WutEngineStarter<Building<HeadlessBackend>> {
    fn default() -> Self {
        WutEngineStarter {
            state: Building {
                windows_to_open: 1,
                backend: PhantomData::default()
            }
        }
    }
}

impl<B: WutEngineRenderer> WutEngineStarter<Building<B>> {
    pub fn with_initial_windows(self, num: usize) -> Self {
        WutEngineStarter {
            state: Building {
                windows_to_open: num,
                backend: self.state.backend
            }
        } 
    }

    pub fn with_backend<T: WutEngineRenderer>(self) -> WutEngineStarter<Building<T>> {
        WutEngineStarter {
            state: Building {
                windows_to_open: self.state.windows_to_open,
                backend: PhantomData::default()
            }
        }
    }

    pub fn build(self) -> WutEngineStarter<Uninitialized<B>> {
        let winbuilders: Vec<_> = (0..self.state.windows_to_open)
            .map(|i| WindowBuilder::new().with_title(format!("WutEngine - Window {}", i)))
            .collect();

        WutEngineStarter {
            state: Uninitialized {
                window_builders: winbuilders,
                backend: B::default()
            }
        }
    }
}

impl<B: WutEngineRenderer> WutEngineStarter<Uninitialized<B>> {
    pub fn init(self) -> WutEngineStarter<Initialized<B>> {
        let event_loop = EventLoopBuilder::<WutEngineWindowEvent>::with_user_event().build().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);

        let windows = self.state.window_builders.into_iter()
            .map(|builder| builder.build(&event_loop).unwrap())
            .map(|window| (window.id(), window))
            .collect();
        
        let mut backend = self.state.backend;
        backend.init();

        WutEngineStarter {
            state: Initialized {
                event_loop,
                windows,
                backend
            }
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

#[derive(Debug)]
struct WutEngine {
    start_time: Instant,
    prev_frame_start: Instant
}

impl WutEngine {
    fn frame(&mut self) {
        let now = Instant::now();
        let delta_time = now.duration_since(self.prev_frame_start).as_secs_f32();

        self.prev_frame_start = now;

        log::trace!("deltatime: {}, FPS: {}", delta_time, 1_f32 / delta_time);
    }
}

impl<B: WutEngineRenderer> WutEngineStarter<Initialized<B>> {
    pub fn run(self, initial_scene: Scene) {
        let event_loop = self.state.event_loop;
        let windows = self.state.windows;
        let mut backend = self.state.backend;
        
        let start_time = Instant::now();
        let mut engine = WutEngine {
            start_time,
            prev_frame_start: start_time
        };

        let eloop_result = event_loop.run(move |event, elwt| {
            match event {
                Event::AboutToWait => {
                    engine.frame();
                    backend.render(&engine);
                },  
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

        eloop_result.unwrap();
    } 
}

