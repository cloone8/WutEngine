use crate::builtins::components::Transform;
use crate::component::Component;

use crate::math::*;
use wutengine_physics2d::PhysicsWorldUpdater;
use wutengine_physics2d::collider::ColliderData2D;

/// A set of colliders
#[derive(Debug, Default)]
pub struct ColliderSet2D {
    colliders: Vec<Collider2D>,
}

impl ColliderSet2D {
    /// Adds a new collider to this set
    pub fn add_collider(&mut self, collider: ColliderData2D) {
        self.colliders.push(Collider2D::new(collider));
    }
}

#[derive(Debug, Default)]
struct Collider2D {
    handle: Option<crate::physics2d::collider::Collider>,
    last_pos_rot: (Vec2, f32),
    data: ColliderData2D,
}

impl Collider2D {
    fn new(data: ColliderData2D) -> Self {
        Self {
            handle: None,
            last_pos_rot: (Vec2::ZERO, 0.0),
            data,
        }
    }
    fn recreate_collider(
        &mut self,
        transform: Option<&Transform>,
        physics_updater: &mut PhysicsWorldUpdater,
    ) {
        self.handle = None;

        let (pos, rot) = Self::calc_pos_rot(transform);

        self.handle = Some(physics_updater.add_collider(self.data.create(pos, rot)));

        self.last_pos_rot = (pos, rot);
    }

    fn calc_pos_rot(transform: Option<&Transform>) -> (Vec2, f32) {
        let Some(transform) = transform else {
            return (Vec2::ZERO, 0.0);
        };

        let up = Vec3::Y;
        let after_rot = transform.world_rotation() * up;
        let without_z = after_rot.with_z(0.0).normalize();
        let angle_pos = after_rot.angle_between(without_z);
        let angle = if without_z.x > 0.0 {
            angle_pos
        } else {
            -angle_pos
        };

        (transform.world_position().xy(), angle.to_degrees())
    }

    fn update_pos_rot(
        &mut self,
        transform: Option<&Transform>,
        physics_updater: &mut PhysicsWorldUpdater,
    ) {
        let Some(handle) = self.handle.as_ref() else {
            return;
        };

        let (pos, rot) = Self::calc_pos_rot(transform);

        if self.last_pos_rot == (pos, rot) {
            return;
        }

        physics_updater.move_collider(handle, (pos + self.data.offset, rot + self.data.rotation));

        self.last_pos_rot = (pos, rot);
    }
}

impl ColliderSet2D {
    /// Syncs all colliders in this set to the physics world using the given [PhysicsWorldUpdater]
    pub(crate) fn sync_to_physics_world(
        &mut self,
        transform: Option<&Transform>,
        physics_updater: &mut PhysicsWorldUpdater,
    ) {
        for collider in self.colliders.iter_mut() {
            if collider.handle.is_none() {
                collider.recreate_collider(transform, physics_updater);
            }

            collider.update_pos_rot(transform, physics_updater);
        }
    }
}

impl Component for ColliderSet2D {
    const ID: uuid::NonNilUuid =
        uuid::NonNilUuid::new(uuid::uuid!("f716d2f8-024e-43df-bcc7-a06e0f984f14")).unwrap();
}
