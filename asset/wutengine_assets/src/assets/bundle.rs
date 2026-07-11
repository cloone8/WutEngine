//! Pre-made bundles of entities

use super::entity::SerializedEntity;
use crate::{AssetRef, SerializedAsset};

/// A serialized bundle of entries, each of which is either an entity or another bundle
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
pub struct SerializedBundle {
    /// The name of the bundle
    pub name: String,

    /// The entries
    pub entries: Vec<BundleEntry>,
}

impl SerializedAsset for SerializedBundle {
    const ID: uuid::NonNilUuid =
        uuid::NonNilUuid::new(uuid::uuid!("678f0243-8d5f-4829-90f1-0caca98e2efc")).unwrap();
}

/// An entry in a [SerializedBundle]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "entry_type"))]
pub enum BundleEntry {
    /// An inline [SerializedEntity]
    Entity(SerializedEntity),

    /// A reference to a [SerializedBundle]
    Bundle(AssetRef<SerializedBundle>),
}
