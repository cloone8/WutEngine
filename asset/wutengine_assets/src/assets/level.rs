//! Level asset

use serde::Deserialize;
use serde::Serialize;

use crate::AssetRef;
use crate::SerializedAsset;

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

impl SerializedAsset for SerializedLevel {
    const ID: uuid::NonNilUuid =
        uuid::NonNilUuid::new(uuid::uuid!("7d1e38e2-ce4f-4aa2-b668-35c7fa495818")).unwrap();
}

/// An entry in a [SerializedLevel]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "entry_type")]
pub enum LevelEntry {
    /// An entity
    Entity(SerializedEntity),

    /// A reference to a bundle
    Bundle(AssetRef<SerializedBundle>),
}
