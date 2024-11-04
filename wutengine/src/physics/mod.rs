//! Physics functionality. Both 2D and 3D.

use std::any::Any;

use physics2d::Physics2D;
use physics3d::Physics3D;

use crate::plugins::WutEnginePlugin;

mod physics2d;
mod physics3d;

/// The WutEngine physics plugin, responsible for
/// handling the interaction for the raw physics backend.
#[derive(Debug)]
pub struct PhysicsPlugin {
    physics_2d: Option<Physics2D>,
    physics_3d: Option<Physics3D>,
}

impl PhysicsPlugin {
    /// Creates a new, empty physics plugin
    pub fn new() -> Self {
        Self {
            physics_2d: None,
            physics_3d: None,
        }
    }
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
}
