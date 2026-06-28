//! Pre-made bundles of entities

use serde::Deserialize;
use serde::Serialize;

use crate::AssetHandle;

use super::entity::SerializedEntity;

/// A serialized bundle of entries, each of which is either an entity or another bundle
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SerializedBundle {
    /// The name of the bundle
    pub name: String,

    /// The entries
    pub entries: Vec<BundleEntry>,
}

/// An entry in a [SerializedBundle]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "entry_type")]
pub enum BundleEntry {
    /// An inline [SerializedEntity]
    Entity(SerializedEntity),

    /// A reference to a [SerializedBundle]
    Bundle(AssetHandle<SerializedBundle>),
}
