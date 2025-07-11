//! Internal WutEngine utilities

use core::any::Any;
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::ops::Deref;

pub use nohash_hasher;
pub use wutengine_util_macro::*;

#[derive(Debug)]
pub struct GlobalManager<T> {
    #[cfg(debug_assertions)]
    initialized: core::sync::atomic::AtomicBool,

    inner: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T> Sync for GlobalManager<T> where T: Sync {}

impl<T> GlobalManager<T> {
    pub const fn new() -> Self {
        Self {
            #[cfg(debug_assertions)]
            initialized: core::sync::atomic::AtomicBool::new(false),

            inner: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

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

    pub fn init(target: &GlobalManager<T>, val: T) {
        log::debug!(
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

pub trait TypeName {
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

#[macro_export]
macro_rules! warn_once {
    ($($tokens:tt)*) => {{
        static WARNED: ::core::sync::atomic::AtomicBool = ::core::sync::atomic::AtomicBool::new(false);

        if !WARNED.swap(true, ::core::sync::atomic::Ordering::AcqRel) {
            ::log::warn!($($tokens)*);
        }
    }};
}

#[macro_export]
macro_rules! err_once {
    ($($tokens:tt)*) => {{
        static ERRD: ::core::sync::atomic::AtomicBool = ::core::sync::atomic::AtomicBool::new(false);

        if !ERRD.swap(true, ::core::sync::atomic::Ordering::AcqRel) {
            ::log::error!($($tokens)*);
        }
    }};
}
