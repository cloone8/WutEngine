use crate::builtins::components::Transform;

pub enum ColliderSet3D {}

impl ColliderSet3D {
    pub(crate) fn sync_to_physics_world(
        &mut self,
        transform: Option<&Transform>,
        physics_updater: &mut crate::physics::phys3d::PhysicsWorldUpdater,
    ) {
    }
}
