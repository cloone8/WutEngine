use glam::{Vec2, Vec3Swizzles};
use wutengine_macro::component_boilerplate;

use crate::builtins::components::Transform;
use crate::component::{Component, Context};
use crate::physics::PhysicsPlugin;

/// A 2D-physics rectangular collider
#[derive(Debug)]
pub struct RectangleCollider2D {
    center: Vec2,
    size: Vec2,
}

impl RectangleCollider2D {
    /// Creates a new 2D rectangle collider component with the
    /// given initial state
    pub fn new(center: Vec2, size: Vec2) -> Self {
        Self { center, size }
    }
}

impl Component for RectangleCollider2D {
    component_boilerplate!();

    fn start(&mut self, context: &mut Context) {
        let physics_plugin = context.plugin.get::<PhysicsPlugin>();

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

        log::info!(
            "Creating rectangle collider at {} with size {}",
            world_center,
            world_size
        );
    }
}
