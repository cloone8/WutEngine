use crate::math::Vec3;
use glam::{Mat4, Quat};
use wutengine_core::Component;

//TODO: Will probably be heavily modified once we get into transform hierarchies, 3D transformations, etc.
#[derive(Debug)]
pub struct Transform {
    pub(crate) data: TransformData,
}

#[derive(Debug)]
pub(crate) struct TransformData {
    pub pos: Vec3,
    pub scale: Vec3,
    pub rot: Quat,
}

impl Default for TransformData {
    fn default() -> Self {
        Self {
            pos: Vec3::ZERO,
            scale: Vec3::ONE,
            rot: Quat::IDENTITY,
        }
    }
}

impl Transform {
    #[expect(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            data: TransformData::default(),
        }
    }

    pub fn with_pos(pos: Vec3) -> Self {
        Self {
            data: TransformData {
                pos,
                ..TransformData::default()
            },
        }
    }

    pub fn with_rot(rot: Quat) -> Self {
        Self {
            data: TransformData {
                rot,
                ..TransformData::default()
            },
        }
    }

    pub fn with_scale(scale: Vec3) -> Self {
        Self {
            data: TransformData {
                scale,
                ..TransformData::default()
            },
        }
    }

    pub fn with_pos_rot(pos: Vec3, rot: Quat) -> Self {
        Self {
            data: TransformData {
                pos,
                rot,
                ..TransformData::default()
            },
        }
    }

    pub fn with_pos_rot_scale(pos: Vec3, rot: Quat, scale: Vec3) -> Self {
        Self {
            data: TransformData { pos, rot, scale },
        }
    }

    pub fn world_pos(&self) -> Vec3 {
        self.data.pos
    }

    pub fn set_world_pos(&mut self, pos: Vec3) {
        self.data.pos = pos;
    }

    pub fn local_pos(&self) -> Vec3 {
        self.data.pos
    }

    pub fn set_local_pos(&mut self, pos: Vec3) {
        self.data.pos = pos;
    }

    pub fn world_rot(&self) -> Quat {
        self.data.rot
    }

    pub fn local_rot(&self) -> Quat {
        self.data.rot
    }

    pub fn local_scale(&self) -> Vec3 {
        self.data.scale
    }

    pub fn lossy_scale(&self) -> Vec3 {
        self.data.scale
    }

    pub fn local_to_world(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.data.scale, self.data.rot, self.data.pos)
    }

    pub fn world_to_local(&self) -> Mat4 {
        self.local_to_world().inverse()
    }
}

impl Component for Transform {}
