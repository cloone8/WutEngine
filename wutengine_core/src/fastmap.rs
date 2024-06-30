use core::{
    fmt::Debug,
    hash::{BuildHasherDefault, Hash},
};
use std::collections::HashMap;

use nohash_hasher::NoHashHasher;

use crate::id::{instance::InstanceID, KeyType};

#[derive(Debug)]
pub struct FastMap<K, V>
where
    K: nohash_hasher::IsEnabled + Eq + Hash + Debug,
{
    map: HashMap<K, V, BuildHasherDefault<NoHashHasher<KeyType>>>,
}

impl<K, V> FastMap<K, V>
where
    K: nohash_hasher::IsEnabled + Eq + Hash + Debug,
{
    pub fn new() -> Self {
        FastMap {
            map: HashMap::with_hasher(BuildHasherDefault::default()),
        }
    }

    pub fn insert(&mut self, key: K, val: V) {
        debug_assert!(
            !self.map.contains_key(&key),
            "Map already contains item with key {:?}",
            key
        );

        self.map.insert(key, val);
    }

    pub fn get(&self, key: K) -> Option<&V> {
        self.map.get(&key)
    }

    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        self.map.get_mut(&key)
    }
}

impl<K, V> IntoIterator for FastMap<K, V>
where
    K: nohash_hasher::IsEnabled + Eq + Hash + Debug,
{
    type Item = (K, V);

    type IntoIter = std::collections::hash_map::IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}

impl<'a, K, V> IntoIterator for &'a FastMap<K, V>
where
    K: nohash_hasher::IsEnabled + Eq + Hash + Debug,
{
    type Item = (&'a K, &'a V);

    type IntoIter = std::collections::hash_map::Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter()
    }
}

impl<'a, K, V> IntoIterator for &'a mut FastMap<K, V>
where
    K: nohash_hasher::IsEnabled + Eq + Hash + Debug,
{
    type Item = (&'a K, &'a mut V);

    type IntoIter = std::collections::hash_map::IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter_mut()
    }
}
