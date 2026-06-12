//! Low level thread pool initialization and functionality

use core::num::NonZero;
use core::sync::atomic::{AtomicBool, Ordering};
use std::thread::available_parallelism;

use serde::Deserialize;

use wutengine_util::assert_main_thread;

mod detect;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct ThreadConfig {
    /// The amount of worker threads to spawn. If zero, automatically
    /// picks the number of worker threads based on the available
    /// OS threads.
    worker_threads: usize,
}

/// The default amount of worker threads
const DEFAULT_NUM_THREADS: NonZero<usize> = NonZero::new(8).unwrap();

/// A flag containing whether the thread pool was initialized or not
static THREAD_POOL_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initializes the global thread pool.
pub(crate) fn init_thread_pool() {
    assert_main_thread!();

    if THREAD_POOL_INITIALIZED.swap(true, Ordering::AcqRel) {
        panic!("Thread pool already initialized");
    }

    let num_threads = determine_worker_thread_count();

    log::debug!("Using {num_threads} job worker threads");

    let builder = rayon::ThreadPoolBuilder::new();

    builder
        .num_threads(num_threads.get())
        .start_handler(thread_start_handler)
        .thread_name(make_thread_name)
        .build_global()
        .expect("A global thread pool has already been initialized")
}

fn determine_worker_thread_count() -> NonZero<usize> {
    let thread_config = crate::config::get::<ThreadConfig>("wutengine.thread");

    let configured_num_threads = NonZero::new(thread_config.worker_threads);

    if let Some(num_threads) = configured_num_threads {
        log::trace!("Using user configured worker thread count of {num_threads}");
        // If the user configured a thread count, use that
        return num_threads;
    }

    // Next, try to get the amount of performance CPU cores on the system.
    if let Some(core_config) = detect::try_detect_core_config() {
        log::debug!("Detected core configuration: {:#?}", core_config);

        if let Some(perf_cores) = core_config
            .threads_by_class
            .last()
            .and_then(|pc| NonZero::new(*pc))
        {
            log::trace!("Using detected performance core count of {perf_cores}");
            return perf_cores;
        }

        let num_threads = core_config.threads;

        log::trace!(
            "Using detected logical thread count of {} because the CPU core performance class could not be determined",
            num_threads
        );
        return num_threads;
    }

    // Finally, try to determine the thread count based on the CPU threads.
    match available_parallelism() {
        Ok(available) => available,
        Err(e) => {
            log::warn!(
                "Failed to determine available threads on this machine: {e}. Defaulting to {DEFAULT_NUM_THREADS}"
            );
            DEFAULT_NUM_THREADS
        }
    }
}

fn thread_start_handler(index: usize) {
    let _thread_name = make_thread_name(index); // Might be unused depending on the profiling backend
    profiling::register_thread!(_thread_name.as_str());
}

fn make_thread_name(index: usize) -> String {
    format!("wutengine_worker_{index}")
}
