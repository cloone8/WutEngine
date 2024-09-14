use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

#[derive(Debug, Clone, Copy, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityId {
    id: usize,
}

impl EntityId {
    pub fn new() -> Self {
        let mut rng = SmallRng::from_entropy();
        Self { id: rng.gen() }
    }
}
