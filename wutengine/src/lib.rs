//! The WutEngine game engine

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicBool, Ordering};

use thiserror::Error;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowAttributes;
use wutengine_windowing::WutEngineWinitEvent;

use crate::config::WutEngineConfig;
use crate::winit_app::{WinitApp, WinitInitData};

pub mod config;
pub use wutengine_windowing::display;
pub use wutengine_windowing::window;
pub mod asset;
pub mod builtin;
pub mod component;
pub mod gameobject;
pub mod graphics;
pub mod math;
pub mod prelude;
pub mod profiling;
mod runtime;
mod threading;
pub mod time;
mod winit_app;

pub use wutengine_util::map;

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

pub fn run(config: WutEngineConfig) -> Result<(), InitErr> {
    static INITIALIZED: AtomicBool = AtomicBool::new(false);

    if INITIALIZED.swap(true, Ordering::AcqRel) {
        return Err(InitErr::AlreadyInitialized);
    }

    log::info!("Initializing WutEngine");

    // Rayon worker threads
    threading::init_threadpool();

    // Graphics stack
    pollster::block_on(crate::graphics::init(config.backends))?;

    // Time management
    unsafe {
        time::init(config.fixed_timestep);
    }

    // GameObject and Component managers
    gameobject::init();
    component::init();
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
