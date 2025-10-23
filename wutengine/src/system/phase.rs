//! [SystemPhase] related functionality

use core::fmt::Display;

use wutengine_util::VariantCount;
use wutengine_util::hash::nohash_hasher;

/// A phase in which a system can run
#[derive(Debug, Clone, Copy, PartialEq, Eq, VariantCount)]
#[repr(u8)]
pub enum SystemPhase {
    /// Run on fixed update
    PhysicsUpdate,

    /// Run on update
    Update,

    /// Run right before rendering
    PreRender,
}

impl Display for SystemPhase {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SystemPhase::PhysicsUpdate => "PhysicsUpdate".fmt(f),
            SystemPhase::Update => "Update".fmt(f),
            SystemPhase::PreRender => "PreRender".fmt(f),
        }
    }
}

impl core::hash::Hash for SystemPhase {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
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
