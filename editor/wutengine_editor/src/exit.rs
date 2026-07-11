//! Exit interruptions and handlers

use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;

use crate::project;

static EXIT_REQUESTED: AtomicBool = AtomicBool::new(false);
static EXIT_ALLOWED: AtomicBool = AtomicBool::new(false);

/// Handler for [wutengine::runtime::add_on_exit_requested_handler]
pub(crate) fn on_exit_requested_handler() -> bool {
    if EXIT_ALLOWED.load(Ordering::Acquire) {
        false
    } else {
        EXIT_REQUESTED.store(true, Ordering::Release);
        true
    }
}

/// Handler for [wutengine::runtime::add_on_exit_handler]
pub(crate) fn on_exit_handler() {
    if let Err(e) = project::save() {
        log::error!("Failed to save project to disk: {e}");
    }
}

/// Returns whether an editor exit was requested
pub(crate) fn exit_requested() -> bool {
    EXIT_REQUESTED.load(Ordering::Acquire)
}

/// Returns stops the requested editor exit
pub(crate) fn stop_exit() {
    EXIT_REQUESTED.store(false, Ordering::Release);
}

/// Allows the requested editor exit to proceed, which will shut down the editor
pub(crate) fn allow_exit() {
    EXIT_ALLOWED.store(true, Ordering::Release);
    wutengine::runtime::exit(); // Request exit again, but do not interrupt it this time
}
