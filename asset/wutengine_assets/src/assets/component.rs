//! Serializable components

use serde::{Deserialize, Serialize};
use uuid::NonNilUuid;

/// A serialized component
/// Contains the UUID identifying the component type, and the generic
/// component data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedComponent {
    /// The UUID identifying the type of the component, so it can be forwarded
    /// to the correct deserializer
    pub component_type: NonNilUuid,

    /// The component data. The actual format depends on the serialization strategy used when
    /// storing this asset to disk, and should be known by the runtime
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
}
