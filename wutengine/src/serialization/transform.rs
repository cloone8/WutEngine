use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SerializedTransform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}
