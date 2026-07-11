use core::{fmt::Display, ops::Deref};

use alloc::string::String;

use crate::component::Component;

/// Simple component describing the user-assigned name for an entity
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct Name(pub(crate) String);

impl Name {
    /// Create a new [Name] from the given string
    #[inline]
    pub fn new(name: String) -> Self {
        Self(name)
    }
}

impl Component for Name {
    const ID: uuid::NonNilUuid =
        uuid::NonNilUuid::new(uuid::uuid!("aeed35e7-4dbb-4ec4-9d19-5478fe9ca4e2")).unwrap();
}

impl Deref for Name {
    type Target = str;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}
