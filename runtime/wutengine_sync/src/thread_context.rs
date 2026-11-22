//! Which kind of thread code runs on, and whether it runs inside a job
//!
//! Each thread has a [`ThreadKind`], set once when the thread starts, plus a job depth raised by [`JobGuard`]. Together
//! they answer two questions:
//!
//! - [`in_main_context`]: on the main thread and outside any job. Main-thread-only state may be touched.
//! - [`in_frame_context`]: on the main thread or a job pool thread, so inside the frame's synchronization. Frame data
//!   such as the clocks may be read.

use core::cell::Cell;
use core::marker::PhantomData;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;

/// The kind of a thread, set once when the thread starts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadKind {
    /// The thread running the event loop and the frame. Marked by [`mark_main_thread`]
    Main,

    /// A job pool thread, which only runs frame work. Marked by [`mark_pool_thread`]
    JobPool,

    /// Any thread not marked otherwise: background computation, audio, loaders
    Other,
}

thread_local! {
    /// The kind of the current thread
    static THREAD_KIND: Cell<ThreadKind> = const { Cell::new(ThreadKind::Other) };

    /// How many [`JobGuard`]s are alive on the current thread
    static JOB_DEPTH: Cell<u32> = const { Cell::new(0) };
}

/// Whether a thread was already marked as the main thread, so a second one can be refused
static MAIN_THREAD_MARKED: AtomicBool = AtomicBool::new(false);

/// Marks the current thread as the main thread. Call once, when the runtime starts
///
/// Panics if a main thread was already marked, or if the current thread already has a kind
#[track_caller]
pub fn mark_main_thread() {
    let already_marked = MAIN_THREAD_MARKED.swap(true, Ordering::Relaxed);
    assert!(!already_marked, "Main thread already marked");

    set_thread_kind(ThreadKind::Main);
}

/// Marks the current thread as a job pool thread. Call from the pool's thread-start hook
///
/// Panics if the current thread already has a kind
#[track_caller]
pub fn mark_pool_thread() {
    set_thread_kind(ThreadKind::JobPool);
}

/// Sets the kind of the current thread.
///
/// If the thread kind has already been set to anything other than [`ThreadKind::Other`], panics.
#[track_caller]
fn set_thread_kind(kind: ThreadKind) {
    let current = THREAD_KIND.get();

    assert_eq!(
        current,
        ThreadKind::Other,
        "Thread kind already set to {current:?}, cannot set it to {kind:?}"
    );

    THREAD_KIND.set(kind);
}

/// Returns the kind of the current thread
#[must_use]
#[inline]
pub fn thread_kind() -> ThreadKind {
    THREAD_KIND.get()
}

/// Returns whether the current thread is running a job
#[must_use]
#[inline]
pub fn in_job() -> bool {
    JOB_DEPTH.get() > 0
}

/// Returns whether code runs on the main thread, outside any job
#[must_use]
#[inline]
pub fn in_main_context() -> bool {
    thread_kind() == ThreadKind::Main && !in_job()
}

/// Returns whether code runs on the main thread or a job pool thread, inside or outside a job
#[must_use]
#[inline]
pub fn in_frame_context() -> bool {
    matches!(thread_kind(), ThreadKind::Main | ThreadKind::JobPool)
}

/// Panics unless code runs on the main thread, outside any job. See [`in_main_context`]
#[track_caller]
#[inline]
pub fn assert_main_context() {
    assert!(
        in_main_context(),
        "Must run on the main thread outside a job, but runs on a {:?} thread at job depth {}",
        thread_kind(),
        JOB_DEPTH.get()
    );
}

/// Panics unless code runs on the main thread or a job pool thread. See [`in_frame_context`]
#[track_caller]
#[inline]
pub fn assert_frame_context() {
    assert!(
        in_frame_context(),
        "Must run on the main thread or a job pool thread, but runs on a {:?} thread",
        thread_kind()
    );
}

/// Same as [`assert_frame_context`], but only checked in builds with debug assertions
#[track_caller]
#[inline]
pub fn dbg_assert_frame_context() {
    if cfg!(debug_assertions) {
        assert_frame_context();
    }
}

/// Marks the current thread as running a job while it is alive. The job wrapper holds one around each job it runs
///
/// Not [`Send`]: it must be dropped on the thread that created it.
#[derive(Debug)]
#[must_use = "the job ends when the guard is dropped"]
pub struct JobGuard {
    /// Keeps the guard on its thread
    _not_send: PhantomData<*const ()>,
}

impl JobGuard {
    /// Raises the current thread's job depth until the returned guard is dropped
    #[inline]
    pub fn enter() -> Self {
        JOB_DEPTH.set(JOB_DEPTH.get() + 1);

        Self {
            _not_send: PhantomData,
        }
    }
}

impl Drop for JobGuard {
    #[inline]
    fn drop(&mut self) {
        JOB_DEPTH.set(JOB_DEPTH.get() - 1);
    }
}
