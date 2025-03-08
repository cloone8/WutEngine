//! Texture data for the WutEngine graphics APIs

use image::DynamicImage;

/// The raw data corresponding to a texture
#[derive(Debug, Clone, Default)]
pub struct TextureData {
    /// The actual image data, in the format provided by [image]
    pub imagedata: DynamicImage,
}
