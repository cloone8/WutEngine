#[cfg(not(target_arch = "wasm32"))]
mod block_on {

    use core::pin::Pin;
    use core::sync::atomic::AtomicBool;
    use core::sync::atomic::Ordering;
    use core::task::Context;
    use core::task::Poll;
    use core::task::Waker;
    use core::time::Duration;
    use std::sync::Arc;
    use std::sync::mpsc;
    use std::sync::mpsc::RecvTimeoutError;
    use std::thread;

    use crate::executor::ThreadLocalExecutor;

    /// Runs `fut` to completion with [`ThreadLocalExecutor::block_on`] on a new executor
    fn block_on<T>(fut: impl Future<Output = T>) -> T {
        ThreadLocalExecutor::new().block_on(fut)
    }

    /// How long a test waits for `block_on` to return. A lost wake parks the thread forever, so without a limit a
    /// broken `block_on` would hang the test run instead of failing it
    const TIMEOUT: Duration = Duration::from_secs(5);

    /// How long the waking thread sleeps before each wake, so the blocked thread has really parked by then
    const WAKE_DELAY: Duration = Duration::from_millis(50);

    /// Runs `f` on a thread of its own and returns its result, failing the test if it doesn't finish within
    /// [`TIMEOUT`]
    fn with_timeout<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
        let (result_tx, result_rx) = mpsc::channel();

        thread::spawn(move || {
            _ = result_tx.send(f());
        });

        match result_rx.recv_timeout(TIMEOUT) {
            Ok(result) => result,
            Err(RecvTimeoutError::Timeout) => {
                panic!("`block_on` didn't return within {TIMEOUT:?}: a wake was probably lost")
            }
            Err(RecvTimeoutError::Disconnected) => panic!("`block_on` panicked"),
        }
    }

    /// How a future wakes itself
    #[derive(Debug, Clone, Copy)]
    enum WakeStyle {
        /// [`Waker::wake_by_ref`]
        ByRef,

        /// [`Waker::wake`] on a clone, which consumes it
        ByValue,
    }

    /// A future that wakes itself and returns `Pending` `remaining` times, then returns how often it was polled
    #[derive(Debug)]
    struct YieldTimes {
        /// How many more times to return `Pending`
        remaining: usize,

        /// How often this future has been polled
        polls: usize,

        /// How the future wakes itself before returning `Pending`
        style: WakeStyle,
    }

    impl YieldTimes {
        /// A future that yields `times` times, waking itself the given way
        fn new(times: usize, style: WakeStyle) -> Self {
            Self {
                remaining: times,
                polls: 0,
                style,
            }
        }
    }

    impl Future for YieldTimes {
        type Output = usize;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<usize> {
            self.polls += 1;

            if self.remaining == 0 {
                return Poll::Ready(self.polls);
            }

            self.remaining -= 1;

            match self.style {
                WakeStyle::ByRef => cx.waker().wake_by_ref(),
                #[expect(
                    clippy::waker_clone_wake,
                    reason = "tests the consuming `wake`, which `wake_by_ref` would skip"
                )]
                WakeStyle::ByValue => cx.waker().clone().wake(),
            }

            Poll::Pending
        }
    }

    /// A future that is done once `done` is set by another thread. Each poll that returns `Pending` sends a clone of
    /// its waker to that thread. Returns how often it was polled
    #[derive(Debug)]
    struct WaitForFlag {
        /// Set by the waking thread when the future is done
        done: Arc<AtomicBool>,

        /// Hands the waker to the waking thread
        wakers: mpsc::Sender<Waker>,

        /// How often this future has been polled
        polls: usize,
    }

    impl Future for WaitForFlag {
        type Output = usize;

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<usize> {
            self.polls += 1;

            if self.done.load(Ordering::Acquire) {
                return Poll::Ready(self.polls);
            }

            self.wakers
                .send(cx.waker().clone())
                .expect("waking thread stopped early");

            Poll::Pending
        }
    }

    /// Blocks on a [`WaitForFlag`] while another thread wakes it `wakes` times, setting the flag only before the last
    /// wake. Returns how often the future was polled
    fn wait_for_other_thread(wakes: usize) -> usize {
        with_timeout(move || {
            let done = Arc::new(AtomicBool::new(false));
            let (wakers_tx, wakers_rx) = mpsc::channel::<Waker>();

            let waking_thread = {
                let done = Arc::clone(&done);

                thread::spawn(move || {
                    for i in 0..wakes {
                        let waker = wakers_rx.recv().expect("future dropped before being woken");

                        thread::sleep(WAKE_DELAY);

                        if i + 1 == wakes {
                            done.store(true, Ordering::Release);
                        }

                        waker.wake();
                    }
                })
            };

            let polls = block_on(WaitForFlag {
                done,
                wakers: wakers_tx,
                polls: 0,
            });

            waking_thread.join().expect("waking thread panicked");

            polls
        })
    }

    /// A future that is ready right away gives back its output
    #[test]
    fn ready_future_returns_output() {
        let output = with_timeout(|| block_on(async { String::from("done") }));

        assert_eq!(output, "done", "output of the future");
    }

    /// A future that is ready right away is polled exactly once
    #[test]
    fn ready_future_is_polled_once() {
        let polls = with_timeout(|| block_on(YieldTimes::new(0, WakeStyle::ByRef)));

        assert_eq!(polls, 1, "number of polls");
    }

    /// A future that wakes itself (by reference) before returning `Pending` is polled again. The wake comes before
    /// the thread parks, so this fails if a wake that arrives before parking is lost
    #[test]
    fn self_wake_by_ref() {
        let polls = with_timeout(|| block_on(YieldTimes::new(1, WakeStyle::ByRef)));

        assert_eq!(polls, 2, "number of polls");
    }

    /// Like [`self_wake_by_ref`], waking through a consumed clone of the waker instead
    #[test]
    fn self_wake_by_value() {
        let polls = with_timeout(|| block_on(YieldTimes::new(1, WakeStyle::ByValue)));

        assert_eq!(polls, 2, "number of polls");
    }

    /// Many wakes in a row each lead to one more poll
    #[test]
    fn many_self_wakes() {
        let polls = with_timeout(|| block_on(YieldTimes::new(10_000, WakeStyle::ByRef)));

        assert_eq!(polls, 10_001, "number of polls");
    }

    /// A wake from another thread, after the blocked thread has parked, resumes it
    #[test]
    fn wake_from_other_thread() {
        assert_eq!(wait_for_other_thread(1), 2, "number of polls");
    }

    /// Wakes from another thread while the future isn't done yet (spurious from the future's point of view) poll it
    /// again, and `block_on` keeps waiting until the future is really done
    #[test]
    fn early_wakes_from_other_thread() {
        assert_eq!(wait_for_other_thread(3), 4, "number of polls");
    }

    /// An `async` block awaiting several futures that each return `Pending` runs to the end
    #[test]
    fn async_block_with_awaits() {
        let total = with_timeout(|| {
            block_on(async {
                let first = YieldTimes::new(2, WakeStyle::ByRef).await;
                let second = YieldTimes::new(3, WakeStyle::ByValue).await;

                first + second
            })
        });

        assert_eq!(total, 3 + 4, "sum of both futures' poll counts");
    }

    /// Every poll gets a waker that wakes the same task as the first poll's, so futures that store their waker and
    /// compare it with [`Waker::will_wake`] don't have to replace it. Not needed for correctness, but cheap: make the
    /// waker once, before the loop
    #[test]
    fn same_waker_every_poll() {
        /// A future that yields `remaining` times, then returns whether every poll's waker would wake the same task
        /// as the first poll's
        #[derive(Debug)]
        struct CompareWakers {
            /// The first poll's waker
            first: Option<Waker>,

            /// How many more times to return `Pending`
            remaining: usize,

            /// Whether every waker so far would wake the same task as `first`
            all_same: bool,
        }

        impl Future for CompareWakers {
            type Output = bool;

            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<bool> {
                match &self.first {
                    Some(first) => {
                        let same = first.will_wake(cx.waker());
                        self.all_same &= same;
                    }
                    None => self.first = Some(cx.waker().clone()),
                }

                if self.remaining == 0 {
                    return Poll::Ready(self.all_same);
                }

                self.remaining -= 1;
                cx.waker().wake_by_ref();

                Poll::Pending
            }
        }

        let all_same = with_timeout(|| {
            block_on(CompareWakers {
                first: None,
                remaining: 3,
                all_same: true,
            })
        });

        assert!(all_same, "a poll got a waker for a different task");
    }

    /// A wake isn't lost when code inside `poll` uses the thread's park token for its own blocking wait.
    ///
    /// The future wakes itself, which turns the thread's single park token on, then does a blocking `mpsc` receive
    /// for a message another thread sends a little later. That receive parks internally: its first `park` consumes the
    /// token and returns at once, and it parks again until the message arrives. When `poll` then returns `Pending`, a
    /// `block_on` that relies only on the token parks with the token gone, and nothing wakes it again. A `block_on`
    /// that records wakes itself (a flag set by its waker) polls again
    #[test]
    fn wake_survives_blocking_inside_poll() {
        /// A future that, on its first poll, wakes itself and then blocks on `message`. Ready on its second poll,
        /// with how often it was polled
        #[derive(Debug)]
        struct WakeThenBlock {
            /// Receives the message the other thread sends after [`WAKE_DELAY`]
            message: mpsc::Receiver<()>,

            /// How often this future has been polled
            polls: usize,
        }

        impl Future for WakeThenBlock {
            type Output = usize;

            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<usize> {
                self.polls += 1;

                if self.polls > 1 {
                    return Poll::Ready(self.polls);
                }

                cx.waker().wake_by_ref();

                self.message
                    .recv()
                    .expect("sending thread stopped without sending");

                Poll::Pending
            }
        }

        let polls = with_timeout(|| {
            let (message_tx, message_rx) = mpsc::channel();

            let sending_thread = thread::spawn(move || {
                thread::sleep(WAKE_DELAY);
                message_tx.send(()).expect("future dropped its receiver");
            });

            let polls = block_on(WakeThenBlock {
                message: message_rx,
                polls: 0,
            });

            sending_thread.join().expect("sending thread panicked");

            polls
        });

        assert_eq!(polls, 2, "number of polls");
    }
}
