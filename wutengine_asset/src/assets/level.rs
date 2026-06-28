//! Level asset

use serde::Deserialize;
use serde::Serialize;

use crate::AssetHandle;

use super::bundle::SerializedBundle;
use super::entity::SerializedEntity;

/// A playable level, containing a set of entity/bundle entries.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SerializedLevel {
    /// Name of the level
    pub name: String,

    /// Entries in the level
    pub entries: Vec<LevelEntry>,
}

/// An entry in a [SerializedLevel]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "entry_type")]
pub enum LevelEntry {
    /// An entity
    Entity(SerializedEntity),

    /// A reference to a bundle
    Bundle(AssetHandle<SerializedBundle>),
}
