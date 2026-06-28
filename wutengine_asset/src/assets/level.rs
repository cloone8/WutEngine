//! Level asset

use serde::Deserialize;
use serde::Serialize;

use crate::AssetHandle;

use super::bundle::SerializedBundle;
use super::entity::SerializedEntity;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SerializedLevel {
    pub name: String,
    pub entries: Vec<LevelEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "entry_type")]
pub enum LevelEntry {
    Entity(SerializedEntity),
    Bundle(AssetHandle<SerializedBundle>),
}
