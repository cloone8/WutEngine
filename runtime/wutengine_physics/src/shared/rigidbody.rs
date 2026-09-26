//! Rigidbody types and API

use super::rapier::prelude::*;

/// Handle to a rigidbody in the physics world
#[derive(Debug)]
pub struct Rigidbody {
    /// The rapier handle
    #[expect(dead_code, reason = "Rigidbodies are not implemented yet")]
    handle: RigidBodyHandle,
}

impl Rigidbody {
    /// Removes this rigidbody from the physics world
    pub fn destroy(self) {
        core::mem::drop(self);
    }
}

impl Drop for Rigidbody {
    #[expect(clippy::todo, reason = "Rigidbodies are not implemented yet")]
    fn drop(&mut self) {
        todo!()
    }
}
