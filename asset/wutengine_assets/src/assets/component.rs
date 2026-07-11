//! Serializable components

use uuid::NonNilUuid;

/// A serialized component
/// Contains the UUID identifying the component type, and the generic
/// component data
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
pub struct SerializedComponent {
    /// The UUID identifying the type of the component, so it can be forwarded
    /// to the correct deserializer
    pub component_type: NonNilUuid,

    /// The component data. The actual format depends on the serialization strategy used when
    /// storing this asset to disk, and should be known by the runtime
    #[cfg_attr(feature = "serde", serde(with = "serde_bytes"))]
    pub data: Vec<u8>,
}
