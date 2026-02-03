//! The main WutEngine runtime, responsible for the application lifecycle

use core::sync::atomic::{AtomicBool, Ordering};

use derive_more::{Display, Error, From};
use winit::error::EventLoopError;

use crate::entity::{self, EntityManager};
use crate::system::{self, SystemManager};
use crate::util::InitOnce;
use crate::window::{self};
use crate::world;

mod system_builder;
mod winit_app;

pub use system_builder::*;

pub(crate) use winit_app::WinitEvent;

static EVENT_LOOP_PROXY: InitOnce<winit::event_loop::EventLoopProxy<WinitEvent>> = InitOnce::new();

/// Notifies the main [winit] event loop of a given event.
///
/// If the loop was already closed, does nothing and logs an error
pub(crate) fn notify_event_loop(event: WinitEvent) {
    if let Err(e) = EVENT_LOOP_PROXY.send_event(event) {
        log::error!(
            "Failed to notify event loop of event {:#?} because it was already closed",
            e.0
        );
    }
}

/// Data only relevant before/during application initialization in [winit::application::ApplicationHandler::resumed]
pub(crate) struct InitializationData {
    post_start_callback: Option<Box<dyn FnOnce()>>,
}

/// The main WutEngine runtime
pub(crate) struct Runtime {
    /// Set to `true` if the `resumed` event was sent by [winit]
    initialization_data: Option<Box<InitializationData>>,

    /// The entity manager. Spawns entities and components
    entity_manager: EntityManager,

    systems: SystemManager,
}

/// An error while starting the WutEngine runtime with [run]
#[derive(Debug, Error, Display, From)]
pub enum RuntimeStartErr {
    /// Another WutEngine runtime was already started in this process
    #[display("Another WutEngine runtime was already started in this process")]
    AlreadyRunning,

    /// Error running the winit event loop
    #[display("Error running the winit event loop: {_0}")]
    EventLoop(EventLoopError),
}

/// Starts and runs the WutEngine runtime. MUST be called from the main thread
///
/// Can only be called once per process
pub fn run(
    systems: SystemManifest,
    post_start: Option<Box<dyn FnOnce()>>,
) -> Result<(), RuntimeStartErr> {
    static WUTENGINE_RUNNING: AtomicBool = AtomicBool::new(false);

    if WUTENGINE_RUNNING.swap(true, Ordering::AcqRel) {
        return Err(RuntimeStartErr::AlreadyRunning);
    }

    log::info!("Starting WutEngine");

    InitOnce::init(&crate::MAIN_THREAD_ID, std::thread::current().id());

    let mut runtime = Runtime {
        initialization_data: Some(Box::new(InitializationData {
            post_start_callback: post_start,
        })),
        entity_manager: entity::initialize(),
        systems: system::SystemManager::new(),
    };

    runtime.systems.build_schedule(systems);

    log::debug!("Final schedule:\n{}", runtime.systems.dump());

    window::manager::initialize();
    world::initialize();

    let event_loop = winit::event_loop::EventLoop::<WinitEvent>::with_user_event().build()?;
    let event_loop_proxy = event_loop.create_proxy();

    InitOnce::init(&EVENT_LOOP_PROXY, event_loop_proxy);

    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    event_loop.run_app(&mut runtime)?;

    Ok(())
}
