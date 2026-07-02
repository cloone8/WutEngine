//! Public runtime API functions

use alloc::sync::Arc;
use core::sync::atomic::Ordering;

use super::AddOnExitHandler;
use super::AddOnExitRequestedHandler;
use super::MainThreadEvent;
use super::WUTENGINE_RUNNING;

/// Requests that the WutEngine runtime stops cleanly.
/// This usually happens somewhere before the next frame.
///
/// The on-exit handlers will be run, and will have a chance to stop the exit process
pub fn exit() {
    request_exit(false);
}

/// Instructs the WutEngine runtime to stop cleanly.
/// This usually happens somewhere before the next frame.
///
/// The on-exit handlers will be run, but will not be able to stop the exit process.
pub fn force_exit() {
    request_exit(true);
}

/// Internal. Use [exit] or [force_exit] instead.
#[inline]
fn request_exit(force: bool) {
    if !WUTENGINE_RUNNING.load(Ordering::Acquire) {
        log::error!("WutEngine runtime is not running. Cannot request exit");
        return;
    }

    if force {
        log::info!("Runtime exit forced");
    } else {
        log::info!("Runtime exit requested");
    }

    crate::runtime::send_to_main_thread(MainThreadEvent::RuntimeExitRequested(force));
}

/// Useful when a frequency setting other than [crate::runtime::FrameFrequency::Fast] was selected
pub fn request_frame() {
    crate::runtime::send_to_main_thread(MainThreadEvent::Wake);
}

/// Run a future on the main thread
pub fn run_on_main_thread<F, T>(task: F) -> wutengine_thread::TaskHandle<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let (handle, future) = wutengine_thread::TaskHandle::from_future(task);

    crate::runtime::send_to_main_thread(MainThreadEvent::RunTask(future));

    handle
}

/// Adds an application on-exit handler
pub fn add_on_exit_handler(handler: impl Fn() + Send + Sync + 'static) {
    crate::event::publish(AddOnExitHandler(Arc::new(handler)));
}

/// Adds an application on-exit-requested handler
pub fn add_on_exit_requested_handler(handler: impl Fn() -> bool + Send + Sync + 'static) {
    crate::event::publish(AddOnExitRequestedHandler(Arc::new(handler)));
}
