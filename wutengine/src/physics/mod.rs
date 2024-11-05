//! Physics functionality. Both 2D and 3D.

pub(crate) mod physics2d;
pub(crate) mod physics3d;

pub use physics2d::*;
pub use physics3d::*;

/// A collision event type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionType {
    /// Collision has started
    Started,

    /// Collision has ended
    Stopped,
}
