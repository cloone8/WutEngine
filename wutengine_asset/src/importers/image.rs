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
    const SUPPORTED_FILE_TYPES: &[&'static str] = &[".jpg", ".jpeg", ".png", ".webp"];

    type Error = ImageImportError;

    fn import(asset_path: &Path, file_type: &str) -> Result<SerializedTexture, Self::Error> {
        profiling::function_scope!();

        log::info!(
            "Importing image of type {file_type} from path {}",
            asset_path.to_string_lossy()
        );

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
