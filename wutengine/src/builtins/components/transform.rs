//! The transform component

use crate::component::Component;
use crate::math::Vec3;
use glam::{Mat4, Quat};

//TODO: Will probably be heavily modified once we get into transform hierarchies, 3D transformations, etc.
/// The transform component. Contains all information about the place of an entity in 3D space.
#[derive(Debug, Default)]
pub struct Transform {
    /// The actual data of this transform.
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

#[profiling::all_functions]
impl Transform {
    /// Creates a new identity transform
    pub fn new() -> Self {
        Self {
            data: TransformData::default(),
        }
    }

    /// Creates a new transform with the given position, but with identity scale and rotation
    pub fn with_pos(pos: Vec3) -> Self {
        Self {
            data: TransformData {
                pos,
                ..TransformData::default()
            },
        }
    }

    /// Creates a new transform with the given rotation, but with identity position and scale
    pub fn with_rot(rot: Quat) -> Self {
        Self {
            data: TransformData {
                rot,
                ..TransformData::default()
            },
        }
    }

    /// Creates a new transform with the given scale, but with identity position and rotation
    pub fn with_scale(scale: Vec3) -> Self {
        Self {
            data: TransformData {
                scale,
                ..TransformData::default()
            },
        }
    }

    /// Creates a new transform with the given position and rotation, but with identity scale
    pub fn with_pos_rot(pos: Vec3, rot: Quat) -> Self {
        Self {
            data: TransformData {
                pos,
                rot,
                ..TransformData::default()
            },
        }
    }

    /// Creates a new transform with the given local position, rotation and scale
    pub fn with_pos_rot_scale(pos: Vec3, rot: Quat, scale: Vec3) -> Self {
        Self {
            data: TransformData { pos, rot, scale },
        }
    }

    /// Gets the world-space position of this transform
    pub fn world_pos(&self) -> Vec3 {
        self.data.pos
    }

    /// Sets the world-space position of this transform
    pub fn set_world_pos(&mut self, pos: Vec3) {
        self.data.pos = pos;
    }

    /// Gets the local-space position of this transform
    pub fn local_pos(&self) -> Vec3 {
        self.data.pos
    }

    /// Sets the local-space position of this transform
    pub fn set_local_pos(&mut self, pos: Vec3) {
        self.data.pos = pos;
    }

    /// Gets the world-space rotation of this transform
    pub fn world_rot(&self) -> Quat {
        self.data.rot
    }

    /// Sets the world-space rotation of this transform
    pub fn set_world_rot(&mut self, rot: Quat) {
        self.data.rot = rot;
    }

    /// Gets the local-space rotation of this transform
    pub fn local_rot(&self) -> Quat {
        self.data.rot
    }

    /// Sets the local-space rotation of this transform
    pub fn set_local_rot(&mut self, rot: Quat) {
        self.data.rot = rot;
    }

    /// Gets the local-space scale of this transform
    pub fn local_scale(&self) -> Vec3 {
        self.data.scale
    }

    /// Gets the lossy world-space scale of this transform.
    /// Note: The world-space scale is lossy due to the math that is involved.
    /// It will be inaccurate
    pub fn lossy_scale(&self) -> Vec3 {
        self.data.scale
    }

    /// Gets the local-to-world matrix of this transform
    pub fn local_to_world(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.data.scale, self.data.rot, self.data.pos)
    }

    /// Gets the world-to-local matrix of this transform
    pub fn world_to_local(&self) -> Mat4 {
        self.local_to_world().inverse()
    }
}

#[profiling::all_functions]
impl Component for Transform {}
