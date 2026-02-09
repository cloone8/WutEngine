//! Low level thread pool initialization and functionality

use core::sync::atomic::{AtomicBool, Ordering};
use std::thread::available_parallelism;

use crate::util::assert_main_thread;

/// A flag containing whether the thread pool was initialized or not
static THREAD_POOL_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initializes the global thread pool.
pub(crate) fn init_thread_pool() {
    assert_main_thread!();

    if THREAD_POOL_INITIALIZED.swap(true, Ordering::AcqRel) {
        panic!("Thread pool already initialized");
    }

    let available_paralellism = match available_parallelism() {
        Ok(available) => available.get(),
        Err(e) => {
            const DEFAULT_NUM_THREADS: usize = 8;

            log::warn!(
                "Failed to determine available threads on this machine: {e}. Defaulting to {DEFAULT_NUM_THREADS}"
            );
            DEFAULT_NUM_THREADS
        }
    };

    log::debug!("Using {available_paralellism} job worker threads");

    let builder = rayon::ThreadPoolBuilder::new();

    builder
        .num_threads(available_paralellism)
        .start_handler(thread_start_handler)
        .thread_name(make_thread_name)
        .build_global()
        .expect("A global thread pool has already been initialized")
}

/// Checks whether the global thread pool was initialized
pub(crate) fn thread_pool_initialized() -> bool {
    THREAD_POOL_INITIALIZED.load(Ordering::Acquire)
}

fn thread_start_handler(index: usize) {
    let _thread_name = make_thread_name(index); // Might be unused depending on the profiling backend
    profiling::register_thread!(_thread_name.as_str());
}

fn make_thread_name(index: usize) -> String {
    format!("wutengine_worker_{index}")
}
