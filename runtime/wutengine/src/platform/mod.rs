//! Platform-specific parts of the engine.
//!
//! The functions in this module are the API every platform provides; each one forwards to the
//! implementation for the platform being built. The rest of the engine calls these, so it needs no
//! `#[cfg]` of its own. Items that only exist on one platform stay in that platform's module.

use winit::window::WindowAttributes;
use wutengine_log::LoggerOutput;

use crate::runtime::StartupOptions;
use crate::window::Window;

cfg_select! {
    target_arch = "wasm32" => {
        mod web;
        use web as imp;
    }
    _ => {
        mod native;
        use native as imp;
    }
}

/// Return type of the engine entry point, [`main`]. `!` on platforms where it runs the engine
/// itself, `()` where it returns and the engine is started later
pub(crate) type StartRet = imp::StartRet;

/// Platform entry point, called from the game's `main` through
/// [`_wutengine_entry`](crate::runtime::_wutengine_entry). Starts the engine, or prepares it to be
/// started by the platform later
#[inline]
pub(crate) fn main(options: StartupOptions, app_entrypoint: impl FnOnce() + 'static) -> StartRet {
    imp::main(options, app_entrypoint);
}

/// The log output that is available from the very start of the engine
#[inline]
pub(crate) fn base_log_output() -> Box<dyn LoggerOutput> {
    imp::base_log_output()
}

/// Adds the log outputs that need the logger to be running already, so that failures to create
/// them can be logged
#[inline]
pub(crate) fn add_log_outputs(options: &StartupOptions) {
    imp::add_log_outputs(options);
}

/// Adds the platform-specific window attributes to the platform-independent `attributes` of
/// `window`
#[inline]
pub(crate) fn finish_window_attributes(
    window: Window,
    attributes: WindowAttributes,
) -> WindowAttributes {
    imp::finish_window_attributes(window, attributes)
}

/// Exits the application with the given exit code
#[inline]
pub(crate) fn exit(code: i32) -> ! {
    imp::exit(code)
}
