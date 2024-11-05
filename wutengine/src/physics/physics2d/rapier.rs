use core::fmt::Debug;

use glam::Vec2;
use rapier2d::prelude::*;

pub(super) struct RapierStructs2D {
    pub(super) rigids: RigidBodySet,
    pub(super) colliders: ColliderSet,
    pub(super) physics_pipeline: PhysicsPipeline,
    pub(super) parameters: IntegrationParameters,
    pub(super) gravity: Vec2,
    pub(super) island_manager: IslandManager,
    pub(super) broad: DefaultBroadPhase,
    pub(super) narrow: NarrowPhase,
    pub(super) impulse_joints: ImpulseJointSet,
    pub(super) multibody_joints: MultibodyJointSet,
    pub(super) ccd_solver: CCDSolver,
    pub(super) query_pipeline: QueryPipeline,
}

impl Debug for RapierStructs2D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RapierStructs2D")
            .field("rigids", &self.rigids)
            .field("colliders", &self.colliders)
            .field("physics_pipeline", &"<no debug>")
            .field("parameters", &self.parameters)
            .field("gravity", &self.gravity)
            .field("island_manager", &"<no debug>")
            .field("broad", &"<no debug>")
            .field("narrow", &"<no debug>")
            .field("impulse_joints", &self.impulse_joints)
            .field("multibody_joints", &self.multibody_joints)
            .field("ccd_solver", &"<no debug>")
            .field("query_pipeline", &"<no debug>")
            .finish()
    }
}
