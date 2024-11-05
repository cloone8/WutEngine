use rapier2d::prelude::*;

/// An ID for a 2D collider
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
#[repr(transparent)]
pub struct Collider2DID {
    /// The raw [rapier2d] handle
    pub(crate) raw: ColliderHandle,
}

impl Collider2DID {
    /// Creates a new [Collider2DID] for a given raw handle.
    ///
    /// For two equal handles, will always produce two equal [Collider2DID] instances.
    #[inline]
    pub(crate) fn new(raw: ColliderHandle) -> Self {
        Self { raw }
    }
}
