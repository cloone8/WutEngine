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
}

impl Default for TransformData {
    fn default() -> Self {
        Self { pos: Vec3::ZERO }
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
            data: TransformData { pos },
        }
    }

    pub fn world_pos(&self) -> Vec3 {
        self.data.pos
    }

    pub fn local_pos(&self) -> Vec3 {
        self.data.pos
    }

    pub fn local_to_world(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(Vec3::ONE, Quat::IDENTITY, self.data.pos)
    }

    pub fn world_to_local(&self) -> Mat4 {
        self.local_to_world().inverse()
    }
}

impl Component for Transform {}
