//! Low level thread pool initialization and functionality

use core::num::NonZero;
use core::sync::atomic::{AtomicBool, Ordering};
use std::thread::available_parallelism;

use serde::Deserialize;

use crate::util::assert_main_thread;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct ThreadConfig {
    /// The amount of worker threads to spawn. If zero, automatically
    /// picks the number of worker threads based on the available
    /// OS threads.
    worker_threads: usize,
}

/// The default amount of worker threads
const DEFAULT_NUM_THREADS: usize = 8;

/// A flag containing whether the thread pool was initialized or not
static THREAD_POOL_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initializes the global thread pool.
pub(crate) fn init_thread_pool() {
    assert_main_thread!();

    if THREAD_POOL_INITIALIZED.swap(true, Ordering::AcqRel) {
        panic!("Thread pool already initialized");
    }

    let thread_config = crate::config::get::<ThreadConfig>("wutengine.thread");

    let configured_num_threads = NonZero::new(thread_config.worker_threads);

    let num_threads = match configured_num_threads {
        Some(num_threads) => num_threads.get(), // If the user configured a thread count, use that
        None => match available_parallelism() {
            // If not, try to determine the thread count based on the CPU threads.
            Ok(available) => available.get(),
            Err(e) => {
                log::warn!(
                    "Failed to determine available threads on this machine: {e}. Defaulting to {DEFAULT_NUM_THREADS}"
                );
                DEFAULT_NUM_THREADS
            }
        },
    };

    log::debug!("Using {num_threads} job worker threads");

    let builder = rayon::ThreadPoolBuilder::new();

    builder
        .num_threads(num_threads)
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
