//! Job queueing and throttling

use alloc::sync::Arc;
use core::num::NonZero;
use std::sync::Condvar;
use std::sync::Mutex;

/// A job queue that can issue jobs for other users to consume. Has a certain budget, after which issuing
/// new jobs blocks the thread until another job is complete
#[derive(Debug, Clone)]
#[repr(transparent)]
pub(crate) struct JobQueue {
    job_budget: Arc<(Mutex<usize>, Condvar)>,
}

impl JobQueue {
    /// Create a new job queue with a given concurrent job budget
    pub(crate) fn new(budget: NonZero<usize>) -> Self {
        let job_budget_mtx = Mutex::new(budget.get());
        let job_budget_condvar = Condvar::new();

        let job_budget = Arc::new((job_budget_mtx, job_budget_condvar));

        Self { job_budget }
    }

    /// Issues a new job, returning a [token](JobToken) to it.
    /// If the resulting token is dropped, the job slot is freed again
    pub(crate) fn issue_job(&self) -> JobToken {
        // Wait for a slot to open up

        let job_budget_lock = self.job_budget.0.lock().unwrap();
        let mut job_budget_lock = self
            .job_budget
            .1
            .wait_while(job_budget_lock, |budget| *budget == 0)
            .unwrap();

        *job_budget_lock -= 1;
        drop(job_budget_lock);

        JobToken {
            job_budget: self.job_budget.clone(),
        }
    }
}

/// A token to a job in a [JobQueue]. Frees up its slot in the queue once dropped
#[derive(Debug)]
#[repr(transparent)]
pub(crate) struct JobToken {
    job_budget: Arc<(Mutex<usize>, Condvar)>,
}

impl Drop for JobToken {
    fn drop(&mut self) {
        let mut job_budget_lock = self.job_budget.0.lock().unwrap();

        *job_budget_lock += 1;

        drop(job_budget_lock);
        self.job_budget.1.notify_all();
    }
}
