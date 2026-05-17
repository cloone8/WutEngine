use serde::Deserialize;
use serde::Serialize;

use crate::SerializedAsset;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedTexture {
    pub config: TextureConfig,

    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,

    #[serde(with = "serde_bytes")]
    pub mips: Option<Vec<u8>>,
}

impl SerializedAsset for SerializedTexture {}

/// The configuration for creating a new texture
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TextureConfig {
    /// The width of the texture in pixels. Must be at least 1
    pub width: u32,

    /// The height of the texture in pixels. Must be at least 1
    pub height: u32,

    /// The texture format
    pub format: TextureFormat,
}

/// The format of a [SerializedTexture]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TextureFormat {
    /// RGBA with 8-bits per component
    Rgba8,

    /// RGBA with 8-bits per component, with sRGB
    Rgba8Srgb,

    /// RGBA with 32-bit per color float components
    Rgba32,
}

impl TextureFormat {
    /// Returns whether this format is an sRGB format
    #[inline]
    pub fn is_srgb(self) -> bool {
        self == TextureFormat::Rgba8Srgb
    }
}
