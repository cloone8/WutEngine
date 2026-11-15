#![doc = include_str!("../README.md")]

use std::sync::Mutex as StdMutex;
use std::sync::MutexGuard;
use std::sync::RwLock as StdRwLock;
use std::sync::RwLockReadGuard;
use std::sync::RwLockWriteGuard;
use std::sync::TryLockError;

/// Wrapper around [the standard `Mutex`](std::sync::Mutex) for nonpoisoning calls
#[derive(Debug)]
#[repr(transparent)]
pub struct Mutex<T>(StdMutex<T>);

impl<T> Mutex<T> {
    /// Creates a new [`Mutex`]
    #[inline]
    pub const fn new(val: T) -> Self {
        Self(StdMutex::new(val))
    }

    /// Same as [`StdMutex::lock`], but panics on poison
    #[inline]
    #[expect(
        clippy::disallowed_methods,
        reason = "Disallowed because this is the only place that can use it"
    )]
    pub fn lock(&self) -> MutexGuard<'_, T> {
        self.0.lock().expect("Lock poisoned")
    }

    /// Same as [`StdMutex::try_lock`]
    #[inline]
    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        match self.0.try_lock() {
            Ok(guard) => Some(guard),
            Err(TryLockError::WouldBlock) => None,
            Err(TryLockError::Poisoned(_)) => {
                unreachable!("Panic should abort");
            }
        }
    }
}

/// Wrapper around [the standard `RwLock`](std::sync::RwLock) for nonpoisoning calls
#[derive(Debug)]
#[repr(transparent)]
pub struct RwLock<T>(StdRwLock<T>);

impl<T> RwLock<T> {
    /// Creates a new [`RwLock`]
    #[inline]
    pub const fn new(val: T) -> Self {
        Self(StdRwLock::new(val))
    }

    /// Same as [`StdRwLock::read`], but panics on poison
    #[inline]
    #[expect(
        clippy::disallowed_methods,
        reason = "Disallowed because this is the only place that can use it"
    )]
    pub fn read(&self) -> RwLockReadGuard<'_, T> {
        self.0.read().expect("Lock poisoned")
    }

    /// Same as [`StdRwLock::write`], but panics on poison
    #[inline]
    #[expect(
        clippy::disallowed_methods,
        reason = "Disallowed because this is the only place that can use it"
    )]
    pub fn write(&self) -> RwLockWriteGuard<'_, T> {
        self.0.write().expect("Lock poisoned")
    }
}
