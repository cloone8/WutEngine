//! Contains a wrapper around a [nohash_hasher::NoHashHasher] that hashes with a stride constistent with the shard stride of a [dashmap::DashMap].
//! This has the advantage that consecutive integers are always in different shards, meaning lower contention when generating keys with steps of one.

use core::marker::PhantomData;
use std::sync::OnceLock;

use nohash_hasher::NoHashHasher;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ShardHasher<T> {
    rot: u32,
    hasher: NoHashHasher<u32>,
    ph: PhantomData<T>,
}

fn guess_shard_count() -> usize {
    static DEFAULT_SHARD_AMOUNT: OnceLock<usize> = OnceLock::new();

    *DEFAULT_SHARD_AMOUNT.get_or_init(|| {
        let threads = std::thread::available_parallelism().map_or(8, usize::from);
        shard_count_threads(threads)
    })
}

const fn shard_count_threads(num_threads: usize) -> usize {
    (num_threads * 4).next_power_of_two()
}

const fn calc_shift(shard_count: usize) -> u32 {
    usize::BITS - shard_count.trailing_zeros()
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct BuildShardHasher<T> {
    rot: u32,
    ph: PhantomData<T>,
}

impl<T> BuildShardHasher<T> {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn new_for_shards(shard_count: usize) -> Self {
        Self {
            rot: calc_shift(shard_count),
            ph: PhantomData,
        }
    }

    pub(crate) fn new_for_threads(thread_count: usize) -> Self {
        Self {
            rot: calc_shift(shard_count_threads(thread_count)),
            ph: PhantomData,
        }
    }
}

impl<T> Default for BuildShardHasher<T> {
    fn default() -> Self {
        Self {
            rot: calc_shift(guess_shard_count()) - 7,
            ph: PhantomData,
        }
    }
}

impl<T: nohash_hasher::IsEnabled> core::hash::BuildHasher for BuildShardHasher<T> {
    type Hasher = ShardHasher<T>;

    fn build_hasher(&self) -> Self::Hasher {
        ShardHasher {
            rot: self.rot,
            hasher: NoHashHasher::default(),
            ph: PhantomData,
        }
    }
}

impl<T: nohash_hasher::IsEnabled> core::hash::Hasher for ShardHasher<T> {
    #[inline(always)]
    fn finish(&self) -> u64 {
        self.hasher.finish().rotate_left(self.rot)
    }

    #[inline(always)]
    fn write(&mut self, bytes: &[u8]) {
        self.hasher.write(bytes);
    }

    #[inline(always)]
    fn write_u8(&mut self, i: u8) {
        self.hasher.write_u8(i);
    }

    #[inline(always)]
    fn write_u16(&mut self, i: u16) {
        self.hasher.write_u16(i);
    }

    #[inline(always)]
    fn write_u32(&mut self, i: u32) {
        self.hasher.write_u32(i);
    }

    #[inline(always)]
    fn write_u64(&mut self, i: u64) {
        self.hasher.write_u64(i);
    }

    #[inline(always)]
    fn write_u128(&mut self, i: u128) {
        self.hasher.write_u128(i);
    }

    #[inline(always)]
    fn write_usize(&mut self, i: usize) {
        self.hasher.write_usize(i);
    }

    #[inline(always)]
    fn write_i8(&mut self, i: i8) {
        self.hasher.write_i8(i);
    }

    #[inline(always)]
    fn write_i16(&mut self, i: i16) {
        self.hasher.write_i16(i);
    }

    #[inline(always)]
    fn write_i32(&mut self, i: i32) {
        self.hasher.write_i32(i);
    }

    #[inline(always)]
    fn write_i64(&mut self, i: i64) {
        self.hasher.write_i64(i);
    }

    #[inline(always)]
    fn write_i128(&mut self, i: i128) {
        self.hasher.write_i128(i);
    }

    #[inline(always)]
    fn write_isize(&mut self, i: isize) {
        self.hasher.write_isize(i);
    }
}
