use glam::{Vec2, Vec3Swizzles};
use rapier2d::prelude::*;
use wutengine_macro::component_boilerplate;

use crate::builtins::components::Transform;
use crate::component::{Component, Context};
use crate::physics::physics2d::Physics2DPlugin;
use crate::physics::Collider2DID;

/// A 2D-physics rectangular collider
#[derive(Debug)]
pub struct RectangleCollider2D {
    center: Vec2,
    size: Vec2,

    /// The raw collider handle
    pub(crate) handle: Option<Collider2DID>,
}

impl RectangleCollider2D {
    /// Creates a new 2D rectangle collider component with the
    /// given initial state
    pub fn new(center: Vec2, size: Vec2) -> Self {
        Self {
            center,
            size,
            handle: None,
        }
    }
}

impl RectangleCollider2D {
    /// Returns the raw collider handle for this collider component,
    /// if it has already been created.
    pub fn get_handle(&self) -> Option<Collider2DID> {
        self.handle
    }
}

impl Component for RectangleCollider2D {
    component_boilerplate!();

    fn start(&mut self, context: &mut Context) {
        let physics_plugin = context.plugin.get::<Physics2DPlugin>();

        if physics_plugin.is_none() {
            log::warn!("Trying to initialize rectangle collider component failed because the physics plugin was missing");
            return;
        }

        let physics_plugin = physics_plugin.unwrap();

        let transform = context.gameobject.get_component::<Transform>();

        let world_center = match transform {
            Some(transform) => transform
                .local_to_world()
                .transform_point3(self.center.extend(0.0))
                .xy(),
            None => self.center,
        };

        let world_size = match transform {
            Some(transform) => transform.lossy_scale().xy() * self.size,
            None => self.size,
        };

        log::trace!(
            "Creating rectangle collider at {} with size {}",
            world_center,
            world_size
        );

        let collider = ColliderBuilder::cuboid(world_size.x / 2.0, world_size.y / 2.0)
            .translation(world_center.into())
            .active_events(ActiveEvents::COLLISION_EVENTS)
            .active_collision_types(ActiveCollisionTypes::all())
            .sensor(true)
            .build();

        let handle = physics_plugin.add_collider(collider, context.gameobject.object.id);

        self.handle = Some(handle);
    }

    fn post_physics_update(&mut self, context: &mut Context) {
        let physics_plugin = context.plugin.get::<Physics2DPlugin>();

        if physics_plugin.is_none() {
            return;
        }

        let physics_plugin = physics_plugin.unwrap();

        if let Some(handle) = self.handle {
            let transform = context.gameobject.get_component::<Transform>();

            let world_pos = match transform {
                Some(transform) => transform
                    .local_to_world()
                    .transform_point3(self.center.extend(0.0))
                    .xy(),
                None => self.center,
            };

            physics_plugin.update_collider(handle, world_pos);
        }
    }
}
