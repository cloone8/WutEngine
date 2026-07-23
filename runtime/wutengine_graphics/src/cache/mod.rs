//! Various graphics caches

use alloc::sync::Arc;
use core::hash::{BuildHasher, Hash};
use std::hash::RandomState;

pub(crate) mod pipeline;
pub(crate) mod sampler;
pub(crate) mod shader;

/// A generic cache for graphics resources. Synchronized, so can be put in a static
#[derive(Debug)]
pub(crate) struct GraphicsCache<K, V, H = RandomState>(dashmap::DashMap<K, Arc<V>, H>)
where
    K: Eq + Hash,
    H: BuildHasher + Clone;

impl<K, V, H> Default for GraphicsCache<K, V, H>
where
    K: Eq + Hash,
    H: Default + BuildHasher + Clone,
{
    fn default() -> Self {
        Self(dashmap::DashMap::default())
    }
}

impl<K, V, H> GraphicsCache<K, V, H>
where
    K: core::hash::Hash + Eq,
    H: core::hash::BuildHasher + Clone,
{
    /// Tries to find the value for the given key in the cache
    #[inline]
    pub(crate) fn find(&self, key: &K) -> Option<Arc<V>> {
        self.0.get(key).map(|val| val.clone())
    }

    /// Inserts the given value under the given key. If the key already exists,
    /// does not insert the new value and simply returns the already existing one
    #[inline]
    pub(crate) fn insert(&self, key: K, value: V) -> Arc<V> {
        self.0.entry(key).or_insert_with(|| Arc::new(value)).clone()
    }
}
