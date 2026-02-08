//! Various graphics caches

use std::collections::HashMap;
use std::hash::RandomState;
use std::sync::{Arc, RwLock};

pub(crate) mod pipeline;
pub(crate) mod sampler;
pub(crate) mod shader;

/// A generic cache for graphics resources. Synchronized, so can be put in a static
#[derive(Debug)]
pub(crate) struct GraphicsCache<K, V, H = RandomState>(RwLock<HashMap<K, Arc<V>, H>>);

impl<K, V, H> Default for GraphicsCache<K, V, H>
where
    H: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<K, V, H> GraphicsCache<K, V, H>
where
    K: core::hash::Hash + Eq,
    V: Clone,
    H: core::hash::BuildHasher,
{
    /// Tries to find the value for the given key in the cache
    #[inline]
    pub(crate) fn find(&self, key: &K) -> Option<Arc<V>> {
        let cache = self.0.read().unwrap();

        cache.get(key).map(Clone::clone)
    }

    /// Inserts the given value under the given key. If the key already exists,
    /// does not insert the new value and simply returns the already existing one
    #[inline]
    pub(crate) fn insert(&self, key: K, value: V) -> Arc<V> {
        let mut cache = self.0.write().unwrap();

        if let Some(existing) = cache.get(&key) {
            existing.clone()
        } else {
            let as_arc = Arc::new(value);
            cache.insert(key, as_arc.clone());

            as_arc
        }
    }
}
