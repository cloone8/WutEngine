//! 3D Physics logic

use rapier3d::prelude::*;

#[derive(Debug)]
pub(crate) struct Physics3DPlugin {}

impl Physics3DPlugin {
    pub(crate) fn new() -> Self {
        Self {}
    }
}
