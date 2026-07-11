//! Collider types and API

use wutengine_util_macro::unique_id_type64;

unique_id_type64! {
    /// The handle to a single collider
    pub(crate) ColliderId
}

/// Handle to a raw collider in the physics world
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Collider(pub(crate) ColliderId);

use super::ColliderPose;
use crate::RapierConversion;

use rapier2d::prelude::*;
use wutengine_math::*;

/// Data to a 2D collider
#[derive(Debug, Clone)]
pub struct ColliderData2D {
    /// Offset of the collider in local space, with regards to its containing entity
    pub offset: Vec2,

    /// Rotation (in degrees) of the collider in local space in the X axis, with regards to its containing entity
    pub rotation: f32,

    /// Whether this collider is a trigger
    pub trigger: bool,

    /// The type-specific data
    pub type_data: ColliderType2D,
}

/// Collider-type specific data
#[derive(Debug, Clone)]
pub enum ColliderType2D {
    /// Cube collider
    Cube {
        /// Width
        x: f32,

        /// Height
        y: f32,
    },
}

impl Default for ColliderType2D {
    fn default() -> Self {
        Self::Cube { x: 1.0, y: 1.0 }
    }
}

impl Default for ColliderData2D {
    fn default() -> Self {
        Self {
            offset: Vec2::ZERO,
            rotation: 0.0,
            trigger: false,
            type_data: ColliderType2D::default(),
        }
    }
}

impl ColliderData2D {
    /// Create a [ColliderBuilder] from this data
    pub fn create(&self, local_to_world_offset: Vec2, local_to_world_rot: f32) -> ColliderBuilder {
        let mut builder = match self.type_data {
            ColliderType2D::Cube { x, y } => ColliderBuilder::cuboid(x * 0.5, y * 0.5),
        };

        builder = builder
            .position(Pose2::new(
                (self.offset + local_to_world_offset).to_rapier(),
                (self.rotation + local_to_world_rot).to_radians(),
            ))
            .sensor(self.trigger);

        builder
    }
}

/// Create a pose from the given pose data
pub(crate) fn make_pose(pose: ColliderPose) -> Pose2 {
    Pose2::new(pose.0.to_rapier(), pose.1.to_radians())
}
