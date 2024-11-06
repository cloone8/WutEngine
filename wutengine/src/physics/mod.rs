//! Physics functionality. Both 2D and 3D.

pub(crate) mod physics2d;
pub(crate) mod physics3d;

use glam::Vec2;
pub use physics2d::*;
pub use physics3d::*;

use crate::gameobject::GameObjectId;

/// Event fired when a collision with another object has started
#[derive(Debug, Clone)]
pub struct CollisionStart {
    pub other: GameObjectId,
    pub other_pos: Vec2,
}

/// Event fired when a collision with another object has ended
#[derive(Debug, Clone)]
pub struct CollisionEnd {
    pub other: GameObjectId,
}
