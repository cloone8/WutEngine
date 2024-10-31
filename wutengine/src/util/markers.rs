//! Marker structs and traits

use core::marker::PhantomData;

/// Empty marker struct that is not [Send]. Place in other types
/// to make them non-[Send] too.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub(crate) struct NonSend(PhantomData<*mut ()>);
unsafe impl Sync for NonSend {}

impl NonSend {
    /// New [NonSend]
    #[inline(always)]
    pub(crate) fn new() -> Self {
        Self(PhantomData)
    }
}

/// Empty marker struct that is not [Sync]. Place in other types
/// to make them non-[Sync] too.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub(crate) struct NonSync(PhantomData<*mut ()>);
unsafe impl Send for NonSync {}

impl NonSync {
    /// New [NonSync]
    #[inline(always)]
    pub(crate) fn new() -> Self {
        Self(PhantomData)
    }
}

/// Empty marker struct that is not [Send] nor [Sync]. Place in other types
/// to make them non-[Send] and non-[Sync] too.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub(crate) struct NonSendSync(NonSend, NonSync);

impl NonSendSync {
    /// New [NonSendSync]
    #[inline(always)]
    pub(crate) fn new() -> Self {
        Self(NonSend::new(), NonSync::new())
    }
}
