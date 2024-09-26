use core::fmt::Display;
use core::hash::{Hash, Hasher};

use nohash_hasher::IsEnabled;
use rand::{rngs::SmallRng, RngCore, SeedableRng};

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityId(u64);

impl Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", self.0)
    }
}

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
