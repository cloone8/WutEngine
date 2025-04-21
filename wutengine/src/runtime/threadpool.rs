//! Threadpool initialization and functionality

/// Initializes the global thread pool
pub(crate) fn init_threadpool() {
    let num_available_threads = match std::thread::available_parallelism() {
        Ok(n) => n,
        Err(e) => {
            log::error!("Could not determine available paralellism: {}", e);
            log::error!("Defaulting to 4 threads");

            core::num::NonZero::new(4).unwrap()
        }
    };

    log::debug!(
        "Initializing threadpool with {} threads",
        num_available_threads
    );

    rayon::ThreadPoolBuilder::new()
        .num_threads(num_available_threads.get())
        .start_handler(thread_start_handler)
        .thread_name(make_thread_name)
        .build_global()
        .expect("Could not initialize thread pool");
}

fn thread_start_handler(_index: usize) {
    profiling::register_thread!();
}

fn make_thread_name(index: usize) -> String {
    format!("wutengine_worker_{}", index)
}
