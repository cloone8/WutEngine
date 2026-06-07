//! 2D physics

use rapier2d::prelude::CCDSolver;
use rapier2d::prelude::ColliderSet;
use rapier2d::prelude::DefaultBroadPhase;
use rapier2d::prelude::ImpulseJointSet;
use rapier2d::prelude::IntegrationParameters;
use rapier2d::prelude::IslandManager;
use rapier2d::prelude::MultibodyJointSet;
use rapier2d::prelude::NarrowPhase;
use rapier2d::prelude::PhysicsPipeline;
use rapier2d::prelude::RigidBodySet;
use wutengine_math::Vec2;

use crate::RapierConversion;

pub(crate) struct Phys2DManager {
    gravity: Vec2,
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

impl Phys2DManager {
    pub(crate) fn new() -> Self {
        Self {
            gravity: Vec2::new(0.0, -9.81),
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

    pub(crate) fn step(&mut self, dt: f32) {
        profiling::function_scope!();

        log::info!("Stepping 2D physics with dt: {dt}");

        self.integration_parameters.dt = dt;

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
            &(),
        );
    }
}

impl RapierConversion<rapier2d::math::Vec2> for Vec2 {
    #[inline(always)]
    fn to_rapier(self) -> rapier2d::math::Vec2 {
        rapier2d::math::Vec2::new(self.x, self.y)
    }

    #[inline(always)]
    fn from_rapier(val: rapier2d::math::Vec2) -> Self {
        Self::new(val.x, val.y)
    }
}
