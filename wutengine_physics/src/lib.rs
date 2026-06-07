#![cfg_attr(phys2d, doc = include_str!("../../wutengine_physics2d/README.md"))]
#![cfg_attr(phys3d, doc = include_str!("../../wutengine_physics3d/README.md"))]

use std::sync::RwLock;

use wutengine_util::InitOnce;

#[cfg(all(phys2d, phys3d))]
compile_error!("Cannot enable both 2D and 3D physics");

#[cfg(phys2d)]
use rapier2d as rapier;

#[cfg(phys3d)]
use rapier3d as rapier;

use rapier::prelude::*;

#[cfg(phys2d)]
mod types {
    #![allow(clippy::missing_docs_in_private_items, reason = "Type aliases only")]
    //! Dynamic type aliases. Used to abstract over 2D/3D physics

    pub(crate) type VecX = wutengine_math::Vec2;
}

#[cfg(phys3d)]
mod types {
    #![allow(clippy::missing_docs_in_private_items, reason = "Type aliases only")]
    //! Dynamic type aliases. Used to abstract over 2D/3D physics

    pub(crate) type VecX = wutengine_math::Vec3;
}

use types::*;

/// Global physics manager
pub(crate) static PHYSICS_MANAGER: InitOnce<RwLock<PhysicsManager>> = InitOnce::new();

/// Initialize the physics system
#[doc(hidden)]
pub fn init() {
    InitOnce::init(&PHYSICS_MANAGER, RwLock::new(PhysicsManager::new()));
}

/// Runs the physics simulation for one frame
pub fn step(dt: f32) {
    profiling::function_scope!();

    log::debug!("Stepping simulation with dt: {dt}");

    PHYSICS_MANAGER.write().unwrap().step(dt);
}

/// Physics manager
pub(crate) struct PhysicsManager {
    gravity: VecX,
    rigidbody_set: RigidBodySet,
    collider_set: ColliderSet,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
}

impl PhysicsManager {
    /// Create a new, empty, physics manager
    fn new() -> Self {
        PhysicsManager {
            gravity: VecX::ZERO.with_y(-9.81),
            rigidbody_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
        }
    }

    /// Step the physics simulation by a given delta-time
    fn step(&mut self, dt: f32) {
        profiling::function_scope!();

        self.integration_parameters.dt = dt;

        let (collision_send, collision_recv) = std::sync::mpsc::channel();
        let (contact_force_send, contact_force_recv) = std::sync::mpsc::channel();
        let event_handler = ChannelEventCollector::new(collision_send, contact_force_send);

        self.physics_pipeline.step(
            self.gravity.to_rapier(),
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigidbody_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            &(),
            &event_handler,
        );
    }
}

/// Easier inline rapier-wutengine type conversion
trait RapierConversion<T> {
    /// Convert to rapier
    fn to_rapier(self) -> T;

    /// Convert from rapier
    fn from_rapier(val: T) -> Self;
}

impl RapierConversion<rapier::math::Vector> for VecX {
    #[inline(always)]
    fn to_rapier(self) -> rapier::math::Vector {
        rapier::math::Vector::from_array(self.to_array())
    }

    #[inline(always)]
    fn from_rapier(val: rapier::math::Vector) -> Self {
        Self::from_array(val.to_array())
    }
}
