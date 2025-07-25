//! Job system for WutEngine

use core::fmt::Display;
use core::ops::Deref;
use core::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, sync_channel};

/// Global Job ID counter. Automatically incremented every time a new job is started
static NEXT_JOB_ID: AtomicU64 = AtomicU64::new(0);

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

#[derive(Debug)]
pub struct JobHandle<T> {
    job_id: u64,
    inner: Mutex<JobHandleInner<T>>,
}

#[derive(Debug)]
enum JobHandleInner<T> {
    Waiting(Receiver<T>),
    Ready(T),
}

impl<T> JobHandle<T> {
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

    pub fn result(self) -> T {
        match self.inner.into_inner().unwrap() {
            JobHandleInner::Waiting(receiver) => receiver.recv().expect("Job stopped"),
            JobHandleInner::Ready(result) => result,
        }
    }
}

impl<T> Display for JobHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JobHandle(id={:x})", self.job_id)
    }
}
