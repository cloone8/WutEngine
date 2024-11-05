//! Physics functionality. Both 2D and 3D.

use std::any::Any;
use std::sync::Mutex;

use glam::Vec2;
use physics2d::Physics2D;
use physics3d::Physics3D;

use crate::plugins::{Context, WutEnginePlugin};
use crate::time::Time;

mod physics2d;
mod physics3d;

#[doc(inline)]
pub use rapier2d as raw_2d;

#[doc(inline)]
pub use rapier3d as raw_3d;

/// The WutEngine physics plugin, responsible for
/// handling the interaction for the raw physics backend.
#[derive(Debug)]
pub struct PhysicsPlugin {
    physics_2d: Mutex<Option<Physics2D>>,
    physics_3d: Mutex<Option<Physics3D>>,
}

impl PhysicsPlugin {
    /// Creates a new, empty physics plugin
    pub fn new() -> Self {
        Self {
            physics_2d: Mutex::new(None),
            physics_3d: Mutex::new(None),
        }
    }

    /// Adds a raw 2D collider to the 2D physics solver
    pub fn add_collider_2d(
        &self,
        collider: rapier2d::geometry::Collider,
    ) -> rapier2d::geometry::ColliderHandle {
        let mut locked = self.physics_2d.lock().unwrap();
        if locked.is_none() {
            *locked = Some(Physics2D::new());
        }

        let handle = locked.as_mut().unwrap().add_collider(collider);

        handle
    }

    pub fn update_collider_2d(
        &self,
        collider: rapier2d::geometry::ColliderHandle,
        translation: Vec2,
    ) {
        let mut locked = self.physics_2d.lock().unwrap();
        let physics = locked.as_mut().unwrap();

        physics.update_collider(collider, translation);
    }

    pub fn add_collider_3d(&self, collider: rapier3d::geometry::Collider) {}
}

impl Default for PhysicsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl WutEnginePlugin for PhysicsPlugin {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn post_physics_update(&mut self, _context: &mut Context) {
        // First, we step the physics solvers

        let mut locked = self.physics_2d.lock().unwrap();

        if let Some(physics2d) = &mut *locked {
            physics2d.step(Time::get().fixed_delta);
        }
    }
}
