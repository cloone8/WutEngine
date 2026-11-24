//! Async executor

use core::cell::RefCell;
use core::pin::Pin;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;
use core::task::Context;
use core::task::Poll;
use core::task::Waker;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::TryRecvError;
use std::task::Wake;

use wutengine_macro::unique_id_type64;
use wutengine_sync::Mutex;

use crate::log;

#[cfg(test)]
mod test;

#[cfg(not(target_arch = "wasm32"))]
mod block_on;

#[derive(Debug)]
struct ThreadLocalExecutorSignal {
    task: TaskId,

    /// Set while the task's ID is in the ready queue, so repeated wakes queue it only once.
    /// Cleared by the executor right before it polls the task
    queued: AtomicBool,

    mark_ready: Sender<TaskId>,
    waker: Arc<Mutex<Waker>>,
}

impl Wake for ThreadLocalExecutorSignal {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        if self.queued.swap(true, Ordering::AcqRel) {
            // Already in the ready queue, and the host was woken when it was put there
            return;
        }

        if self.mark_ready.send(self.task).is_err() {
            log::warn!(
                "Signal for task {} was fired, but receiver channel is gone.",
                self.task
            );
        }

        let waker = self.waker.lock().clone(); // Clone so we can release the lock before waking
        waker.wake_by_ref();
    }
}

unique_id_type64! {
    /// A unique ID for a task in a [`ThreadLocalExecutor`]
    TaskId
}

type PinnedFuture = Pin<Box<dyn Future<Output = ()>>>;

#[derive(derive_more::Debug)]
struct TaskInfo {
    #[debug(skip)]
    future: PinnedFuture,
    signal: Arc<ThreadLocalExecutorSignal>,
    waker: Waker,
}

impl TaskInfo {
    fn new(
        fut: impl Future<Output = ()> + 'static,
        signal: Arc<ThreadLocalExecutorSignal>,
    ) -> Self {
        Self {
            future: Box::pin(fut) as PinnedFuture,
            waker: Waker::from(Arc::clone(&signal)),
            signal,
        }
    }
}

/// An async executor that runs on the main thread
#[derive(derive_more::Debug)]
pub(crate) struct ThreadLocalExecutor {
    #[debug(skip)]
    tasks: RefCell<HashMap<TaskId, TaskInfo>>,
    ready: Receiver<TaskId>,
    mark_ready: Sender<TaskId>,
    waker: Arc<Mutex<Waker>>,
}

impl ThreadLocalExecutor {
    /// Create a new, empty, [`ThreadLocalExecutor`]
    pub(crate) fn new() -> Self {
        let (send, recv) = mpsc::channel::<TaskId>();

        Self {
            tasks: RefCell::new(HashMap::new()),
            ready: recv,
            mark_ready: send,
            waker: Arc::new(Mutex::new(Waker::noop().clone())),
        }
    }

    /// Spawns a future on this executor
    pub(crate) fn spawn(&self, future: impl Future<Output = ()> + 'static) {
        profiling::function_scope!();

        let id = TaskId::new();

        log::trace!("Spawning new future with ID {id}");

        let signal = Arc::new(ThreadLocalExecutorSignal {
            task: id,
            queued: AtomicBool::new(false),
            mark_ready: self.mark_ready.clone(),
            waker: Arc::clone(&self.waker),
        });

        let task_info = TaskInfo::new(future, signal);
        task_info.waker.wake_by_ref(); // Queue once so the new task is polled at least once

        self.tasks.borrow_mut().insert(id, task_info);
    }

    /// Runs all ready tasks in the executor once. Returns how many
    /// tasks were polled
    pub(crate) fn run_ready(&self, host: &Waker) -> usize {
        profiling::function_scope!();

        // Replace the waker with the new one
        self.waker.lock().clone_from(host);

        // First we collect all tasks that are ready _now_.
        // If we kept collecting new ready tasks while we poll, it's possible
        // that we'd loop forever if a task kept waking itself
        let mut to_poll = Vec::new();

        loop {
            match self.ready.try_recv() {
                Ok(id) => {
                    to_poll.push(id);
                }
                Err(TryRecvError::Empty) => {
                    break;
                }
                Err(TryRecvError::Disconnected) => {
                    panic!("Missing sender");
                }
            }
        }

        let mut num_polled = 0;

        for ready_id in to_poll {
            let Some(mut task) = self.tasks.borrow_mut().remove(&ready_id) else {
                continue; // A finished task, woken by a waker that outlived it
            };

            // Mark the task as un-queued, so a new wake on the signal wakes us again
            task.signal.queued.swap(false, Ordering::AcqRel);

            let mut context = Context::from_waker(&task.waker);

            num_polled += 1;

            match task.future.as_mut().poll(&mut context) {
                Poll::Ready(()) => {}
                Poll::Pending => {
                    self.tasks.borrow_mut().insert(ready_id, task);
                }
            }
        }

        num_polled
    }
}
