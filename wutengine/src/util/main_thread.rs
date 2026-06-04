use super::InitOnce;
use core::sync::atomic::{AtomicBool, Ordering};
use std::thread::ThreadId;

/// The ID of the main thread, used by [on_main_thread] and initialized right before the runtime
static MAIN_THREAD_ID: InitOnce<ThreadId, false> = InitOnce::new();

/// Sets the current thread as the "main thread", for use in later checks like [on_main_thread] and [assert_main_thread]
pub(crate) fn set_cur_thread_as_main_thread() {
    static ALREADY_SET: AtomicBool = AtomicBool::new(false);

    if ALREADY_SET.swap(true, Ordering::AcqRel) {
        panic!("Main thread already set. Engine internal error");
    }

    InitOnce::init(&MAIN_THREAD_ID, std::thread::current().id());
}

/// Returns whether the calling thread is the main thread.
///
/// The main thread is the thread that called [crate::runtime::run]
#[inline(always)]
pub(crate) fn on_main_thread() -> bool {
    std::thread::current().id() == *MAIN_THREAD_ID
}

/// Panics if the current thread is not the main thread
macro_rules! assert_main_thread {
    () => {
        if !$crate::util::on_main_thread() {
            let func_name = $crate::util::current_function_name!();
            panic!("'{func_name}' must be run on the main thread!");
        }
    };
}

pub(crate) use assert_main_thread;
