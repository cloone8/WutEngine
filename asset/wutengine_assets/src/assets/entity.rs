//! Entity asset

use super::{bundle::SerializedBundle, component::SerializedComponent};
use crate::AssetRef;

/// A serialized entity. Contains a set of components and sub-entities
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
pub struct SerializedEntity {
    /// Name of the entity
    pub name: String,

    /// The components on this entity
    pub components: Vec<SerializedComponent>,

    /// Children
    pub children: Vec<EntityEntry>,
}

/// A child of a [SerializedEntity]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "entry_type"))]
pub enum EntityEntry {
    /// Another [SerializedEntity]
    Entity(SerializedEntity),

    /// A reference to a bundle
    Bundle(AssetRef<SerializedBundle>),
}
