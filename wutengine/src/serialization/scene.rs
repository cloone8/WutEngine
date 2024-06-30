use serde::{Deserialize, Serialize};

use super::{format::SerializationFormat, object::SerializedObject};

#[derive(Debug, Serialize, Deserialize)]
pub struct SerializedScene<F: SerializationFormat> {
    pub objects: Vec<SerializedObject<F::RawType>>,
}
