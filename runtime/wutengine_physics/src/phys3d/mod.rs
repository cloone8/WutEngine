//! 3D physics, backed by `rapier3d`

use rapier3d as rapier;
use wutengine_math::Vec3 as EngineVector;

pub mod collider;
#[path = "../shared/rigidbody.rs"]
#[cfg_attr(
    feature = "phys2d",
    expect(
        clippy::duplicate_mod,
        reason = "Shared with phys2d on purpose, compiled against rapier3d here"
    )
)]
pub mod rigidbody;
#[path = "../shared/world.rs"]
#[cfg_attr(
    feature = "phys2d",
    expect(
        clippy::duplicate_mod,
        reason = "Shared with phys2d on purpose, compiled against rapier3d here"
    )
)]
mod world;

pub(crate) use world::PhysicsManager;
pub use world::PhysicsWorldUpdater;

/// World-space position and rotation of a collider
pub(crate) type ColliderPose = (EngineVector, wutengine_math::Quat);
