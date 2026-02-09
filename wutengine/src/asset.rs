//! Asset handling

use core::any::Any;
use core::ops::{Deref, DerefMut};
use std::sync::Arc;

/// Trait implemented by types that can be used as a WutEngine asset
pub trait Asset: Send + Sync + Any {}

/// A handle to a WutEngine asset
#[derive(Debug, Clone)]
pub struct Handle<T> {
    inner: Arc<T>,
}

impl<T> Deref for Handle<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}

impl<T> DerefMut for Handle<T>
where
    T: Clone,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        Arc::make_mut(&mut self.inner)
    }
}
