#![doc = include_str!("../README.md")]

extern crate alloc;

use alloc::sync::Arc;
use core::num::NonZero;
use core::sync::atomic::{AtomicBool, Ordering};
use detect::CoreConfig;
use std::thread::available_parallelism;

use serde::Deserialize;

use wutengine_util::InitOnce;
use wutengine_util::assert_main_thread;

mod detect;

/// User thread configuration
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct ThreadConfig {
    /// The amount of worker threads to spawn. If zero, automatically
    /// picks the number of worker threads based on the available
    /// OS threads.
    worker_threads: usize,

    /// The amount of background threads to spawn. If zero, picks the default
    background_threads: usize,
}

/// The default amount of worker threads
const DEFAULT_WORKER_THREADS: NonZero<usize> = NonZero::new(8).unwrap();

/// The default amount of background threads
const DEFAULT_BACKGROUND_THREADS: NonZero<usize> = DEFAULT_WORKER_THREADS
    .checked_mul(NonZero::new(2).unwrap())
    .unwrap();

/// A flag containing whether the thread pool was initialized or not
static THREAD_POOL_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// The global async thread pool
static ASYNC_POOL: InitOnce<futures::executor::ThreadPool> = InitOnce::new();

/// Initializes the global thread pool.
pub fn init_thread_pool() {
    assert_main_thread!();

    if THREAD_POOL_INITIALIZED.swap(true, Ordering::AcqRel) {
        panic!("Thread pool already initialized");
    }

    let thread_config = wutengine_config::get::<ThreadConfig>("wutengine.thread");
    let cpu_config = detect::try_detect_core_config();

    init_worker_threads(&thread_config, cpu_config.as_ref());
    init_async_threads(&thread_config, cpu_config.as_ref());
}

/// Initialize the background async thread pool
fn init_async_threads(config: &ThreadConfig, cpu_config: Option<&CoreConfig>) {
    let num_threads = determine_async_thread_count(config, cpu_config);

    log::info!("Using {num_threads} async background threads");

    let thread_pool = futures::executor::ThreadPoolBuilder::new()
        .pool_size(num_threads.get())
        .name_prefix("wutengine_background_")
        .after_start(|idx| {
            let _thread_name = format!("wutengine_background_{idx}");
            profiling::register_thread!(_thread_name.as_str());
        })
        .create()
        .expect("Failed to initialize background thread pool");

    InitOnce::init(&ASYNC_POOL, thread_pool);
}

/// Determines the amount of background thread
fn determine_async_thread_count(
    config: &ThreadConfig,
    cpu_config: Option<&CoreConfig>,
) -> NonZero<usize> {
    if let Some(num_threads) = NonZero::new(config.background_threads) {
        log::debug!(
            "Using user configured background thread count of {}",
            num_threads
        );
        return num_threads;
    }

    if let Some(cpu_config) = cpu_config {
        log::debug!(
            "Using logical core count ({}) to determine background thread count",
            cpu_config.threads
        );

        return cpu_config
            .threads
            .checked_mul(NonZero::new(2).unwrap())
            .unwrap();
    }

    log::warn!(
        "Failed to determine async thread count, using default number of background threads ({})",
        DEFAULT_BACKGROUND_THREADS
    );

    DEFAULT_BACKGROUND_THREADS
}

/// Initialize the main worker thread pool
fn init_worker_threads(config: &ThreadConfig, cpu_config: Option<&CoreConfig>) {
    let num_threads = determine_worker_thread_count(config, cpu_config);

    log::info!("Using {num_threads} job worker threads");

    let builder = rayon::ThreadPoolBuilder::new();

    builder
        .num_threads(num_threads.get())
        .start_handler(thread_start_handler)
        .thread_name(make_thread_name)
        .build_global()
        .expect("A global thread pool has already been initialized")
}

/// Determines the amount of worker threads based on factors such as user config
/// and core count of the current machine
fn determine_worker_thread_count(
    config: &ThreadConfig,
    cpu_config: Option<&CoreConfig>,
) -> NonZero<usize> {
    let configured_num_threads = NonZero::new(config.worker_threads);

    if let Some(num_threads) = configured_num_threads {
        log::debug!("Using user configured worker thread count of {num_threads}");
        // If the user configured a thread count, use that
        return num_threads;
    }

    // Next, try to get the amount of performance CPU cores on the system.
    if let Some(cpu_config) = cpu_config {
        log::debug!("Detected core configuration: {:#?}", cpu_config);

        if let Some(perf_cores) = cpu_config
            .threads_by_class
            .last()
            .and_then(|pc| NonZero::new(*pc))
        {
            log::trace!("Using detected performance core count of {perf_cores}");
            return perf_cores;
        }

        let num_threads = cpu_config.threads;

        log::debug!(
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
                "Failed to determine available threads on this machine: {e}. Defaulting to {DEFAULT_WORKER_THREADS}"
            );
            DEFAULT_WORKER_THREADS
        }
    }
}

/// Rayon thread start handler
fn thread_start_handler(index: usize) {
    let _thread_name = make_thread_name(index); // Might be unused depending on the profiling backend
    profiling::register_thread!(_thread_name.as_str());
}

/// Rayon thread name handler
fn make_thread_name(index: usize) -> String {
    format!("wutengine_worker_{index}")
}

/// Handle to an async task, spawned with [run_async].
#[derive(Debug)]
pub struct TaskHandle<T> {
    /// Whether the task is done
    done: Arc<AtomicBool>,

    /// Receiver of the task output
    recv: futures::channel::oneshot::Receiver<T>,
}

impl<T> TaskHandle<T> {
    /// Returns `true` if the task is done and has produced its output (if any)
    #[inline]
    pub fn ready(&self) -> bool {
        self.done.load(Ordering::Acquire)
    }

    /// Returns the output of the async task. Blocks the current thread until done.
    /// To first check whether the results are ready, see [Self::ready]
    #[inline]
    pub fn get(self) -> T {
        futures::executor::block_on(self.recv).expect("Async task destroyed")
    }

    /// Utility function that checks if an optional task was started and is ready.
    #[inline(always)]
    pub fn started_and_ready(task: &Option<Self>) -> bool {
        let Some(task) = task else {
            return false;
        };

        task.ready()
    }

    /// Utility function that returns the result of an optional task, if it was started
    /// and is now ready. Leaves the task empty if it was ready
    #[inline(always)]
    pub fn get_if_started_and_ready(task: &mut Option<Self>) -> Option<T> {
        if !Self::started_and_ready(task) {
            return None;
        }

        task.take().map(Self::get)
    }
}

/// Spawns an async task on the background thread pool.
pub fn run_async<F, O>(task: F) -> TaskHandle<O>
where
    F: Future<Output = O> + Send + 'static,
    O: Send + 'static,
{
    let (send, recv) = futures::channel::oneshot::channel::<O>();
    let done = Arc::new(AtomicBool::new(false));

    {
        let done = done.clone();
        ASYNC_POOL.spawn_ok(async move {
            let ret = task.await;

            _ = send.send(ret);
            done.store(true, Ordering::Release);
        });
    }

    TaskHandle { done, recv }
}
