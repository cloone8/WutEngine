#![doc = include_str!("../../README.md")]

use std::thread::ThreadId;

use util::InitOnce;

pub mod component;
pub mod entity;
pub mod graphics;
pub mod runtime;
pub(crate) mod util;
pub mod window;
pub mod world;

pub use log;
pub use wgpu;

/// The ID of the main thread, used by [on_main_thread] and initialized right before the runtime
static MAIN_THREAD_ID: InitOnce<ThreadId> = InitOnce::new();

/// Returns whether the calling thread is the main thread.
///
/// The main thread is the thread that called [runtime::run]
pub(crate) fn on_main_thread() -> bool {
    std::thread::current().id() == *MAIN_THREAD_ID
}

/// Panics if the current thread is not the main thread
macro_rules! assert_main_thread {
    ($context_name:literal) => {
        if !crate::on_main_thread() {
            panic!("'{}' must be run on the main thread!", $context_name);
        }
    };
}

pub(crate) use assert_main_thread;
