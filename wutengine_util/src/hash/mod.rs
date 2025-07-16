//! Hashing utilities and algorithms

use core::hash::Hash;
use std::collections::HashMap;

use md5::{Digest, Md5};
pub use nohash_hasher;

/// [core::hash::Hasher] wrapper for an MD5 hasher
#[derive(Debug)]
struct Md5Hasher(Md5);

impl core::hash::Hasher for Md5Hasher {
    fn finish(&self) -> u64 {
        unimplemented!(
            "Do not use the standard finish method. MD5 is not means as a default hashmap algorithm"
        );
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }

    fn write_u8(&mut self, i: u8) {
        self.write(&[i])
    }

    fn write_u16(&mut self, i: u16) {
        self.write(&i.to_le_bytes())
    }

    fn write_u32(&mut self, i: u32) {
        self.write(&i.to_le_bytes())
    }

    fn write_u64(&mut self, i: u64) {
        self.write(&i.to_le_bytes())
    }

    fn write_u128(&mut self, i: u128) {
        self.write(&i.to_le_bytes())
    }

    fn write_usize(&mut self, i: usize) {
        self.write(&i.to_le_bytes())
    }
}

/// Provides a stable hash for the given set of keywords and values
pub fn keyword_hash<V: core::hash::Hash, K: AsRef<str> + core::hash::Hash + Eq>(
    keywords: &HashMap<K, V>,
) -> u128 {
    let mut hasher = Md5Hasher(Md5::new());

    let mut keywords_tuples: Vec<(&str, &V)> = keywords
        .iter()
        .map(|(key, val)| (key.as_ref(), val))
        .collect();

    keywords_tuples.sort_by_key(|(key, _)| *key);

    for (key, val) in keywords_tuples {
        key.hash(&mut hasher);
        val.hash(&mut hasher);
    }

    u128::from_le_bytes(hasher.0.finalize().into())
}
