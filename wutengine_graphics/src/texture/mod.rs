use image::DynamicImage;
use serde::{Deserialize, Serialize};
use wutengine_asset::Asset;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Texture {
    #[serde(with = "wutengine_asset::serializers::image::dynamic_image")]
    pub imagedata: DynamicImage,

    pub filtering: TextureFiltering,

    pub wrapping: TextureWrapping,
}

impl Asset for Texture {}

/// The texture filtering method used in a [TextureData]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TextureFiltering {
    /// Linear interpolation between the two nearest color values in the texture
    #[default]
    Linear,

    /// The nearest color value
    Nearest,
}

/// Per-axis texture wrapping settings for [TextureData]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TextureWrapping {
    /// One setting for both axes
    Both(WrappingMethod),

    /// One setting per-axis
    PerAxis {
        /// U axis (horizontal)
        u: WrappingMethod,

        /// V axis (vertical)
        v: WrappingMethod,
    },
}

impl Default for TextureWrapping {
    fn default() -> Self {
        Self::Both(Default::default())
    }
}

/// The texture wrapping method used in a [TextureData]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WrappingMethod {
    /// Texture is repeated linearly
    #[default]
    Repeat,

    /// Texture is mirrored
    Mirror,

    /// Texture is clamped
    Clamp,
}
