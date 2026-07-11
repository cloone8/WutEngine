//! Rigidbody types and API

use rapier2d::prelude::*;

#[derive(Debug)]
pub struct Rigidbody {
    handle: rapier2d::dynamics::RigidBodyHandle,
}

impl Rigidbody {
    pub fn destroy(self) {
        core::mem::drop(self);
    }
}

impl Drop for Rigidbody {
    fn drop(&mut self) {
        todo!()
    }
}
