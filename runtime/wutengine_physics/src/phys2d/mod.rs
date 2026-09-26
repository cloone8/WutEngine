//! 2D physics, backed by `rapier2d`

use rapier2d as rapier;
use wutengine_math::Vec2 as EngineVector;

pub mod collider;
#[path = "../shared/rigidbody.rs"]
pub mod rigidbody;
#[path = "../shared/world.rs"]
mod world;

pub(crate) use world::PhysicsManager;
pub use world::PhysicsWorldUpdater;

/// World-space position and rotation (in degrees) of a collider
pub(crate) type ColliderPose = (EngineVector, f32);
