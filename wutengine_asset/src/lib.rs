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

use core::error::Error;
use core::fmt::Debug;

use cfg_if::cfg_if;
use serde::Serialize;
use serde::de::DeserializeOwned;
use thiserror::Error;
use wutengine_util::GlobalManager;

pub mod serializers;

static ASSET_MANAGER: GlobalManager<AssetManager> = GlobalManager::new();

#[derive(Debug)]
struct AssetManager {
    loader: Box<dyn AssetLoader>,
    default_format: AssetSerializationFormat,
}

impl AssetManager {
    fn new(loader: Box<dyn AssetLoader>, default_format: AssetSerializationFormat) -> Self {
        Self {
            loader,
            default_format,
        }
    }
}

pub fn init(loader: Box<dyn AssetLoader>, default_format: AssetSerializationFormat) {
    GlobalManager::init(&ASSET_MANAGER, AssetManager::new(loader, default_format));
}

pub fn load<A: Asset>(path: impl AsRef<str>) -> Option<A> {
    let path = path.as_ref();

    log::debug!("Loading asset {path}");

    let mut buf = Vec::new();

    if let Err(e) = ASSET_MANAGER.loader.load_into(path, &mut buf) {
        match e {
            AssetLoadError::UnknownAsset => {
                log::warn!("Could not find asset at path {path}");
            }
            AssetLoadError::Read(error) => {
                log::error!("Error while reading asset at path {path}: {error}");
            }
        }

        return None;
    }

    let asset = if A::FORCE_BINARY
        || ASSET_MANAGER.default_format == AssetSerializationFormat::Binary
    {
        log::debug!("Deserializing binary asset {path}");

        match postcard::from_bytes::<A>(&buf) {
            Ok(asset) => asset,
            Err(e) => {
                log::error!("Error decoding binary asset: {e}");
                return None;
            }
        }
    } else {
        log::debug!("Deserializing text asset {path}");

        cfg_if! {
            if #[cfg(not(feature = "format_text"))] {
                log::error!(
                    "Trying to deserialize text-format asset while WutEngine was compiled without text format support"
                );
                return None;
            } else {
                match serde_json::from_slice::<A>(&buf) {
                    Ok(asset) => asset,
                    Err(e) => {
                        log::error!("Error decoding text asset: {e}");
                        return None;
                    }
                }
            }
        }
    };

    Some(asset)
}

#[derive(Error, Debug)]
pub enum AssetLoadError {
    #[error("Asset could not be found")]
    UnknownAsset,

    #[error("Asset was found but could not be properly read: {0}")]
    Read(Box<dyn Error>),
}

pub trait AssetLoader: Send + Sync + Debug {
    fn load_into(&self, asset: &str, buffer: &mut Vec<u8>) -> Result<(), AssetLoadError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetSerializationFormat {
    Binary,
    Text,
}

pub trait Asset: Serialize + DeserializeOwned {
    const FORCE_BINARY: bool = false;
}

#[derive(Debug)]
pub struct BasicAssetLoader {}

impl AssetLoader for BasicAssetLoader {
    fn load_into(&self, asset: &str, buffer: &mut Vec<u8>) -> Result<(), AssetLoadError> {
        todo!()
    }
}
