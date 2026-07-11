/// Alias for a [hashbrown::HashMap] using the [nohash_hasher] hashing algorithm
pub type IntMap<K, V> = hashbrown::HashMap<K, V, nohash_hasher::BuildNoHashHasher<K>>;

/// Alias for a [hashbrown::HashSet] using the [nohash_hasher] hashing algorithm
pub type IntSet<K> = hashbrown::HashSet<K, nohash_hasher::BuildNoHashHasher<K>>;
