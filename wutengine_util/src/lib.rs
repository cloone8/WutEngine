//! Internal WutEngine utilities

use core::any::Any;
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ops::Deref;

pub use wutengine_util_macro::*;

pub mod hash;
pub mod log;

/// Set-once global manager wrapper.
/// Makes it easier to use the various static lazy-initialized manager
/// structs in WutEngine.
///
/// Must be initialized exactly once using [Self::init]. This is checked in
/// debug builds but _not_ in release builds, where not initializing
/// the manager can lead to UB
#[derive(Debug)]
pub struct GlobalManager<T> {
    #[cfg(debug_assertions)]
    /// Whether this manager was initialized. Checked in debug builds only
    initialized: core::sync::atomic::AtomicBool,

    /// The actual inner object
    inner: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T> Sync for GlobalManager<T> where T: Sync {}

impl<T> GlobalManager<T> {
    #[allow(
        clippy::new_without_default,
        reason = "Should not usually be used except as const-initialized statics"
    )]
    /// Creates a new, uninitialized global manager. Must be initialized at runtime with [Self::init] before use
    pub const fn new() -> Self {
        Self {
            #[cfg(debug_assertions)]
            initialized: core::sync::atomic::AtomicBool::new(false),

            inner: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    /// In debug builds, asserts that the manager was initialized
    #[inline(always)]
    fn assert_initialized(&self) {
        #[cfg(debug_assertions)]
        {
            use core::sync::atomic::Ordering;

            if !self.initialized.load(Ordering::Acquire) {
                panic!(
                    "Global manager of type {} not yet initialized!",
                    std::any::type_name::<T>()
                );
            }
        }
    }

    /// Initializes the global manager to the given value.
    /// Must be called exactly once, and only once.
    /// This is checked in debug builds
    pub fn init(target: &GlobalManager<T>, val: T) {
        ::log::debug!(
            "Initializing GlobalManager of type {}",
            std::any::type_name::<T>()
        );

        #[cfg(debug_assertions)]
        {
            use core::sync::atomic::Ordering;

            if target.initialized.swap(true, Ordering::AcqRel) {
                panic!(
                    "Global manager of type {} already initialized!",
                    std::any::type_name::<T>()
                );
            }
        }

        unsafe { target.inner.get().as_mut().unwrap().write(val) };
    }

    /// Gets the reference to the inner manager.
    /// Must only be called after calling [Self::init] once. This is only
    /// checked in debug builds.
    #[inline(always)]
    pub fn get(target: &GlobalManager<T>) -> &T {
        target.assert_initialized();

        unsafe { target.inner.get().as_ref().unwrap().assume_init_ref() }
    }
}

impl<T> Deref for GlobalManager<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        GlobalManager::get(self)
    }
}

/// Trait for being able to retrieve the type name of dynamic trait objects
pub trait TypeName {
    /// Returns the type name of this object
    fn type_name(&self) -> &'static str;
}

impl<T> TypeName for T
where
    T: Any,
{
    fn type_name(&self) -> &'static str {
        core::any::type_name::<T>()
    }
}

/// Creates a hashmap and inserts the given keys and values. [Into::into] is
/// called on each key and value before it is inserted.
///
/// Used like:
/// ```
/// use wutengine::map;
/// use std::collections::HashMap;
///
/// let new_map: HashMap<String, i32> = map![
///     "a" => 1,
///     "b" => 2
/// ];
/// ```
#[macro_export]
macro_rules! map {
    ($($key:expr => $val:expr),+) => {{
        let mut new_hashmap = ::std::collections::HashMap::default();

        $(
            new_hashmap.insert($key.into(), $val.into());
        )*

        new_hashmap
    }};
}
