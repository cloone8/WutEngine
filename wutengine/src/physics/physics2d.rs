use core::fmt::Debug;

use glam::{vec2, Vec2};
use rapier2d::prelude::*;

/// A 2D physics pipeline
#[derive(Debug)]
pub(crate) struct Physics2D {
    rapier: RapierStructs2D,
}

struct RapierStructs2D {
    rigids: RigidBodySet,
    colliders: ColliderSet,
    physics_pipeline: PhysicsPipeline,
    parameters: IntegrationParameters,
    gravity: Vec2,
    island_manager: IslandManager,
    broad: DefaultBroadPhase,
    narrow: NarrowPhase,
    impulse_joints: ImpulseJointSet,
    multibody_joints: MultibodyJointSet,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,
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

impl Physics2D {
    /// Creates and initializes a new 2D physics pipeline
    pub(crate) fn new() -> Self {
        Self {
            rapier: RapierStructs2D {
                rigids: RigidBodySet::new(),
                colliders: ColliderSet::new(),
                physics_pipeline: PhysicsPipeline::new(),
                parameters: IntegrationParameters::default(),
                gravity: vec2(0.0, -9.81),
                island_manager: IslandManager::new(),
                broad: DefaultBroadPhase::new(),
                narrow: NarrowPhase::new(),
                impulse_joints: ImpulseJointSet::new(),
                multibody_joints: MultibodyJointSet::new(),
                ccd_solver: CCDSolver::new(),
                query_pipeline: QueryPipeline::new(),
            },
        }
    }

    /// Adds a raw collider to the solver
    pub(crate) fn add_collider(&mut self, collider: Collider) -> ColliderHandle {
        self.rapier.colliders.insert(collider)
    }
}
