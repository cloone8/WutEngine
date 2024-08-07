use core::hash::{Hash, Hasher};

use nohash_hasher::IsEnabled;
use rand::{rngs::SmallRng, RngCore, SeedableRng};

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityId(u64);

impl Hash for EntityId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.0);
    }
}

impl IsEnabled for EntityId {}

impl EntityId {
    pub fn random() -> Self {
        let mut rng = SmallRng::from_entropy();
        Self(rng.next_u64())
    }
}
