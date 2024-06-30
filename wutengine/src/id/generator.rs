use rand::{
    distributions::{Distribution, Uniform},
    rngs::SmallRng,
    SeedableRng,
};

use super::KeyType;

pub trait IdGenerator<const BITS: usize> {
    fn get(&mut self) -> KeyType;
}

const fn bit_mask<const BITS: usize>() -> KeyType {
    KeyType::MAX >> ((KeyType::BITS as usize) - BITS)
}

#[derive(Debug)]
pub struct RandomIdGenerator<const BITS: usize> {
    rng: SmallRng,
    distrib: Uniform<KeyType>,
}

impl<const BITS: usize> RandomIdGenerator<BITS> {
    pub fn new() -> Self {
        Self {
            rng: SmallRng::from_entropy(),
            distrib: Uniform::new_inclusive(0, bit_mask::<BITS>()),
        }
    }
}

impl<const BITS: usize> IdGenerator<BITS> for RandomIdGenerator<BITS> {
    fn get(&mut self) -> KeyType {
        self.distrib.sample(&mut self.rng)
    }
}

#[derive(Debug)]
pub struct ConsecutiveIdGenerator<const BITS: usize> {
    next: KeyType,
}

impl<const BITS: usize> ConsecutiveIdGenerator<BITS> {
    pub fn new() -> Self {
        Self { next: 0 }
    }
}

impl<const BITS: usize> IdGenerator<BITS> for ConsecutiveIdGenerator<BITS> {
    fn get(&mut self) -> KeyType {
        debug_assert_ne!(bit_mask::<BITS>(), self.next);

        self.next += 1;
        self.next
    }
}
