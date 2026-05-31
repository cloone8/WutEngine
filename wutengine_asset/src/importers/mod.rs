//! Default asset importers for the built-in WutEngine assets

mod image;

pub use image::*;

use crate::register_importer;

/// Registers all built-in [AssetImporters](crate::AssetImporter)
pub fn register_default_importers() {
    register_importer(ImageAssetImporter);
}
