#![doc = include_str!("../README.md")]

use core::any::Any;
use core::error::Error;
use std::path::Path;

mod image;
pub use image::*;

#[derive(Debug)]
pub struct ImportedAsset {
    /// Corresponds to the [wutengine_assets::SerializedAsset::ID] constant
    pub id: uuid::NonNilUuid,

    /// An optional asset name, if one can be determined from the asset
    pub name: Option<String>,

    /// The actual asset
    pub asset: Box<dyn Any + Send + Sync>,
}

pub trait AssetImporter {
    fn from_bytes(bytes: &[u8], path: Option<&Path>) -> Result<Vec<ImportedAsset>, Box<dyn Error>>;

    fn from_disk(path: &Path) -> Result<Vec<ImportedAsset>, Box<dyn Error>> {
        let bytes = std::fs::read(path)?;

        Self::from_bytes(&bytes, Some(path))
    }
}
