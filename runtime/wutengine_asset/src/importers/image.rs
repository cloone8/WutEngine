use core::any::Any;
use core::error::Error;
use std::path::Path;

use image::DynamicImage;
use image::EncodableLayout;
use image::GenericImageView;

use crate::AssetImporter;
use crate::assets::texture::SerializedMipMap;
use crate::assets::texture::SerializedTexture;
use crate::assets::texture::TextureConfig;
use crate::assets::texture::TextureFormat;

/// Image asset importer. Imports images from an encoded format (png, jpg, etc.) into a raw decoded texture
#[derive(Debug)]
pub struct ImageAssetImporter;

/// An error while importing an image
#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub enum ImageImportError {
    /// I/O error
    #[display("I/O error: {_0}")]
    IO(#[from] std::io::Error),

    /// Decoding error
    #[display("Error decoding image: {_0}")]
    Decode(#[from] image::ImageError),

    /// Unsupported pixel format
    #[display("Unsupported pixel format: {_0:?}")]
    UnsupportedPixelFormat(#[error(not(source))] image::ColorType),
}

impl AssetImporter for ImageAssetImporter {
    type AssetType
        = SerializedTexture
    where
        Self: Sized;

    fn supports_file_type(&self, file_type: &str) -> bool {
        image::ImageFormat::from_extension(file_type).is_some()
    }

    fn import(
        &self,
        asset_bytes: &[u8],
        file_type: &str,
        _asset_dir: Option<&Path>,
    ) -> Result<Box<dyn Any>, Box<dyn Error>> {
        profiling::function_scope!();

        log::info!("Importing image of type {file_type}",);

        let image_format = image::ImageFormat::from_extension(file_type)
            .expect("Passed an incompatible image format");

        let mut loaded = {
            profiling::scope!("Import base image");

            image::load_from_memory_with_format(asset_bytes, image_format)?
        };

        {
            profiling::scope!("Fix orientation");
            loaded.apply_orientation(image::metadata::Orientation::FlipVertical);
        }

        let (width, height) = loaded.dimensions();

        // Load main image
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

        // Generate mipmaps
        let mips = if width.is_power_of_two() && width > 1 && height.is_power_of_two() && height > 1
        {
            log::trace!("Generating mips for image");

            //TODO: Use a different filter? When? Why?
            Some(generate_mips(
                &loaded,
                image::imageops::FilterType::Triangle,
                1,
            ))
        } else {
            None
        };

        Ok(Box::new(SerializedTexture {
            config: TextureConfig {
                width,
                height,
                format: pixel_format,
            },
            data: buffer.to_vec(),
            mips,
        }))
    }
}

fn generate_mips(
    image: &DynamicImage,
    filter: image::imageops::FilterType,
    min_side_size: u32,
) -> Vec<SerializedMipMap> {
    let (base_width, base_height) = image.dimensions();

    profiling::function_scope!(format!("{base_width}x{base_height}"));

    assert_ne!(0, min_side_size, "Min side size must be at least 1");
    assert!(
        min_side_size.is_power_of_two(),
        "Min side size must be a power of two"
    );
    assert!(
        base_width.is_power_of_two(),
        "Base width must be a power of two"
    );
    assert!(
        base_height.is_power_of_two(),
        "Base height must be a power of two"
    );

    let mut prev_mip;
    let mut prev_mip_ref = image;

    let smallest_side = u32::min(base_height, base_height);
    let mip_levels = (smallest_side / min_side_size).ilog2();

    let mut mips = Vec::new();

    log::trace!("Generating {mip_levels} mip levels");

    for mip_level in 1..=mip_levels {
        let mip_scale = 2u32.pow(mip_level);
        let mip_width = base_width / mip_scale;
        let mip_height = base_height / mip_scale;

        profiling::scope!(
            "Generate mip level",
            format!("{mip_level}: {mip_width}x{mip_height}")
        );

        let mip = prev_mip_ref.resize_exact(mip_width, mip_height, filter);

        mips.push(SerializedMipMap {
            data: mip.as_bytes().to_vec(),
        });

        prev_mip = Some(mip);
        prev_mip_ref = prev_mip.as_ref().unwrap();
    }

    log::trace!("Generated {} mips", mips.len());

    mips
}
