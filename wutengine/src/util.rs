//! Utility functions and macros

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ops::Deref;

/// Creates a hashmap and inserts the given keys and values. [Into::into] is
/// called on each key and value before it is inserted.
///
/// Used like:
/// ```
/// use wutengine_util::map; // Or `wutengine::map` when using the full WutEngine engine crate
/// use std::collections::HashMap;
///
/// let new_map: HashMap<String, i32> = map![
///     "a" => 1,
///     "b" => 2
/// ];
/// ```
#[macro_export]
macro_rules! map {
    () => {
        ::std::collections::HashMap::default()
    };

    ($($key:expr => $val:expr),+) => {{
        let mut new_hashmap = ::std::collections::HashMap::default();

        $(
            new_hashmap.insert($key.into(), $val.into());
        )*

        new_hashmap
    }};
}

/// Macro that marks the current spot as unreachable. Checked in debug builds,
/// unchecked in release builds.
macro_rules! unreachable_dbg {
    ($($arg:tt)*) => {{
        // Dummy unsafe no-op to force unsafe{} around this macro
        #[allow(clippy::useless_transmute, reason = "Dummy op")]
        {
        _ = ::core::mem::transmute::<(), ()>(());
        }

        #[cfg(debug_assertions)]
        unreachable!($($arg)*);

        #[cfg(not(debug_assertions))]
        ::core::hint::unreachable_unchecked();
    }};
}

pub(crate) use unreachable_dbg;

/// Set-once global static wrapper.
/// Makes it easier to use the various static lazy-initialized manager
/// structs in WutEngine.
///
/// Must be initialized exactly once using [Self::init]. This is checked in
/// debug builds but _not_ in release builds, where not initializing
/// the manager can lead to UB
#[derive(Debug)]
pub(crate) struct InitOnce<T> {
    #[cfg(debug_assertions)]
    /// Whether this manager was initialized. Checked in debug builds only
    initialized: core::sync::atomic::AtomicBool,

    /// The actual inner object
    inner: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T> Sync for InitOnce<T> where T: Sync {}

impl<T> InitOnce<T> {
    #[allow(
        clippy::new_without_default,
        reason = "Should not usually be used except as const-initialized statics"
    )]
    /// Creates a new, uninitialized global manager. Must be initialized at runtime with [Self::init] before use
    pub(crate) const fn new() -> Self {
        Self {
            #[cfg(debug_assertions)]
            initialized: core::sync::atomic::AtomicBool::new(false),

            inner: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    /// In debug builds, asserts that the value was initialized
    #[inline(always)]
    fn assert_initialized(&self) {
        #[cfg(debug_assertions)]
        {
            use core::sync::atomic::Ordering;

            if !self.initialized.load(Ordering::Acquire) {
                panic!(
                    "Global manager of type {} not yet initialized!",
                    core::any::type_name::<T>()
                );
            }
        }
    }

    /// Initializes the [InitOnce] to the given value.
    /// Must be called exactly once, and only once.
    /// This is checked in debug builds
    pub(crate) fn init(target: &Self, val: T) {
        ::log::debug!(
            "Initializing GlobalManager of type {}",
            core::any::type_name::<T>()
        );

        #[cfg(debug_assertions)]
        {
            use core::sync::atomic::Ordering;

            if target.initialized.swap(true, Ordering::AcqRel) {
                panic!(
                    "Global manager of type {} already initialized!",
                    core::any::type_name::<T>()
                );
            }
        }

        unsafe { target.inner.get().as_mut().unwrap().write(val) };
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

impl<T> Deref for InitOnce<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        InitOnce::get(self)
    }
}
