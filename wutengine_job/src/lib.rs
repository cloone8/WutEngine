//! Job system for WutEngine

use core::fmt::Display;
use core::num::NonZero;
use core::ops::Deref;
use core::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, sync_channel};

use serde::Deserialize;

/// Job system and general runtime threading configuration
#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct JobSystemConfig {
    /// The amount of worker threads to spawn. If `0`, automatically tries
    /// to determine the amount of worker threads by looking at the available system
    /// parallelism with [std::thread::available_parallelism]
    num_threads: usize,
}

/// Global Job ID counter. Automatically incremented every time a new job is started
static NEXT_JOB_ID: AtomicU64 = AtomicU64::new(0);

/// Initializes the job system, and the [rayon] thread pool
#[doc(hidden)]
pub fn init() {
    let config = wutengine_config::get_wutengine::<JobSystemConfig>("jobs");

    let num_threads = match config.num_threads {
        0 => match std::thread::available_parallelism() {
            Ok(n) => n,
            Err(e) => {
                log::error!(
                    "Could not determine available paralellism: {e}\nDefaulting to 4 threads"
                );

                core::num::NonZero::new(4).unwrap()
            }
        },
        other => NonZero::new(other).unwrap(),
    };

    init_threadpool(num_threads);
}

/// Initializes the global thread pool
fn init_threadpool(threads: NonZero<usize>) {
    log::debug!("Initializing threadpool with {threads} threads");

    rayon::ThreadPoolBuilder::new()
        .num_threads(threads.get())
        .start_handler(thread_start_handler)
        .thread_name(make_thread_name)
        .build_global()
        .expect("Could not initialize thread pool");
}

/// Thread on-start handler used by [rayon]
fn thread_start_handler(_index: usize) {
    profiling::register_thread!();
}

/// Thread name handler used by [rayon]
fn make_thread_name(index: usize) -> String {
    format!("wutengine_worker_{index}")
}

/// Starts a job on one of the WutEngine worker threads, and returns a handle to that job.
/// The handle can be used to either block on the completion of the started job,
/// or to non-blockingly check whether it is done.
pub fn start_job<F, T>(job: F) -> JobHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
{
    let my_job_id = NEXT_JOB_ID.fetch_add(1, Ordering::Relaxed);

    let (send, recv) = sync_channel(1);

    let handle = JobHandle {
        job_id: my_job_id,
        inner: Mutex::new(JobHandleInner::Waiting(recv)),
    };

    rayon::spawn_fifo(move || {
        log::trace!("Starting job with id {my_job_id:x}");

        let job_result = job();

        log::trace!("Job {my_job_id:x} done, sending result");

        _ = send.send(job_result);

        log::trace!("Job result for job {my_job_id:x} sent");
    });

    handle
}

/// A handle to a started parallel job.
#[derive(Debug)]
pub struct JobHandle<T> {
    /// The fully unique ID of the job
    job_id: u64,

    /// The job return value and synchronization
    inner: Mutex<JobHandleInner<T>>,
}

#[derive(Debug)]
enum JobHandleInner<T> {
    /// Job is waiting to be done
    Waiting(Receiver<T>),

    /// Job is complete
    Ready(T),
}

impl<T> JobHandle<T> {
    /// Returns true if the job is complete and has its return value ready.
    pub fn ready(&self) -> bool {
        let mut inner = self.inner.lock().unwrap();

        match inner.deref() {
            JobHandleInner::Waiting(recv) => {
                if let Ok(result) = recv.try_recv() {
                    *inner = JobHandleInner::Ready(result);
                    true
                } else {
                    false
                }
            }
            JobHandleInner::Ready(_) => true,
        }
    }

    /// Blocks until the job is complete, and returns the job return value
    pub fn result(self) -> T {
        match self.inner.into_inner().unwrap() {
            JobHandleInner::Waiting(receiver) => receiver.recv().expect("Job stopped"),
            JobHandleInner::Ready(result) => result,
        }
    }
}

impl<T> Display for JobHandle<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "JobHandle(id={:x})", self.job_id)
    }
}
