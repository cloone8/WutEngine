//! Collider types and API

use crate::rapier;
use crate::rapier::prelude::*;
use wutengine_util_macro::unique_id_type64;

#[cfg(phys2d)]
pub use phys2d::*;

#[cfg(phys3d)]
pub use phys3d::*;

unique_id_type64! {
    /// The handle to a single collider
    pub(crate) ColliderId
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Collider(pub(crate) ColliderId);

#[cfg(phys2d)]
mod phys2d {
    use crate::RapierConversion;
    use crate::rapier;
    use crate::types::ColliderPose;

    use rapier::prelude::*;
    use wutengine_math::*;

    #[derive(Debug)]
    pub enum ColliderData2D {
        Cube {
            offset: Vec2,
            rotation: f32,
            trigger: bool,
            x: f32,
            y: f32,
        },
    }

    impl Default for ColliderData2D {
        fn default() -> Self {
            Self::Cube {
                offset: Vec2::ZERO,
                rotation: 0.0,
                trigger: false,
                x: 1.0,
                y: 1.0,
            }
        }
    }

    impl ColliderData2D {
        pub fn create(
            &self,
            local_to_world_offset: Vec2,
            local_to_world_rot: f32,
        ) -> ColliderBuilder {
            match self {
                Self::Cube {
                    offset,
                    rotation,
                    trigger,
                    x,
                    y,
                } => ColliderBuilder::cuboid(x * 0.5, y * 0.5)
                    .position(Pose2::new(
                        (offset + local_to_world_offset).to_rapier(),
                        (rotation + local_to_world_rot).to_radians(),
                    ))
                    .sensor(*trigger),
            }
        }

        pub const fn offset_rot(&self) -> (Vec2, f32) {
            match self {
                Self::Cube {
                    offset, rotation, ..
                } => (*offset, *rotation),
            }
        }
    }

    pub(crate) fn make_pose(pose: ColliderPose) -> Pose2 {
        Pose2::new(pose.0.to_rapier(), pose.1.to_radians())
    }
}

#[cfg(phys3d)]
mod phys3d {
    use crate::RapierConversion;
    use crate::rapier;
    use crate::types::ColliderPose;

    use rapier::prelude::*;
    use wutengine_math::*;

    pub(crate) fn make_pose(pose: ColliderPose) -> Pose3 {
        todo!()
        // Pose3::new(pose.0.to_rapier(), pose.1.to_rapier())
    }
}
