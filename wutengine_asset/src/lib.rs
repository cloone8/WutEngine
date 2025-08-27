//! WutEngine asset management
//!
//! NOTE TO SELF:
//!
//!
//! Assets: Abstract. Not bound to any kind of specific file, but in WutEngine format. Texture, Mesh, etc.
//! AssetLoader: Trait that loads a serialized asset for a given asset name
//!
//!
//! For simple loading, implement specific functions for a given asset. Not bound to a trait. For example:
//! Texture::load_from_image_file
//! Mesh::load_from_obj
//! etc.
//!
//! (Not Mesh::load_from_fbx because FBX can have meshes, materials, etc.)
//!
//! Then in the editor we can make an AssetImporter trait that reads files with an extension (.jpg, .obj, etc.)
//! and imports them into the project

use core::any::{Any, type_name};
use core::error::Error;
use core::fmt::Debug;
use core::ops::{Deref, DerefMut};
use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock};

use cfg_if::cfg_if;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use wutengine_util::GlobalManager;

pub mod serializers;

static ASSET_MANAGER: GlobalManager<AssetManager> = GlobalManager::new();

trait CachableAsset: Any + Send + Sync + Debug {}

impl<T: Asset + Send + Sync> CachableAsset for T {}

#[derive(Debug)]
struct AssetManager {
    loader: Box<dyn AssetLoader>,
    asset_cache: RwLock<HashMap<String, Box<dyn CachableAsset>>>,
    default_format: AssetSerializationFormat,
}

impl AssetManager {
    fn new(loader: Box<dyn AssetLoader>, default_format: AssetSerializationFormat) -> Self {
        Self {
            loader,
            asset_cache: RwLock::new(HashMap::new()),
            default_format,
        }
    }
}

/// Initializes the global asset manager. Should not be called by the user, only by the engine
/// runtime
#[doc(hidden)]
pub fn init(loader: Box<dyn AssetLoader>, default_format: AssetSerializationFormat) {
    GlobalManager::init(&ASSET_MANAGER, AssetManager::new(loader, default_format));
}

fn load_cached<A: Asset>(path: &str) -> Option<A> {
    let cache = ASSET_MANAGER.asset_cache.read().unwrap();

    let cached = if let Some(c) = cache.get(path) {
        c.as_ref()
    } else {
        return None;
    };

    let cached_any: &dyn Any = cached;

    let downcast_result = if let Some(cast) = cached_any.downcast_ref::<A>() {
        cast
    } else {
        log::warn!(
            "Cached value for asset {path} is of a different type than expected ({})",
            type_name::<A>()
        );
        return None;
    };

    Some(downcast_result.clone())
}

pub fn load<A: Asset>(path: impl AsRef<str>) -> Result<A, AssetLoadError> {
    let path = path.as_ref();

    log::debug!("Loading asset {path}");

    if let Some(cached) = load_cached::<A>(path) {
        log::trace!("Returning cached asset for {path}");
        return Ok(cached);
    }

    log::trace!("Loading asset {path} from asset loader");

    let mut buf = Vec::new();

    if let Err(e) = ASSET_MANAGER.loader.load_into(path, &mut buf) {
        match e {
            AssetLoaderError::UnknownAsset => {
                log::warn!("Could not find asset at path {path}");
                return Err(AssetLoadError::UnknownAsset);
            }
            AssetLoaderError::Other(error) => {
                log::error!("Error while reading asset at path {path}: {error}");
                return Err(AssetLoadError::Read(error));
            }
        }
    }

    let asset = if A::FORCE_BINARY
        || ASSET_MANAGER.default_format == AssetSerializationFormat::Binary
    {
        log::debug!("Deserializing binary asset {path}");

        match postcard::from_bytes::<A>(&buf) {
            Ok(asset) => asset,
            Err(e) => {
                log::error!("Error decoding binary asset: {e}");
                return Err(AssetLoadError::CorruptAsset);
            }
        }
    } else {
        log::debug!("Deserializing text asset {path}");

        cfg_if! {
            if #[cfg(not(feature = "format_text"))] {
                log::error!(
                    "Trying to deserialize text-format asset while WutEngine was compiled without text format support"
                );
                return Err(AssetLoadError::UnsupportedTextFormat);
            } else {
                match serde_json::from_slice::<A>(&buf) {
                    Ok(asset) => asset,
                    Err(e) => {
                        log::error!("Error decoding text asset: {e}");
                        return Err(AssetLoadError::CorruptAsset);
                    }
                }
            }
        }
    };

    // Cache the asset
    ASSET_MANAGER
        .asset_cache
        .write()
        .unwrap()
        .insert(path.to_string(), Box::new(asset.clone()));

    Ok(asset)
}

#[derive(Error, Debug)]
pub enum AssetLoadError {
    #[error("Asset could not be found")]
    UnknownAsset,

    #[error("Asset could not be deserialized into the correct format")]
    CorruptAsset,

    #[error("WutEngine was not compiled with text format support")]
    UnsupportedTextFormat,

    #[error("Asset was found but could not be properly read: {0}")]
    Read(Box<dyn Error>),
}

#[derive(Debug, Error)]
pub enum AssetLoaderError {
    #[error("Asset could not be found")]
    UnknownAsset,

    #[error("Error occurred while reading asset: {0}")]
    Other(Box<dyn Error>),
}

pub trait AssetLoader: Send + Sync + Debug {
    fn load_into(&self, asset: &str, buffer: &mut Vec<u8>) -> Result<(), AssetLoaderError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetSerializationFormat {
    Binary,
    Text,
}

pub trait Asset: Any + Debug + Clone + Send + Sync + Serialize + DeserializeOwned {
    const FORCE_BINARY: bool = false;
}

#[derive(Debug)]
pub struct BasicAssetLoader {
    pub root_directory: PathBuf,
}

impl Default for BasicAssetLoader {
    fn default() -> Self {
        let exe_path = std::fs::canonicalize(
            std::env::current_exe().expect("Could not find current executable path"),
        )
        .expect("Could not canonicalize executable path");

        let exe_dir = exe_path.parent().unwrap().to_path_buf();

        Self {
            root_directory: exe_dir,
        }
    }
}

impl AssetLoader for BasicAssetLoader {
    fn load_into(&self, asset: &str, buffer: &mut Vec<u8>) -> Result<(), AssetLoaderError> {
        let path = self.root_directory.join(asset);

        log::debug!("Reading asset {asset} from file {}", path.to_string_lossy());

        let mut file = match std::fs::File::open(path) {
            Ok(f) => f,
            Err(e) => match e.kind() {
                std::io::ErrorKind::NotFound => return Err(AssetLoaderError::UnknownAsset),
                _ => return Err(AssetLoaderError::Other(Box::new(e))),
            },
        };

        file.read_to_end(buffer)
            .map_err(|e| AssetLoaderError::Other(Box::new(e)))?;

        Ok(())
    }
}

/// Handle to any type of [Asset].
/// Automatically loads the asset through the asset manager upon dereference
#[derive(Debug, Clone)]
pub struct AssetHandle<T> {
    /// The asset path
    path: Option<String>,

    /// The actual initialized asset
    inner: OnceLock<Box<T>>,
}

impl<T: Asset> AssetHandle<T> {
    #[inline]
    pub fn new_from_path(path: impl Into<String>) -> Self {
        Self {
            path: Some(path.into()),
            inner: OnceLock::new(),
        }
    }

    #[inline]
    pub fn new_from_val(val: T) -> Self {
        Self {
            path: None,
            inner: OnceLock::from(Box::new(val)),
        }
    }

    pub fn ensure_loaded(this: &Self) -> Result<(), AssetLoadError> {
        if this.inner.get().is_some() {
            return Ok(());
        }

        assert!(
            this.path.is_some(),
            "Unloaded asset handle without path found"
        );

        let val = load::<T>(this.path.as_deref().unwrap())?;

        // We don't care if another thread initialized this at the same time
        _ = this.inner.set(Box::new(val));

        Ok(())
    }
}

impl<T> Deref for AssetHandle<T>
where
    T: Asset,
{
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        Self::ensure_loaded(self).expect("Failed to load asset from handle");

        self.inner.get().unwrap()
    }
}

impl<T> DerefMut for AssetHandle<T>
where
    T: Asset,
{
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        Self::ensure_loaded(self).expect("Failed to load asset from handle");

        self.inner.get_mut().unwrap()
    }
}

impl<T> From<T> for AssetHandle<T>
where
    T: Asset,
{
    fn from(value: T) -> Self {
        Self::new_from_val(value)
    }
}

impl<T> Serialize for AssetHandle<T>
where
    T: Asset,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::Error;

        match &self.path {
            Some(path) => serializer.serialize_str(path.as_str()),
            None => Err(S::Error::custom("Missing path in asset handle")),
        }
    }
}

impl<'de, T> Deserialize<'de> for AssetHandle<T>
where
    T: Asset,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self {
            path: Some(String::deserialize(deserializer)?),
            inner: OnceLock::new(),
        })
    }
}

#[cfg(test)]
mod test {
    use serde::{Deserialize, Serialize};

    use crate::{Asset, AssetHandle};

    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    struct EmptyTestAsset;

    impl Asset for EmptyTestAsset {}

    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    struct BasicTestAsset {
        test: Option<AssetHandle<EmptyTestAsset>>,
        a: String,
        b: String,
        c: i64,
    }

    impl Asset for BasicTestAsset {}

    #[test]
    fn test_asset_handle_serde() {
        let asset = BasicTestAsset {
            test: Some(AssetHandle::<EmptyTestAsset>::new_from_path(
                "my_test_asset.txt",
            )),
            a: "String A".to_string(),
            b: "String B".to_string(),
            c: 400,
        };

        let serialized =
            serde_json::to_string_pretty(&asset).expect("Failed to serialize basic asset");

        let deserialized = serde_json::from_str::<BasicTestAsset>(&serialized)
            .expect("Failed to deserialize basic asset");

        assert_eq!(
            asset.test.unwrap().path,
            deserialized.test.unwrap().path,
            "Serialization roundtrip failed"
        );

        assert_eq!(asset.a, deserialized.a, "Serialization roundtrip failed");

        assert_eq!(asset.b, deserialized.b, "Serialization roundtrip failed");

        assert_eq!(asset.c, deserialized.c, "Serialization roundtrip failed");
    }
}
