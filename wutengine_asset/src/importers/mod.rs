mod image;

pub use image::*;

use crate::assets::texture::SerializedTexture;
use crate::register_importer;

pub fn register_default_importers() {
    register_importer(ImageAssetImporter);
}
