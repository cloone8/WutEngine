//! Project asset manager

use alloc::collections::BTreeMap;
use core::any::Any;
use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;
use wutengine::uuid;

#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub(crate) enum LoadErr {
    #[display("Asset index was missing from disk")]
    MissingIndexFile,

    #[display("Failed to deserialize asset index: {}", _0)]
    Deserialize(serde_json::Error),

    #[display("I/O error while loading asset index: {}", _0)]
    IO(std::io::Error),
}

#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub(crate) enum SaveErr {
    #[display("Failed to serialize asset index: {}", _0)]
    Serialize(serde_json::Error),

    #[display("I/O error while storing asset index: {}", _0)]
    IO(std::io::Error),
}

/// Project asset manager
#[derive(Debug)]
pub(crate) struct ProjectAssetManager {
    root: PathBuf,
    assets: HashMap<uuid::Uuid, ProjectAsset>,
}

impl ProjectAssetManager {
    pub(crate) fn load(root: PathBuf) -> Result<Self, LoadErr> {
        let asset_index_file = root.join("assets.json");

        if !std::fs::exists(&asset_index_file)? {
            return Err(LoadErr::MissingIndexFile);
        }

        let bytes = std::fs::read(&asset_index_file)?;

        let mut assets: HashMap<uuid::Uuid, ProjectAsset> = serde_json::from_slice(&bytes)?;

        for (&id, asset) in assets.iter_mut() {
            asset.id = id;
        }

        Ok(Self { root, assets })
    }

    pub(crate) fn save(&self) -> Result<(), SaveErr> {
        let assets_serialized =
            serde_json::to_string_pretty(&BTreeMap::from_iter(self.assets.iter()))?;

        Ok(std::fs::write(
            self.root.join("assets.json"),
            assets_serialized,
        )?)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ProjectAsset {
    #[serde(skip)]
    id: uuid::Uuid,

    path: PathBuf,

    #[serde(skip)]
    asset: Option<Box<dyn Any + Send + Sync>>,
}
