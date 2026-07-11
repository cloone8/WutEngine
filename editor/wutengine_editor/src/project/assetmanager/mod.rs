//! Project asset manager

use alloc::collections::BTreeMap;
use core::{any::Any, fmt::Display, ops::Deref};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::RwLock,
};

use serde::{Deserialize, Serialize};
use uuid::NonNilUuid;
use wutengine::{asset::SerializedAsset, world};

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
    asset_index: PathBuf,
    asset_root: PathBuf,
    assets: RwLock<HashMap<ProjectAssetId, ProjectAsset>>,
}

/// Loading/saving
impl ProjectAssetManager {
    pub(super) fn load(root: PathBuf) -> Result<Self, LoadErr> {
        let asset_index_file = root.join("assets.json");

        if !std::fs::exists(&asset_index_file)? {
            return Err(LoadErr::MissingIndexFile);
        }

        let bytes = std::fs::read(&asset_index_file)?;

        let mut assets: HashMap<ProjectAssetId, ProjectAsset> = serde_json::from_slice(&bytes)?;

        for (&id, asset) in assets.iter_mut() {
            asset.id = Some(id);
        }

        Ok(Self {
            asset_index: asset_index_file,
            asset_root: root.join("assets"),
            assets: RwLock::new(assets),
        })
    }

    pub(crate) fn save(&self) -> Result<(), SaveErr> {
        let assets = self.assets.read().unwrap();

        let assets_serialized = serde_json::to_string_pretty(&BTreeMap::from_iter(assets.iter()))?;

        Ok(std::fs::write(&self.asset_index, assets_serialized)?)
    }

    pub(crate) fn asset_root(&self) -> &Path {
        &self.asset_root
    }
}

/// An error while inserting a new asset
#[derive(Debug, derive_more::Display, derive_more::Error)]
pub(crate) enum InsertAssetErr {
    /// An invalid asset name was given
    #[display("Asset name was invalid: {}", _0)]
    InvalidName(#[error(not(source))] String),

    /// IO error while writing
    #[display("Failed to write asset to disk: {}", _0)]
    IO(std::io::Error),

    /// Asset path is outside project
    #[display("Path is outside project root: {}", _0.to_string_lossy())]
    OutsideProject(#[error(not(source))] PathBuf),

    /// JSON serialization failed
    #[display("Failed to serialize asset to text format: {}", _0)]
    SerializeText(serde_json::Error),

    /// Postcard serialization failed
    #[display("Failed to serialize asset to binary format: {}", _0)]
    SerializeBinary(postcard::Error),
}

/// Asset management
impl ProjectAssetManager {
    /// Adds a new asset to the project with the provided name, in the given directory.
    /// All directory paths are relative to the project root. If the path contains `..` components,
    /// they must not escape the project root
    pub(crate) fn insert_asset<A: SerializedAsset>(
        &self,
        asset: A,
        directory: impl AsRef<Path>,
        name: &str,
    ) -> Result<ProjectAssetId, InsertAssetErr> {
        let name = name.trim();

        if name.is_empty() {
            return Err(InsertAssetErr::InvalidName(name.to_string()));
        }

        // First we make the containing directory absolute
        let directory = directory.as_ref();

        let directory_abs = if directory.is_absolute() {
            directory.to_path_buf()
        } else {
            self.asset_root.join(directory)
        };

        // Make sure the resulting path is within the asset root
        if !directory_abs.starts_with(&self.asset_root) {
            return Err(InsertAssetErr::OutsideProject(directory_abs));
        }

        // Create the intermediate directories
        std::fs::create_dir_all(&directory_abs).map_err(InsertAssetErr::IO)?;

        // Now determine the path including the asset name and extension
        let extension = if A::PREFER_BINARY_SERIALIZATION {
            ".we-binasset"
        } else {
            ".we-txtasset"
        };

        let path = directory_abs.join(format!("{name}{extension}"));

        // Serialize the asset
        let (serialized, format) = if A::PREFER_BINARY_SERIALIZATION {
            let bytes = postcard::to_allocvec(&asset).map_err(InsertAssetErr::SerializeBinary)?;

            (bytes, ProjectAssetFormat::Postcard)
        } else {
            let as_string =
                serde_json::to_string_pretty(&asset).map_err(InsertAssetErr::SerializeText)?;

            (as_string.into_bytes(), ProjectAssetFormat::Json)
        };

        // Write it to disk
        std::fs::write(&path, serialized).map_err(InsertAssetErr::IO)?;

        let path = path
            .canonicalize()
            .expect("Failed to canonicalize asset path");

        // Store the new asset info, and return the new ID
        let project_relative = path
            .strip_prefix(&self.asset_root)
            .expect("Failed to make path project relative");

        log::info!("{}", project_relative.to_string_lossy());

        let id = ProjectAssetId::new_random();

        let project_asset = ProjectAsset {
            id: Some(id),
            format,
            asset_type: A::ID,
            path: project_relative.to_path_buf(),
        };

        let mut assets = self.assets.write().unwrap();

        assets.insert(id, project_asset);

        drop(assets);

        wutengine::event::publish(AssetCreated {
            path,
            id,
            name: name.to_string(),
        });

        Ok(id)
    }

    /// NOTE: Returns a read-lock, so the asset manager is locked while the returned value is held
    pub(crate) fn asset_iter(&self) -> impl Deref<Target = HashMap<ProjectAssetId, ProjectAsset>> {
        self.assets.read().unwrap()
    }

    pub(crate) fn get_project_asset(&self, id: &ProjectAssetId) -> Option<ProjectAsset> {
        self.assets.read().unwrap().get(id).cloned()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub(crate) struct ProjectAssetId(uuid::NonNilUuid);

impl ProjectAssetId {
    #[inline]
    fn new_random() -> Self {
        Self(uuid::NonNilUuid::new(uuid::Uuid::new_v4()).unwrap())
    }
}

impl Display for ProjectAssetId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ProjectAsset {
    #[serde(skip)]
    id: Option<ProjectAssetId>,

    format: ProjectAssetFormat,

    /// Corresponds to the [SerializedAsset::ID] constant
    asset_type: uuid::NonNilUuid,

    #[serde(serialize_with = "to_cross_platform_path")]
    path: PathBuf,
}

impl ProjectAsset {
    pub(crate) fn id(&self) -> ProjectAssetId {
        self.id.expect("ID should have been filled")
    }

    pub(crate) fn name(&self) -> &str {
        self.path
            .file_stem()
            .expect("Asset should have a name")
            .to_str()
            .expect("Asset name should be UTF8")
    }

    pub(crate) fn directory(&self) -> Option<&Path> {
        self.path.parent()
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn format(&self) -> ProjectAssetFormat {
        self.format
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ProjectAssetFormat {
    Json,
    Postcard,
}

/// Converts a given path to a platform-independent string before serializing
fn to_cross_platform_path<S>(path: impl AsRef<Path>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let path = path.as_ref();

    let as_string = path
        .components()
        .map(|c| c.as_os_str().to_str().expect("Non-utf8 asset path"))
        .collect::<Vec<_>>()
        .join("/");

    as_string.serialize(serializer)
}

#[derive(Debug, Clone)]
pub(crate) struct AssetCreated {
    pub(crate) path: PathBuf,
    pub(crate) id: ProjectAssetId,
    pub(crate) name: String,
}
