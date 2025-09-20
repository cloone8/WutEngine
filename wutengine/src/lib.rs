//! The WutEngine game engine

use core::sync::atomic::{AtomicBool, Ordering};

use thiserror::Error;
use winit::event_loop::{ControlFlow, EventLoop};
use wutengine_windowing::WutEngineWinitEvent;

use crate::config::StaticRuntimeConfig;
use crate::winit_app::{WinitApp, WinitInitData};

pub mod builtin;
pub mod config;

#[doc(inline)]
pub use wutengine_asset as asset;

#[doc(inline)]
pub use wutengine_windowing::display;

#[doc(inline)]
pub use wutengine_windowing::window;

#[doc(inline)]
pub use wutengine_event as event;

#[doc(inline)]
pub use wutengine_graphics as graphics;

#[doc(inline)]
pub use wutengine_job as jobs;

#[doc(inline)]
pub use wutengine_time as time;

#[doc(inline)]
pub use wutengine_math as math;

pub mod component;
pub mod entity;
pub mod system;

pub mod prelude;
pub mod profiling;
mod runtime;
mod winit_app;

pub use hecs;

pub use wutengine_util::map;

/// An error while initializing the WutEngine runtime during a call to [run]
#[derive(Debug, Error)]
pub enum InitErr {
    /// WutEngine was already initialized
    #[error("WutEngine was already initialized in this process")]
    AlreadyInitialized,

    /// Error with graphics stack. Usually due to missing graphics devices or something
    /// similar
    #[error("Error initializing graphics stack: {0}")]
    Graphics(#[from] crate::graphics::InitErr),
}

/// The main entrypoint into WutEngine. Starts the runtime. Can only be called
/// once in a process, and should be called on the main thread only.
///
/// Once this function returns, the engine runtime has stopped and the process
/// should stop too
pub fn run(config: StaticRuntimeConfig) -> Result<(), InitErr> {
    static INITIALIZED: AtomicBool = AtomicBool::new(false);

    if INITIALIZED.swap(true, Ordering::AcqRel) {
        return Err(InitErr::AlreadyInitialized);
    }

    profiling::scope!("Engine Runtime Initialization");

    log::info!("Initializing WutEngine");

    // Configuration
    wutengine_config::init(config.config_file.as_deref());

    // Job system
    jobs::init();

    // Time manager
    time::init();

    // Event manager
    wutengine_event::init();

    // Graphics stack
    pollster::block_on(crate::graphics::init())?;

    // Runtime managers
    system::init();
    runtime::init();

    wutengine_asset::init(config.asset_loader, config.asset_format);

    // Finally we start Winit, which runs the actual window/event loop
    let event_loop = EventLoop::<WutEngineWinitEvent>::with_user_event()
        .build()
        .expect("Could not build winit EventLoop");

    event_loop.set_control_flow(ControlFlow::Poll);

    let mut winit_app = WinitApp::new(
        event_loop.create_proxy(),
        WinitInitData {
            initial_window: config.initial_window,
            post_init_callback: config.post_init,
        },
    );

    event_loop
        .run_app(&mut winit_app)
        .expect("Error in Winit event loop");

    Ok(())
}
