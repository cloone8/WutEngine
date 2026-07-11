//! Rigidbody types and API

use crate::rapier;
use crate::rapier::prelude::*;

#[derive(Debug)]
pub struct Rigidbody {
    handle: rapier::dynamics::RigidBodyHandle,
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
