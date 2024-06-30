use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};
use wutengine_core::transform::LocalTransform;

#[derive(Debug, Serialize, Deserialize)]
pub struct SerializedTransform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl From<SerializedTransform> for LocalTransform {
    fn from(value: SerializedTransform) -> Self {
        Self {
            pos: value.position,
            rot: value.rotation,
            scale: value.scale,
        }
    }
}
