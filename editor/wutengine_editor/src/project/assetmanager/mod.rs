//! Project asset manager

use alloc::collections::BTreeMap;
use core::ops::Deref;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::RwLock;
use wutengine::asset::SerializedAsset;

use serde::Deserialize;
use serde::Serialize;

use crate::assets::path::AssetPath;

/// An error while loading the asset manager
#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub(crate) enum LoadErr {
    /// The asset index was missing
    #[display("Asset index was missing from disk")]
    MissingIndexFile,

    /// Asset index was not deserializable, probably corrupt
    #[display("Failed to deserialize asset index: {}", _0)]
    Deserialize(serde_json::Error),

    /// I/O error
    #[display("I/O error while loading asset index: {}", _0)]
    IO(std::io::Error),
}

/// An error while saving the asset manager to disk
#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub(crate) enum SaveErr {
    /// Serialization failed
    #[display("Failed to serialize asset index: {}", _0)]
    Serialize(serde_json::Error),

    /// I/O error
    #[display("I/O error while storing asset index: {}", _0)]
    IO(std::io::Error),
}

/// Project asset manager
#[derive(Debug)]
pub(crate) struct ProjectAssetManager {
    asset_index: PathBuf,
    asset_root: PathBuf,
    assets: RwLock<HashMap<uuid::NonNilUuid, ProjectAsset>>,
}

/// Loading/saving
impl ProjectAssetManager {
    pub(super) fn load(root: &Path) -> Result<Self, LoadErr> {
        let asset_index_file = root.join("assets.json");

        if !std::fs::exists(&asset_index_file)? {
            return Err(LoadErr::MissingIndexFile);
        }

        let bytes = std::fs::read(&asset_index_file)?;

        let mut assets: HashMap<uuid::NonNilUuid, ProjectAsset> = serde_json::from_slice(&bytes)?;

        for (&id, asset) in &mut assets {
            asset.id = Some(id);
        }

        Ok(Self {
            asset_index: asset_index_file,
            asset_root: root.join("assets"),
            assets: RwLock::new(assets),
        })
    }

    /// Saves the project asset manager to disk
    pub(crate) fn save(&self) -> Result<(), SaveErr> {
        let assets = self.assets.read().unwrap();

        let assets_serialized =
            serde_json::to_string_pretty(&assets.iter().collect::<BTreeMap<_, _>>())?;

        Ok(std::fs::write(&self.asset_index, assets_serialized)?)
    }

    /// Returns the path to the project asset root directory
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
    /// Inserts an asset that has already been serialized into the project
    pub(crate) fn insert_serialized_asset(
        &self,
        asset_content: &[u8],
        asset_format: ProjectAssetFormat,
        asset_type: uuid::NonNilUuid,
        path: impl AsRef<Path>,
    ) -> Result<uuid::NonNilUuid, InsertAssetErr> {
        let path = path.as_ref();

        let path_abs = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.asset_root.join(path)
        };

        // Make sure the resulting path is within the asset root
        if !path_abs.starts_with(&self.asset_root) {
            return Err(InsertAssetErr::OutsideProject(path_abs));
        }

        let parent_dir = path_abs
            .parent()
            .expect("Path must be within asset root, so must have a parent");

        // Create the intermediate directories
        std::fs::create_dir_all(parent_dir).map_err(InsertAssetErr::IO)?;

        std::fs::write(&path_abs, asset_content).map_err(InsertAssetErr::IO)?;

        let canonicalized_path = path_abs
            .canonicalize()
            .expect("Failed to canonicalize asset path");

        // Store the new asset info, and return the new ID
        let project_relative = canonicalized_path
            .strip_prefix(&self.asset_root)
            .expect("Failed to make path project relative");

        log::info!("{}", project_relative.to_string_lossy());

        let id = uuid::NonNilUuid::new(uuid::Uuid::new_v4()).unwrap();

        let project_asset = ProjectAsset {
            id: Some(id),
            format: asset_format,
            asset_type,
            path: project_relative.to_path_buf(),
        };

        let mut assets = self.assets.write().unwrap();

        assets.insert(id, project_asset);

        drop(assets);

        wutengine::event::publish(AssetCreated);

        Ok(id)
    }
    /// Adds a new asset to the project with the provided name, in the given directory.
    /// All directory paths are relative to the project root. If the path contains `..` components,
    /// they must not escape the project root
    pub(crate) fn insert_asset<A: SerializedAsset>(
        &self,
        asset: &A,
        directory: impl AsRef<Path>,
        name: &str,
    ) -> Result<uuid::NonNilUuid, InsertAssetErr> {
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

        self.insert_serialized_asset(&serialized, format, A::ID, path)
    }

    /// NOTE: Returns a read-lock, so the asset manager is locked while the returned value is held
    pub(crate) fn asset_iter(
        &self,
    ) -> impl Deref<Target = HashMap<uuid::NonNilUuid, ProjectAsset>> {
        self.assets.read().unwrap()
    }

    /// Returns the project asset information for a given asset id, if it exists
    pub(crate) fn get_project_asset(&self, id: &uuid::NonNilUuid) -> Option<ProjectAsset> {
        self.assets.read().unwrap().get(id).cloned()
    }
}

/// Information on an asset in the project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ProjectAsset {
    #[serde(skip)]
    id: Option<uuid::NonNilUuid>,

    format: ProjectAssetFormat,

    /// Corresponds to the [`SerializedAsset::ID`] constant
    asset_type: uuid::NonNilUuid,

    #[serde(serialize_with = "to_cross_platform_path")]
    path: PathBuf,
}

impl ProjectAsset {
    /// Returns the ID of the asset type
    pub(crate) fn asset_type(&self) -> uuid::NonNilUuid {
        self.asset_type
    }

    /// Returns the name of the asset
    pub(crate) fn name(&self) -> &str {
        self.path
            .file_stem()
            .expect("Asset should have a name")
            .to_str()
            .expect("Asset name should be UTF8")
    }

    /// Returns the path to the asset
    pub(crate) fn path(&self) -> AssetPath {
        AssetPath::new(super::asset_manager().asset_root.join(&self.path))
    }
}

/// The serialization format of a project asset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ProjectAssetFormat {
    /// JSON
    Json,

    /// Postcard
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

/// Event emitted when a new asset was created
#[derive(Debug, Clone)]
pub(crate) struct AssetCreated;
