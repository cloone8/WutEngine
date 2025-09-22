//! Types and functions for the "Entity" part of the WutEngine ECS

use core::fmt::Display;

use hecs::DynamicBundle;
use wutengine_util::hash::nohash_hasher;

use crate::entity::builder::EntityBuilder;
use crate::prelude::Component;

pub mod builder;

#[derive(Clone, Copy, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Entity(pub(crate) hecs::Entity);

impl Entity {
    #[inline(always)]
    pub(crate) const fn from_hecs(he: hecs::Entity) -> Self {
        Self(he)
    }

    pub fn spawn() -> EntityBuilder {
        EntityBuilder::new()
    }
}

impl Eq for Entity {}

impl PartialEq for Entity {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl core::hash::Hash for Entity {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        let inner: u64 = self.0.to_bits().get();

        inner.hash(state);
    }
}

impl nohash_hasher::IsEnabled for Entity {}

impl Display for Entity {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Entity({:x})", self.0.to_bits().get())
    }
}
