use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ops::Deref;
use core::sync::atomic::Ordering;

use crate::thread;
use crate::util::assert_main_thread;

/// Set-once global static wrapper.
/// Makes it easier to use the various static lazy-initialized manager
/// structs in WutEngine.
///
/// Must be initialized exactly once using [Self::init]. This is checked in
/// debug builds but _not_ in release builds, where not initializing
/// the manager can lead to UB
#[derive(Debug)]
pub(crate) struct InitOnce<
    T,
    const MAIN_THREAD_ONLY: bool = true,
    const BEFORE_THREAD_POOL: bool = true,
> {
    /// Whether this manager was initialized. Checked on every initialization, but only checked on deref in debug builds
    initialized: core::sync::atomic::AtomicU8,

    /// The actual inner object
    inner: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T, const MT: bool, const BTP: bool> Sync for InitOnce<T, MT, BTP> where T: Sync {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
enum InitOnceState {
    Uninitialized = 0,
    Initializing = 1,
    Initialized = 2,
}

impl<T, const MAIN_THREAD_ONLY: bool, const BEFORE_THREAD_POOL: bool>
    InitOnce<T, MAIN_THREAD_ONLY, BEFORE_THREAD_POOL>
{
    #[allow(
        clippy::new_without_default,
        reason = "Should not usually be used except as const-initialized statics"
    )]
    /// Creates a new, uninitialized global manager. Must be initialized at runtime with [Self::init] before use
    pub(crate) const fn new() -> Self {
        Self {
            initialized: core::sync::atomic::AtomicU8::new(InitOnceState::Uninitialized as u8),
            inner: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    /// In debug builds, asserts that the value was initialized
    #[inline(always)]
    fn assert_initialized(&self) {
        if cfg!(debug_assertions) {
            // unchecked in debug builds
            return;
        }

        if self.initialized.load(Ordering::Acquire) != InitOnceState::Initialized as u8 {
            panic!(
                "Global manager of type {} not yet initialized!",
                core::any::type_name::<T>()
            );
        }
    }

    /// Initializes the [InitOnce] to the given value.
    /// Must be called exactly once, and only once.
    /// This is checked in debug builds
    #[track_caller]
    pub(crate) fn init(target: &Self, val: T) {
        ::log::debug!(
            "Initializing InitOnce of type {}",
            core::any::type_name::<T>()
        );

        if const { MAIN_THREAD_ONLY } {
            assert_main_thread!();
        }

        if const { BEFORE_THREAD_POOL } {
            assert!(
                !thread::thread_pool_initialized(),
                "Thread pool has already been initialized. This can result in undefined behaviour"
            );
        }

        if target
            .initialized
            .swap(InitOnceState::Initializing as u8, Ordering::AcqRel)
            != InitOnceState::Uninitialized as u8
        {
            panic!(
                "InitOnce of type {} already initialized or being initialized!",
                core::any::type_name::<T>()
            );
        }

        unsafe { target.inner.get().as_mut().unwrap().write(val) };

        if target
            .initialized
            .swap(InitOnceState::Initialized as u8, Ordering::AcqRel)
            != InitOnceState::Initializing as u8
        {
            panic!(
                "InitOnce of type {} transitioned to a different state while being initialized!",
                core::any::type_name::<T>()
            );
        }
    }

    /// Gets the reference to the inner manager.
    /// Must only be called after calling [Self::init] once. This is only
    /// checked in debug builds.
    #[inline(always)]
    pub(crate) fn get(target: &Self) -> &T {
        target.assert_initialized();

        // Long method chain that optimizes to basically a pointer deref in release builds
        unsafe { target.inner.get().as_ref().unwrap().assume_init_ref() }
    }
}

impl<T, const MT: bool, const BTP: bool> Deref for InitOnce<T, MT, BTP> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        InitOnce::get(self)
    }
}
