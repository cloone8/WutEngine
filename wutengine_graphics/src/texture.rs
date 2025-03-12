//! Texture data for the WutEngine graphics APIs

use image::DynamicImage;

/// The raw data corresponding to a texture
#[derive(Debug, Clone, Default)]
pub struct TextureData {
    /// The actual image data, in the format provided by [image]
    pub imagedata: DynamicImage,

    /// The used filtering method
    pub filtering: TextureFiltering,

    /// How the texture is wrapped outside of UV range `0-1`
    pub wrapping: TextureWrapping,
}

/// The texture filtering method used in a [TextureData]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum TextureFiltering {
    /// Linear interpolation between the two nearest color values in the texture
    #[default]
    Linear,

    /// The nearest color value
    Nearest,
}

/// Per-axis texture wrapping settings for [TextureData]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum WrappingMethod {
    /// Texture is repeated linearly
    #[default]
    Repeat,

    /// Texture is mirrored
    Mirror,

    /// Texture is clamped
    Clamp,
}
