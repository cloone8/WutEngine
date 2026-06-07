//! 3D physics

use rapier3d::prelude::CCDSolver;
use rapier3d::prelude::ColliderSet;
use rapier3d::prelude::DefaultBroadPhase;
use rapier3d::prelude::ImpulseJointSet;
use rapier3d::prelude::IntegrationParameters;
use rapier3d::prelude::IslandManager;
use rapier3d::prelude::MultibodyJointSet;
use rapier3d::prelude::NarrowPhase;
use rapier3d::prelude::PhysicsPipeline;
use rapier3d::prelude::RigidBodySet;
use rapier3d::prelude::SharedShape;
use wutengine_math::Vec3;

use crate::RapierConversion;

pub(crate) struct Phys3DManager {
    gravity: Vec3,
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

impl Phys3DManager {
    pub(super) fn new() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
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

        log::info!("Stepping 3D physics with dt: {dt}");

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

impl RapierConversion<rapier3d::math::Vec3> for Vec3 {
    #[inline(always)]
    fn to_rapier(self) -> rapier3d::math::Vec3 {
        rapier3d::math::Vec3::new(self.x, self.y, self.z)
    }

    #[inline(always)]
    fn from_rapier(val: rapier3d::math::Vec3) -> Self {
        Self::new(val.x, val.y, val.z)
    }
}
