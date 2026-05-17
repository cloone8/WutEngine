use core::any::Any;
use core::any::TypeId;
use core::error::Error;
use std::path::Path;

use image::EncodableLayout;
use image::GenericImageView;

use crate::AssetImporter;
use crate::assets::texture::SerializedTexture;
use crate::assets::texture::TextureConfig;
use crate::assets::texture::TextureFormat;

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

impl AssetImporter for ImageAssetImporter {
    type AssetType
        = SerializedTexture
    where
        Self: Sized;

    fn supports_file_type(&self, file_type: &str) -> bool {
        match file_type {
            "jpg" | "jpeg" | "png" | "webp" => true,
            _ => false,
        }
    }

    fn import(
        &self,
        asset_bytes: &[u8],
        file_type: &str,
        _asset_dir: Option<&Path>,
    ) -> Result<Box<dyn Any>, Box<dyn Error>> {
        profiling::function_scope!();

        log::info!("Importing image of type {file_type}",);

        let image_format = match file_type {
            "jpg" | "jpeg" => image::ImageFormat::Jpeg,
            "png" => image::ImageFormat::Png,
            "webp" => image::ImageFormat::WebP,
            _ => unreachable!("Passed an incompatible image format"),
        };

        let mut loaded = image::load_from_memory_with_format(asset_bytes, image_format)?;

        loaded.apply_orientation(image::metadata::Orientation::FlipVertical);

        let (width, height) = loaded.dimensions();

        let (pixel_format, buffer): (_, &[u8]) = match &loaded {
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
            other => {
                return Err(Box::new(ImageImportError::UnsupportedPixelFormat(
                    other.color(),
                )));
            }
        };

        Ok(Box::new(SerializedTexture {
            config: TextureConfig {
                width,
                height,
                format: pixel_format,
            },
            data: buffer.to_vec(),
        }))
    }
}
