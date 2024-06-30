use glam::{Quat, Vec3};

#[derive(Debug)]
pub struct Transform {
    local_pos: Vec3,
    local_rot: Quat,
    local_scale: Vec3,
}

impl Transform {
    pub fn new(pos: Vec3, rot: Quat, scale: Vec3) -> Self {
        Self {
            local_pos: pos,
            local_rot: rot,
            local_scale: scale,
        }
    }

    pub fn from_local(local: LocalTransform) -> Self {
        Self {
            local_pos: local.pos,
            local_rot: local.rot,
            local_scale: local.scale,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LocalTransform {
    pub pos: Vec3,
    pub rot: Quat,
    pub scale: Vec3,
}
