//! Engine runtime lifecycle

use core::panic::Location;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;
use core::time::Duration;
use std::panic;

use wutengine_sync::thread_context;

use crate::log;
use crate::panic_handling;
use crate::platform;
use crate::runtime::driver::headless::HeadlessApp;
use crate::time;
use crate::window;

mod driver;
mod frame;
pub mod hooks;

static HEADLESS: AtomicBool = AtomicBool::new(false);

/// Whether the runtime is running in headless mode
#[inline]
#[must_use]
pub(crate) fn is_headless() -> bool {
    cfg!(not(feature = "interactive")) || HEADLESS.load(Ordering::Relaxed)
}

/// Startup options used to start the [WutEngine runtime](crate::runtime)
/// Not public API, should be used through the [`wutengine_macro::main`] macro
#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub struct StartupOptions {
    /// Organization name, used for storing/loading data
    pub org_name: &'static str,

    /// Application name, used for storing/loading data
    pub app_name: &'static str,

    /// Whether the application should be run in headless mode, even if the
    /// build is an interactive one. If the build is a headless build, this is
    /// ignored
    pub headless: bool,
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

    if options.headless {
        HEADLESS.store(true, Ordering::Relaxed);
    }

    // Run the app entrypoint first, so that users can have chance to run registrations
    app_entrypoint();

    // Start the window manager
    window::init();

    time::set_fixed_delta(Duration::from_secs_f64(1.0 / 60.0).as_nanos() as u64); // TODO: Make configurable

    #[cfg(feature = "interactive")]
    if !is_headless() {
        run_winit();
        return;
    }

    run_headless();
}

#[cfg(feature = "interactive")]
fn run_winit() {
    use winit::event_loop::EventLoop;

    use crate::runtime::driver::winit::WutEngineApp;

    let event_loop = match EventLoop::builder().build() {
        Ok(el) => el,
        Err(e) => {
            fatal_error(format!("Failed to build an event loop: {e}").as_str());
        }
    };

    if let Err(e) = event_loop.run_app(WutEngineApp::new()) {
        fatal_error(format!("Error while running app event loop: {e}").as_str());
    }
}

fn run_headless() -> ! {
    log::info!("Starting application in headless mode");

    let mut app = HeadlessApp::new(60.0); //TODO: Make configurable

    app.run();
}

/// Flag set when [`quit`] was called. Should not be reversed
static SHOULD_QUIT: AtomicBool = AtomicBool::new(false);

/// Returns whether a quit was requested with [`quit`]
pub(crate) fn should_quit() -> bool {
    SHOULD_QUIT.load(Ordering::Acquire)
}

/// Requests that the WutEngine runtime quits as soon as possible.
/// This call might still return. Duplicate calls are ignored, as the first
/// call is already final.
///
/// This call always overrides a quit _request_. If called from a `quit requested` handler,
/// for example, the app _will_ quit as soon as possible, even if the quit request was cancelled.
pub fn quit() {
    if !SHOULD_QUIT.swap(true, Ordering::AcqRel) {
        log::info!("Quit scheduled");
    }

    //TODO: This should probably wake up the driver loops
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
