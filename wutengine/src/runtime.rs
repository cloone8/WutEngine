//! The main WutEngine runtime, responsible for the application lifecycle

use core::sync::atomic::{AtomicBool, Ordering};

use derive_more::{Display, Error, From};
use winit::error::EventLoopError;
use world::World;

mod winit_app;
pub(crate) mod world;

pub(crate) struct InitializationData {
    post_start_callback: Option<Box<dyn FnOnce()>>,
}

/// The main WutEngine runtime
pub(crate) struct Runtime {
    /// Set to `true` if the `resumed` event was sent by [winit]
    initialization_data: Option<InitializationData>,

    /// The complete set of entities and components
    pub(crate) world: World,
}

/// An error while starting the WutEngine runtime with [start]
#[derive(Debug, Error, Display, From)]
pub enum RuntimeStartErr {
    /// Another WutEngine runtime was already started in this process
    #[display("Another WutEngine runtime was already started in this process")]
    AlreadyRunning,

    /// Error running the winit event loop
    #[display("Error running the winit event loop: {_0}")]
    EventLoop(EventLoopError),
}

/// Starts and runs the WutEngine runtime.
///
/// Can only be called once per process
pub fn run(post_start: Option<Box<dyn FnOnce()>>) -> Result<(), RuntimeStartErr> {
    static WUTENGINE_RUNNING: AtomicBool = AtomicBool::new(false);

    //TODO: Check if this is called from the main thread

    if WUTENGINE_RUNNING.swap(true, Ordering::AcqRel) {
        return Err(RuntimeStartErr::AlreadyRunning);
    }

    log::info!("Starting WutEngine");

    let mut runtime = Runtime {
        initialization_data: Some(InitializationData {
            post_start_callback: post_start,
        }),
        world: World::new(),
    };

    let event_loop = winit::event_loop::EventLoop::new()?;
    let event_loop_proxy = event_loop.create_proxy();

    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    event_loop.run_app(&mut runtime)?;

    Ok(())
}
