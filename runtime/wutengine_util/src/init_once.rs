//! Single init, slightly unsafe, zero-overhead global manager

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ops::Deref;
use core::sync::atomic::Ordering;

use crate::assert_main_thread;

/// Set-once global static wrapper.
/// Makes it easier to use the various static lazy-initialized manager
/// structs in WutEngine.
///
/// Must be initialized exactly once using [Self::init]. This is checked in
/// debug builds but _not_ in release builds, where not initializing
/// the manager can lead to UB
#[derive(Debug)]
pub struct InitOnce<T, const UNCHECKED: bool = false, const MAIN_THREAD_ONLY: bool = true> {
    /// Whether this manager was initialized. Checked on every initialization, but only checked on deref in debug builds
    initialized: core::sync::atomic::AtomicU8,

    /// The actual inner object
    inner: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T, const U: bool, const MT: bool> Sync for InitOnce<T, U, MT> where T: Sync {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
enum InitOnceState {
    Uninitialized = 0,
    Initializing = 1,
    Initialized = 2,
}

impl<T, const MAIN_THREAD_ONLY: bool> InitOnce<T, false, MAIN_THREAD_ONLY> {
    #[allow(
        clippy::new_without_default,
        reason = "Should not usually be used except as const-initialized statics"
    )]
    /// Creates a new, uninitialized global manager. Must be initialized at runtime with [Self::init] before use
    pub const fn new_checked() -> Self {
        Self {
            initialized: core::sync::atomic::AtomicU8::new(InitOnceState::Uninitialized as u8),
            inner: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }
}

impl<T, const MAIN_THREAD_ONLY: bool> InitOnce<T, true, MAIN_THREAD_ONLY> {
    #[allow(
        clippy::new_without_default,
        reason = "Should not usually be used except as const-initialized statics"
    )]
    /// Creates a new, uninitialized, unchecked global manager. Must be initialized at runtime with [Self::init] before use.
    ///
    /// # Safety
    ///
    /// MUST be initialized with [Self::init] before use to prevent UB
    pub const unsafe fn new_unchecked() -> Self {
        Self {
            initialized: core::sync::atomic::AtomicU8::new(InitOnceState::Uninitialized as u8),
            inner: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }
}

impl<T, const UNCHECKED: bool, const MAIN_THREAD_ONLY: bool>
    InitOnce<T, UNCHECKED, MAIN_THREAD_ONLY>
{
    /// Checks whether this [InitOnce] is initialized
    #[inline(always)]
    pub fn is_initialized(target: &Self) -> bool {
        target.initialized.load(Ordering::Acquire) == InitOnceState::Initialized as u8
    }

    /// In debug builds, asserts that the value was initialized
    #[inline(always)]
    fn assert_initialized(&self) {
        if const { UNCHECKED } && !cfg!(debug_assertions) {
            // unchecked in release builds
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
    pub fn init(target: &Self, val: T) {
        ::log::debug!(
            "Initializing InitOnce of type {}",
            core::any::type_name::<T>()
        );

        if const { MAIN_THREAD_ONLY } {
            assert_main_thread!();
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
    pub fn get(target: &Self) -> &T {
        target.assert_initialized();

        // Long method chain that optimizes to basically a pointer deref in release builds
        unsafe { target.inner.get().as_ref().unwrap().assume_init_ref() }
    }
}

impl<T, const U: bool, const MT: bool> Deref for InitOnce<T, U, MT> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        InitOnce::get(self)
    }
}
