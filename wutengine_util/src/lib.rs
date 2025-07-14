//! Internal WutEngine utilities

use core::any::Any;
use core::cell::UnsafeCell;
use core::hash::Hash;
use core::mem::MaybeUninit;
use core::ops::Deref;
use std::collections::HashMap;

use md5::{Digest, Md5};
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

#[derive(Debug)]
struct Md5Hasher(Md5);

impl core::hash::Hasher for Md5Hasher {
    fn finish(&self) -> u64 {
        unimplemented!(
            "Do not use the standard finish method. MD5 is not means as a default hashmap algorithm"
        );
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }

    fn write_u8(&mut self, i: u8) {
        self.write(&[i])
    }

    fn write_u16(&mut self, i: u16) {
        self.write(&i.to_le_bytes())
    }

    fn write_u32(&mut self, i: u32) {
        self.write(&i.to_le_bytes())
    }

    fn write_u64(&mut self, i: u64) {
        self.write(&i.to_le_bytes())
    }

    fn write_u128(&mut self, i: u128) {
        self.write(&i.to_le_bytes())
    }

    fn write_usize(&mut self, i: usize) {
        self.write(&i.to_le_bytes())
    }
}

pub fn keyword_hash<V: core::hash::Hash, K: AsRef<str> + core::hash::Hash + Eq>(
    keywords: &HashMap<K, V>,
) -> u128 {
    let mut hasher = Md5Hasher(Md5::new());

    let mut keywords_tuples: Vec<(&str, &V)> = keywords
        .iter()
        .map(|(key, val)| (key.as_ref(), val))
        .collect();

    keywords_tuples.sort_by_key(|(key, _)| *key);

    for (key, val) in keywords_tuples {
        key.hash(&mut hasher);
        val.hash(&mut hasher);
    }

    u128::from_le_bytes(hasher.0.finalize().into())
}
