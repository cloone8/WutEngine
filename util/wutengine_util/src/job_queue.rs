//! Job queueing and throttling

use core::error::Error;
use core::fmt::Display;
use core::num::NonZero;
use std::sync::Arc;
use std::sync::Condvar;
use std::sync::Mutex;

/// A job queue that can issue jobs for other users to consume. Has a certain budget, after which issuing
/// new jobs blocks the thread until another job is complete
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct JobQueue {
    job_budget: Arc<(Mutex<JobState>, Condvar)>,
}

#[derive(Debug)]
struct JobState {
    canceled: bool,
    jobs_available: usize,
}

impl JobState {
    fn new(budget: NonZero<usize>) -> Self {
        Self {
            canceled: false,
            jobs_available: budget.get(),
        }
    }
}

/// An error from [`JobQueue::issue_job`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobIssueErr {
    /// The job queue was canceled
    Canceled,
}

impl Display for JobIssueErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Canceled => write!(f, "The job was canceled"),
        }
    }
}

impl Error for JobIssueErr {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

impl JobQueue {
    /// Create a new job queue with a given concurrent job budget
    pub fn new(budget: NonZero<usize>) -> Self {
        let job_budget_mtx = Mutex::new(JobState::new(budget));
        let job_budget_condvar = Condvar::new();

        let job_budget = Arc::new((job_budget_mtx, job_budget_condvar));

        Self { job_budget }
    }

    /// Issues a new job, returning a [`token`](JobToken) to it.
    /// If the resulting token is dropped, the job slot is freed again
    pub fn issue_job(&self) -> Result<JobToken, JobIssueErr> {
        // Wait for a slot to open up

        let job_budget_lock = self.job_budget.0.lock().unwrap();

        let mut job_budget_lock = self
            .job_budget
            .1
            .wait_while(job_budget_lock, |job_state| {
                // Wait while the budget is 0 AND the job is not canceled yet
                !job_state.canceled && job_state.jobs_available == 0
            })
            .unwrap();

        if job_budget_lock.canceled {
            return Err(JobIssueErr::Canceled);
        }

        job_budget_lock.jobs_available -= 1;

        drop(job_budget_lock);

        Ok(JobToken {
            job_budget: self.job_budget.clone(),
        })
    }
}

/// A token to a job in a [``JobQueue``]. Frees up its slot in the queue once dropped
#[derive(Debug)]
#[repr(transparent)]
pub struct JobToken {
    job_budget: Arc<(Mutex<JobState>, Condvar)>,
}

impl JobToken {
    /// Cancels the entire job queue. After this returns, no more job tokens will be issued again for the entire queue
    pub fn cancel_job_queue(self) {
        let mut job_budget_lock = self.job_budget.0.lock().unwrap();

        job_budget_lock.canceled = true;

        drop(job_budget_lock);

        self.job_budget.1.notify_all();
    }
}

impl Drop for JobToken {
    fn drop(&mut self) {
        let mut job_budget_lock = self.job_budget.0.lock().unwrap();

        job_budget_lock.jobs_available += 1;

        drop(job_budget_lock);
        self.job_budget.1.notify_all();
    }
}
