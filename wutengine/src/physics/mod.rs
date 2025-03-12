//! Physics functionality. Both 2D and 3D.

pub(crate) mod physics2d;
pub(crate) mod physics3d;

use glam::Vec2;
pub use physics2d::*;

#[expect(unused_imports, unreachable_pub)]
pub use physics3d::*;

use crate::gameobject::GameObjectId;

/// Event fired when a collision with another object has started
#[derive(Debug, Clone)]
pub struct CollisionStart {
    /// The [crate::gameobject::GameObject] that we collided with
    pub other: GameObjectId,

    /// The position of the other [crate::gameobject::GameObject] upon collision
    pub other_pos: Vec2,
}

/// Event fired when a collision with another object has ended
#[derive(Debug, Clone)]
pub struct CollisionEnd {
    /// The [crate::gameobject::GameObject] that we collided with
    pub other: GameObjectId,
}
