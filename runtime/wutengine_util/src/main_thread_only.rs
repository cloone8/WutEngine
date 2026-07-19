//! Module for the [MainThreadOnly] type

use core::ops::Deref;
use core::ops::DerefMut;

use crate::assert_main_thread;

/// Type that can only be accessed from the main thread, and can thus always be marked [Send]/[`Sync`]
#[derive(Debug)]
#[repr(transparent)]
pub struct MainThreadOnly<T>(T);

impl<T> MainThreadOnly<T> {
    /// Creates a new [`MainThreadOnly`]
    #[inline]
    pub const fn new(val: T) -> Self {
        Self(val)
    }

    /// Returns a reference to the contained value, or panics if not on the main thread
    #[inline]
    pub fn get(this: &Self) -> &T {
        assert_main_thread!();

        &this.0
    }

    /// Returns a mutable reference to the contained value, or panics if not on the main thread
    #[inline]
    pub fn get_mut(this: &mut Self) -> &mut T {
        assert_main_thread!();

        &mut this.0
    }

    /// Returns the contained value, or panics if not on the main thread
    #[inline]
    pub fn take(this: Self) -> T {
        assert_main_thread!();

        this.0
    }
}

impl<T> Deref for MainThreadOnly<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        Self::get(self)
    }
}

impl<T> DerefMut for MainThreadOnly<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        Self::get_mut(self)
    }
}

/// Safety: Every access is checked for being on the main thread
unsafe impl<T> Send for MainThreadOnly<T> {}

/// Safety: Every access is checked for being on the main thread
unsafe impl<T> Sync for MainThreadOnly<T> {}
