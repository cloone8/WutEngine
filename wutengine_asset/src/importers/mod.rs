mod image;

use core::any::Any;
use core::error::Error;
use std::path::Path;

pub use image::*;

use crate::Asset;
use crate::AssetHandle;
use crate::AssetImporter;
use crate::FromSerializedAnyErr;

#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum ImportErr<A: Asset> {
    #[display("I/O error while reading asset from disk: {_0}")]
    IO(std::io::Error),

    #[display(
        "Failed to determine asset type because the asset file extension could not be determined: {_0}"
    )]
    UnknownExtension(#[error(not(source))] String),

    #[display("Asset importer {importer} failed with error {source}")]
    ImporterError {
        importer: String,

        #[error(source)]
        source: Box<dyn Error>,
    },

    #[display("Failed to load imported asset: {_0}")]
    LoadError(FromSerializedAnyErr<A::FromSerializedErr>),

    #[display("No importer found for asset of type: {_0}")]
    UnknownAssetType(#[error(not(source))] String),
}

impl<A: Asset> From<std::io::Error> for ImportErr<A> {
    fn from(value: std::io::Error) -> Self {
        ImportErr::IO(value)
    }
}

impl<A: Asset> From<FromSerializedAnyErr<A::FromSerializedErr>> for ImportErr<A> {
    fn from(value: FromSerializedAnyErr<A::FromSerializedErr>) -> Self {
        Self::LoadError(value)
    }
}

pub fn import<A: Asset>(asset: impl AsRef<Path>) -> Result<AssetHandle<A>, ImportErr<A>> {
    let asset_path = asset.as_ref();

    let asset_type = asset_path
        .extension()
        .and_then(|path_os| path_os.to_str())
        .ok_or_else(|| ImportErr::UnknownExtension(asset_path.to_string_lossy().to_string()))?;

    let asset_dir = asset_path.parent();

    let asset = std::fs::read(asset_path)?;

    import_from_bytes(&asset, asset_type, asset_dir)
}

pub fn import_from_bytes<A: Asset>(
    asset_bytes: &[u8],
    asset_type: &str,
    asset_dir: Option<&Path>,
) -> Result<AssetHandle<A>, ImportErr<A>> {
    let as_any: Box<dyn Any> = if ImageAssetImporter::supports_file_type(asset_type) {
        let img = ImageAssetImporter::import(asset_bytes, asset_type, asset_dir).map_err(|e| {
            ImportErr::ImporterError {
                importer: core::any::type_name::<ImageAssetImporter>().to_string(),
                source: Box::new(e),
            }
        })?;

        Box::new(img)
    } else {
        return Err(ImportErr::UnknownAssetType(asset_type.to_string()));
    };

    let loaded_asset = A::from_serialized_any(as_any.as_ref())?;

    Ok(AssetHandle::new(loaded_asset))
}
