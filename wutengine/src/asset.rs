//! Asset handling

use core::any::Any;
use std::path::Path;
use std::sync::Arc;

use image::EncodableLayout;
use image::GenericImageView;
use image::ImageFormat;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::graphics::texture::SerializedTexture;
use crate::graphics::texture::TextureConfig;
use crate::graphics::texture::TextureFormat;

/// Trait implemented by types that can be used as a WutEngine asset
pub trait Asset: Send + Sync + Any {
    type Serialized: SerializedAsset<AssetType = Self>;
    type FromSerializedErr: core::error::Error;
    fn from_serialized(serialized: &Self::Serialized) -> Result<Self, Self::FromSerializedErr>
    where
        Self: Sized;
}

pub trait SerializedAsset: Serialize + DeserializeOwned + Any {
    type AssetType: Asset<Serialized = Self>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssetHandle<T> {
    #[serde(skip, default = "default_none")]
    asset: Option<Arc<T>>,
}

const fn default_none<T>() -> Option<Arc<T>> {
    None
}

impl<T> Default for AssetHandle<T> {
    fn default() -> Self {
        Self { asset: None }
    }
}

impl<T: Asset> AssetHandle<T> {
    #[inline(always)]
    pub fn new(asset: impl Into<Self>) -> Self {
        asset.into()
    }

    /// Returns a reference to the asset, if the asset was loaded. Otherwise returns [None]
    #[inline(always)]
    pub fn get_ref(&self) -> Option<&T> {
        self.asset.as_ref().map(Arc::as_ref)
    }

    /// Returns the cloned [Arc] containing the asset, if the asset was loaded. Otherwise returns [None]
    #[inline(always)]
    pub fn get_arc(&self) -> Option<Arc<T>> {
        self.asset.as_ref().map(Arc::clone)
    }
}

impl<T> From<T> for AssetHandle<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self {
            asset: Some(Arc::new(value)),
        }
    }
}

impl<T> From<Option<T>> for AssetHandle<T> {
    #[inline]
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => Self::from(v),
            None => Self { asset: None },
        }
    }
}

// === Code below here should be moved to an editor or similar tool later ===
pub trait AssetImporter<T: SerializedAsset> {
    const SUPPORTED_FILE_TYPES: &[&'static str];
    type Error: core::error::Error;

    fn import(asset_path: &Path, file_type: &str) -> Result<T, Self::Error>;
}

#[derive(Debug)]
pub struct ImageAssetImporter;

#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub enum ImageImportError {
    #[display("I/O error: {_0}")]
    IO(#[from] std::io::Error),

    #[display("Error decoding image: {_0}")]
    Decode(#[from] image::ImageError),

    #[display("Unsupported pixel format: {_0:?}")]
    UnsupportedPixelFormat(#[error(not(source))] image::ColorType),
}

impl AssetImporter<SerializedTexture> for ImageAssetImporter {
    const SUPPORTED_FILE_TYPES: &[&'static str] = &[".jpg", ".jpeg", ".png", ".webp"];

    type Error = ImageImportError;

    fn import(asset_path: &Path, file_type: &str) -> Result<SerializedTexture, Self::Error> {
        let format = match file_type {
            ".jpg" | ".jpeg" => image::ImageFormat::Jpeg,
            ".png" => image::ImageFormat::Png,
            ".webp" => image::ImageFormat::WebP,
            _ => unreachable!("Passed an incompatible image format"),
        };

        let content = std::fs::read(asset_path)?;

        let loaded = image::load_from_memory_with_format(&content, format)?;

        let (width, height) = loaded.dimensions();

        let (format, buffer): (_, &[u8]) = match &loaded {
            image::DynamicImage::ImageRgba8(image_buffer) => (
                if loaded.color_space() == image::metadata::Cicp::SRGB {
                    TextureFormat::Rgba8Srgb
                } else {
                    TextureFormat::Rgba8
                },
                image_buffer.as_bytes(),
            ),
            image::DynamicImage::ImageRgba32F(image_buffer) => {
                (TextureFormat::Rgba32, image_buffer.as_bytes())
            }
            other => return Err(ImageImportError::UnsupportedPixelFormat(other.color())),
        };

        Ok(SerializedTexture {
            config: TextureConfig {
                width,
                height,
                format,
            },
            data: buffer.to_vec(),
        })
    }
}
