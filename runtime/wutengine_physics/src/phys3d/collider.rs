//! Collider types and API

use super::ColliderPose;
use crate::RapierConversion;
use rapier3d::prelude::*;
use wutengine_util_macro::unique_id_type64;

unique_id_type64! {
    /// The handle to a single collider
    pub(crate) ColliderId
}

/// Handle to a raw collider in the physics world
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Collider(pub(crate) ColliderId);

/// Create a pose from the given pose data
pub(crate) fn make_pose(pose: ColliderPose) -> Pose3 {
    todo!()
    // Pose3::new(pose.0.to_rapier(), pose.1.to_rapier())
}
