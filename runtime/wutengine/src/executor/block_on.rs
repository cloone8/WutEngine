use core::mem::DropGuard;
use core::pin::pin;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;
use core::task::Context;
use core::task::Poll;
use core::task::Waker;
use std::sync::Arc;
use std::task::Wake;
use std::thread;
use std::thread::Thread;

use crate::executor::ThreadLocalExecutor;

/// Basic async signal
struct Signal {
    thread: Thread,
    woken: AtomicBool,
}

impl Signal {
    fn new_on_current() -> Self {
        Self {
            thread: thread::current(),
            woken: AtomicBool::new(false),
        }
    }
}

impl Wake for Signal {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.woken.store(true, Ordering::Release);
        self.thread.unpark();
    }
}

impl ThreadLocalExecutor {
    /// Blocks the current thread until `fut` is done, and keeps running futures while it's not
    pub(crate) fn block_on<T>(&self, fut: impl Future<Output = T>) -> T {
        profiling::function_scope!();

        // Make sure we restore the old waker so tasks don't try
        // to wake this local-thread signal after we're done here
        let prev_waker = self.waker.lock().clone();
        let _restore_waker = DropGuard::new((), |()| {
            self.waker.lock().clone_from(&prev_waker);
            prev_waker.wake_by_ref();
        });

        let mut pinned_future = pin!(fut);

        let signal = Arc::new(Signal::new_on_current());
        let waker = Waker::from(Arc::clone(&signal));

        let mut context = Context::from_waker(&waker);

        loop {
            match pinned_future.as_mut().poll(&mut context) {
                Poll::Pending => {
                    self.run_ready(&waker);
                    while !signal.woken.swap(false, Ordering::AcqRel) {
                        // Park ourselves until we were actually woken
                        thread::park();
                    }
                }
                Poll::Ready(result) => break result,
            }
        }
    }
}
