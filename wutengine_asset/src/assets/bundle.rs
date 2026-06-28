//! Pre-made bundles of entities

use serde::Deserialize;
use serde::Serialize;

use crate::AssetHandle;

use super::entity::SerializedEntity;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SerializedBundle {
    pub name: String,
    pub entries: Vec<BundleEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "entry_type")]
pub enum BundleEntry {
    Entity(SerializedEntity),
    Bundle(AssetHandle<SerializedBundle>),
}
