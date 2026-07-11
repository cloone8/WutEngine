#![doc = include_str!("../README.md")]

use core::any::Any;
use core::error::Error;
use std::path::Path;

mod image;
pub use image::*;

/// An asset imported through an [AssetImporter]
#[derive(Debug)]
pub struct ImportedAsset {
    /// Corresponds to the [wutengine_assets::SerializedAsset::ID] constant
    pub id: uuid::NonNilUuid,

    /// An optional asset name, if one can be determined from the asset
    pub name: Option<String>,

    /// The actual asset
    pub asset: Box<dyn Any + Send + Sync>,
}

/// Something than can import assets from an external format into a [wutengine_assets::SerializedAsset], usable by WutEngine
pub trait AssetImporter: Send + Sync + 'static {
    /// The file types that this importer supports. Should be the on-disk extension of the file, like `png`, `jpg`, `obj`, `wav`, `mp3`, etc.
    fn supported_file_types() -> Vec<&'static str>;

    /// Imports a given asset from in-memory bytes. If known, the original path to the asset is also given
    fn from_bytes(
        bytes: &[u8],
        file_type: &str,
        path: Option<&Path>,
    ) -> Result<Vec<ImportedAsset>, Box<dyn Error>>;

    /// Imports a given asset of a certain type from disk.
    fn from_disk(file_type: &str, path: &Path) -> Result<Vec<ImportedAsset>, Box<dyn Error>> {
        let bytes = std::fs::read(path)?;

        Self::from_bytes(&bytes, file_type, Some(path))
    }
}
