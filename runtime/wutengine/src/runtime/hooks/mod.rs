//! Various runtime hooks and hook registration

use core::mem;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;

use wutengine_sync::Mutex;
use wutengine_sync::thread_context;

use crate::log;

#[derive(Debug)]
struct RegisteredHooks<T: ?Sized> {
    hooks: Mutex<Vec<Box<T>>>,
    has_run: AtomicBool,
}

impl<T: ?Sized> RegisteredHooks<T> {
    const fn new() -> Self {
        Self {
            hooks: Mutex::new(Vec::new()),
            has_run: AtomicBool::new(false),
        }
    }

    fn insert(&self, hook: Box<T>) {
        self.hooks.lock().push(hook);
    }

    /// Borrows the hooks mutably, keeping the list as-is, except for hooks registered while running the hooks
    #[track_caller]
    #[cfg_attr(
        not(feature = "interactive"),
        expect(unused, reason = "Only used for quit requests at the moment")
    )]
    fn for_each_hook_mut(&self, mut cb: impl FnMut(&mut T)) {
        thread_context::assert_main_context();

        self.has_run.store(true, Ordering::Release);

        // Take out the current hooks so registering new ones while
        // we're running them doesn't deadlock
        let mut cur_hooks: Vec<_> = mem::take(self.hooks.lock().as_mut());

        for hook in cur_hooks.iter_mut().map(AsMut::as_mut) {
            cb(hook);
        }

        // Take out any newly registered hooks, and append them at the end of the "current" list
        let mut hooks = self.hooks.lock();

        cur_hooks.append(&mut hooks);

        *hooks = cur_hooks;
    }

    /// Consumes the hooks, leaving the list empty, except for hooks registered while running the hooks
    #[track_caller]
    fn for_each_hook_once(&self, mut cb: impl FnMut(Box<T>)) {
        thread_context::assert_main_context();

        self.has_run.store(true, Ordering::Release);

        // Take out the current hooks so registering new ones while
        // we're running them doesn't deadlock
        let cur_hooks: Vec<_> = mem::take(self.hooks.lock().as_mut());

        for hook in cur_hooks {
            cb(hook);
        }
    }

    fn has_run(&self) -> bool {
        self.has_run.load(Ordering::Acquire)
    }
}

static STARTED_HOOKS: RegisteredHooks<dyn FnOnce() + Send + 'static> = RegisteredHooks::new();

/// Registers a new hook that runs when the engine is done with startup, and is starting the main
/// loop
pub fn register_started_hook(hook: impl FnOnce() + Send + 'static) {
    if STARTED_HOOKS.has_run() {
        log::warn!("New started hook registered even though the event already fired");
    }

    STARTED_HOOKS.insert(Box::new(hook));
}

/// Run the `started` hooks. Should be called after startup and before
/// actually running the game
pub(crate) fn run_started_hooks() {
    profiling::function_scope!();

    assert!(!STARTED_HOOKS.has_run(), "Started hooks already ran once");

    log::debug!("Running `started` hooks");

    STARTED_HOOKS.for_each_hook_once(|hook| {
        hook();
    });
}

static QUITTING_HOOKS: RegisteredHooks<dyn FnOnce() + Send + 'static> = RegisteredHooks::new();

/// Register a new `quitting` hook that fires when the engine is shutting down.
pub fn register_quitting_hook(hook: impl FnOnce() + Send + 'static) {
    if QUITTING_HOOKS.has_run() {
        log::warn!("New started hook registered even though the event already fired");
    }

    QUITTING_HOOKS.insert(Box::new(hook));
}

/// Run the `quitting` hooks. Should be called after quitting starts
pub(crate) fn run_quitting_hooks() {
    profiling::function_scope!();

    assert!(!QUITTING_HOOKS.has_run(), "Quitting hooks already ran once");

    log::debug!("Running `quitting` hooks");

    QUITTING_HOOKS.for_each_hook_once(|hook| {
        hook();
    });
}

/// Information about a quit request, delivered through a [`quit requested` hook](register_quit_requested_hook)
#[derive(Debug)]
#[expect(missing_copy_implementations, reason = "Should be shared")]
pub struct QuitRequest {
    cancelled: bool,
}

#[cfg_attr(
    not(feature = "interactive"),
    expect(unused, reason = "Quit requests are not for headless mode")
)]
impl QuitRequest {
    /// A new [`QuitRequest`]
    pub(crate) fn new() -> Self {
        Self { cancelled: false }
    }

    /// Cancel the quit request.
    pub fn cancel(&mut self) {
        self.cancelled = true;
    }

    /// Returns whether this request has been cancelled
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

static QUIT_REQUESTED_HOOKS: RegisteredHooks<dyn FnMut(&mut QuitRequest) + Send + 'static> =
    RegisteredHooks::new();

/// Register a new `quit requested` hook. Each hook gets a request object that
/// allows the hook to cancel the quit. Cancelling a quit is final and
/// cannot be overridden by other hooks. All hooks run, in registration order, even
/// if an earlier hook already cancelled the request.
pub fn register_quit_requested_hook(hook: impl FnMut(&mut QuitRequest) + Send + 'static) {
    QUIT_REQUESTED_HOOKS.insert(Box::new(hook));
}

/// Run the `quit requested` hooks. See [`register_quit_requested_hook`]
#[cfg_attr(
    not(feature = "interactive"),
    expect(unused, reason = "Only used for quit requests at the moment")
)]
pub(crate) fn run_quit_requested_hooks(request: &mut QuitRequest) {
    profiling::function_scope!();

    log::debug!("Running `quit requested` hooks");

    QUIT_REQUESTED_HOOKS.for_each_hook_mut(|hook| {
        hook(request);
    });
}
