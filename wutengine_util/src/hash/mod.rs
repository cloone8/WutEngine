//! Hashing utilities and algorithms

use core::hash::Hash;
use std::collections::HashMap;

pub use nohash_hasher;

struct KeywordHasher {
    val: u64,
}

impl KeywordHasher {
    const fn new() -> Self {
        Self { val: 0 }
    }
}

impl core::hash::Hasher for KeywordHasher {
    fn finish(&self) -> u64 {
        self.val
    }

    fn write(&mut self, bytes: &[u8]) {
        const U64_BYTES: usize = size_of::<u64>();

        let to_hash = bytes.len();
        let mut bytes_left = to_hash;

        while bytes_left >= U64_BYTES {
            let bytes_done = to_hash - bytes_left;
            let u64_slice = &bytes[bytes_done..(bytes_done + U64_BYTES)];

            let cur_int = u64::from_le_bytes(u64_slice.try_into().unwrap());

            self.val = self.val.wrapping_add(cur_int).rotate_right(1) ^ 0xBEEFFEEB;

            bytes_left -= U64_BYTES;
        }

        if bytes_left > 0 {
            debug_assert!(bytes_left < U64_BYTES, "Too many bytes left");

            let mut remaining_bytes: [u8; U64_BYTES] = [0; U64_BYTES];

            let remaining_input_bytes = &bytes[(to_hash - bytes_left)..];

            debug_assert_eq!(
                bytes_left,
                remaining_input_bytes.len(),
                "Invalid length calculated"
            );

            for (i, &byte) in remaining_input_bytes.iter().enumerate() {
                remaining_bytes[i] = byte;
            }

            self.val = self
                .val
                .wrapping_add(u64::from_le_bytes(remaining_bytes))
                .rotate_right(1)
                ^ 0xBEEFFEEB;
        }
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
pub fn keyword_hash<K: AsRef<str> + core::hash::Hash + Eq, V: core::hash::Hash>(
    keywords: &HashMap<K, V>,
) -> u64 {
    let mut hasher = KeywordHasher::new();

    let mut keywords_tuples: Vec<(&str, &V)> = keywords
        .iter()
        .map(|(key, val)| (key.as_ref(), val))
        .collect();

    keywords_tuples.sort_by_key(|(key, _)| *key);

    for (key, val) in keywords_tuples {
        key.hash(&mut hasher);
        val.hash(&mut hasher);
    }

    hasher.val
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::hash::keyword_hash;

    #[test]
    fn test_empty_keyword_hash() {
        assert_eq!(0, keyword_hash::<String, i64>(&HashMap::default()));
    }
}
