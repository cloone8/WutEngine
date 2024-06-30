use core::hash::BuildHasherDefault;
use std::collections::HashMap;

use nohash_hasher::NoHashHasher;

use crate::id::{instance::InstanceID, KeyType};

#[derive(Debug)]
pub struct LookupTable<T: InstanceID> {
    map: HashMap<KeyType, T, BuildHasherDefault<NoHashHasher<KeyType>>>,
}

impl<T: InstanceID> LookupTable<T> {
    pub fn new() -> Self {
        LookupTable {
            map: HashMap::with_hasher(BuildHasherDefault::default()),
        }
    }

    pub fn insert(&mut self, item: T) {
        debug_assert!(
            !self.map.contains_key(&item.id()),
            "Map already contains item with key {}",
            item.id()
        );

        self.map.insert(item.id(), item);
    }

    pub fn get(&self, key: KeyType) -> Option<&T> {
        self.map.get(&key)
    }

    pub fn get_mut(&mut self, key: KeyType) -> Option<&mut T> {
        self.map.get_mut(&key)
    }
}

impl<T: InstanceID> IntoIterator for LookupTable<T> {
    type Item = (KeyType, T);

    type IntoIter = std::collections::hash_map::IntoIter<KeyType, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}

impl<'a, T: InstanceID> IntoIterator for &'a LookupTable<T> {
    type Item = (&'a KeyType, &'a T);

    type IntoIter = std::collections::hash_map::Iter<'a, KeyType, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter()
    }
}

impl<'a, T: InstanceID> IntoIterator for &'a mut LookupTable<T> {
    type Item = (&'a KeyType, &'a mut T);

    type IntoIter = std::collections::hash_map::IterMut<'a, KeyType, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.iter_mut()
    }
}
