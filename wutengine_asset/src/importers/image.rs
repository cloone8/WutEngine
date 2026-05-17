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

impl AssetImporter<SerializedTexture> for ImageAssetImporter {
    type Error = ImageImportError;

    fn supports_file_type(file_type: &str) -> bool {
        match file_type {
            "jpg" | "jpeg" | "png" | "webp" => true,
            _ => false,
        }
    }

    fn import(
        asset_bytes: &[u8],
        file_type: &str,
        _asset_dir: Option<&Path>,
    ) -> Result<SerializedTexture, Self::Error> {
        profiling::function_scope!();

        log::info!("Importing image of type {file_type}",);

        let image_format = match file_type {
            "jpg" | "jpeg" => image::ImageFormat::Jpeg,
            "png" => image::ImageFormat::Png,
            "webp" => image::ImageFormat::WebP,
            _ => unreachable!("Passed an incompatible image format"),
        };

        let loaded = image::load_from_memory_with_format(asset_bytes, image_format)?;

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
            other => return Err(ImageImportError::UnsupportedPixelFormat(other.color())),
        };

        Ok(SerializedTexture {
            config: TextureConfig {
                width,
                height,
                format: pixel_format,
            },
            data: buffer.to_vec(),
        })
    }
}
