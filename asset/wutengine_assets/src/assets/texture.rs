//! Texture asset

use serde::{Deserialize, Serialize};

use crate::SerializedAsset;

/// The data for the texture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedTexture {
    /// The texture configuration
    pub config: TextureConfig,

    /// The raw decoded image data
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,

    /// The data for each mip level
    pub mips: Option<Vec<SerializedMipMap>>,
}

/// The data for a mip-map level of a [SerializedTexture]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedMipMap {
    /// The raw decoded image data
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
}

impl SerializedAsset for SerializedTexture {
    const ID: uuid::NonNilUuid =
        uuid::NonNilUuid::new(uuid::uuid!("27f1a488-9df8-40e8-8663-75c3e769861c")).unwrap();
}

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
