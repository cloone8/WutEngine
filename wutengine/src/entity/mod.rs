//! Types and functions for the "Entity" part of the WutEngine ECS

use wutengine_util::hash::nohash_hasher;

#[derive(Clone, Copy, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Entity(pub(crate) hecs::Entity);

impl Eq for Entity {}

impl PartialEq for Entity {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl core::hash::Hash for Entity {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let inner: u64 = self.0.to_bits().get();

        inner.hash(state);
    }
}

impl nohash_hasher::IsEnabled for Entity {}
