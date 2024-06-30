use core::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::id::KeyType;

use super::{script::SerializedScript, transform::SerializedTransform};

#[derive(Debug, Serialize, Deserialize)]
pub struct SerializedObject<F> {
    pub instance_id: KeyType,
    pub name: String,
    pub transform: SerializedTransform,
    pub scripts: Vec<SerializedScript<F>>,
}
