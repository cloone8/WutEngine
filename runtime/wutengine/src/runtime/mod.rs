//! Engine runtime lifecycle

use core::panic::Location;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;
use std::panic;

use winit::event_loop::EventLoop;
use wutengine_sync::thread_context;

use crate::log;
use crate::panic_handling;
use crate::platform;
use crate::runtime::winit_app::WutEngineApp;

mod frame;
mod startup;
mod winit_app;

/// Startup options used to start the [WutEngine runtime](crate::runtime)
/// Not public API, should be used through the [`wutengine_macro::main`] macro
#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub struct StartupOptions {
    /// Organization name, used for storing/loading data
    pub org_name: &'static str,

    /// Application name, used for storing/loading data
    pub app_name: &'static str,
}

/// Entry point of the WutEngine runtime. Must be called once, from the main thread.
/// Not public API, should be used through the [`wutengine_macro::main`] macro.
///
/// On native, this runs the engine and never returns. On the web, it stores the arguments and
/// returns: the JavaScript-side loader starts the engine afterwards
#[doc(hidden)]
pub fn _wutengine_entry(
    options: StartupOptions,
    app_entrypoint: impl FnOnce() + 'static,
) -> platform::StartRet {
    platform::main(options, app_entrypoint);
}

/// Runs the engine until its event loop exits. Called by the platform once it is ready to start
pub(crate) fn run(options: &StartupOptions, app_entrypoint: impl FnOnce() + 'static) {
    static INIT: AtomicBool = AtomicBool::new(false);

    if INIT.swap(true, Ordering::Relaxed) {
        panic!("Runtime already initialized");
    }

    thread_context::mark_main_thread();

    panic::set_hook(Box::new(panic_handling::panic_hook));

    log::init(
        log::LogSettings::with_default_categories(),
        vec![platform::base_log_output()],
    );

    platform::add_log_outputs(options);

    // Run the app entrypoint first, so that users can have chance to run registrations
    app_entrypoint();

    let event_loop = match EventLoop::builder().build() {
        Ok(el) => el,
        Err(e) => {
            panic!("Failed to build an event loop: {e}"); //TODO: Replace with fatal error once we have an API for it
        }
    };

    if let Err(e) = event_loop.run_app(WutEngineApp::new()) {
        panic!("Error while running app event loop: {e}"); //TODO: Replace with fatal error later
    }
}

/// Requests that the WutEngine runtime quits as soon as possible.
/// This call might still return
pub fn quit() {
    //TODO: This will probably be handled more neatly by the runtime later, with hooks and stuff
    exit_now(0);
}

/// Immediatly quits the WutEngine runtime with an error message and exit code `100`
#[track_caller]
pub fn fatal_error(message: &str) -> ! {
    let location = Location::caller();
    log::error!("FATAL ERROR @ {location}: {message}");

    exit_now(100); // 100 chosen arbitrarily
}

/// Exit the application with the given exit code
pub(crate) fn exit_now(code: i32) -> ! {
    log::info!("Quitting runtime with exit code {code}");
    log::flush();

    platform::exit(code);
}
