use core::fmt::Display;

use wutengine_util::VariantCount;
use wutengine_util::hash::nohash_hasher;

#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantCount)]
#[repr(u8)]
pub enum SystemPhase {
    Update,
    FixedUpdate,
}

impl Display for SystemPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SystemPhase::Update => "Update".fmt(f),
            SystemPhase::FixedUpdate => "FixedUpdate".fmt(f),
        }
    }
}

impl core::hash::Hash for SystemPhase {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        const {
            assert!(
                size_of::<u8>() == size_of::<Self>(),
                "Size mismatch. Programmer error"
            )
        }

        let as_int = *self as u8;
        as_int.hash(state);
    }
}

impl nohash_hasher::IsEnabled for SystemPhase {}
