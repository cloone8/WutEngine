use core::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::id::KeyType;

#[derive(Debug, Serialize, Deserialize)]
pub struct SerializedScript<F> {
    pub script_id: KeyType,
    pub instance_id: KeyType,
    pub data: F,
}
