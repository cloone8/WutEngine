//! Texture functionality

use std::sync::LazyLock;

use wutengine_asset::assets::texture::TextureConfig;
use wutengine_asset::assets::texture::TextureFormat;

use crate::asset::Asset;

/// The default texture. Used for missing texture parameters
pub(crate) static DEFAULT_TEXTURE: LazyLock<Texture> = LazyLock::new(|| {
    log::debug!("Loading default texture");

    let tex = Texture::new(&TextureConfig {
        width: 512,
        height: 512,
        format: TextureFormat::Rgba8Srgb,
    });

    let image_encoded_bytes = include_bytes!("default_texture.png");
    let image_loaded = image::load_from_memory(image_encoded_bytes).unwrap();

    let as_rgba8 = image_loaded.into_rgba8();

    tex.set_data(&as_rgba8);

    tex
});

/// The handle to a native [wgpu::Texture]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Texture {
    tex: wgpu::Texture,
    view: wgpu::TextureView,
}

impl Asset for Texture {
    type Serialized = wutengine_asset::assets::texture::SerializedTexture;

    type FromSerializedErr = image::ImageError;

    fn from_serialized(serialized: &Self::Serialized) -> Result<Self, Self::FromSerializedErr>
    where
        Self: Sized,
    {
        let texture = Texture::new(&serialized.config);

        let image_loaded = image::load_from_memory(&serialized.data)?;

        //TODO: Check if the loaded image is actually the format as declared in `serialized.config`
        texture.set_data(image_loaded.as_bytes());

        Ok(texture)
    }
}

impl Texture {
    /// Creates a new texture with the given format, without initial content
    pub(crate) fn new(config: &TextureConfig) -> Self {
        assert!(config.width >= 1, "Width must be at least 1");
        assert!(config.height >= 1, "Height must be at least 1");

        let format_wgpu = convert_texture_format(config.format);

        let tex = super::device().create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: format_wgpu,
            usage: wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[
                format_wgpu.add_srgb_suffix(),
                format_wgpu.remove_srgb_suffix(),
            ],
        });

        Self {
            view: tex.create_view(&wgpu::TextureViewDescriptor::default()),
            tex,
        }
    }

    /// Converts an existing texture view to a WutEngine texture
    pub(crate) fn new_from_existing(view: wgpu::TextureView) -> Self {
        Self {
            tex: view.texture().clone(),
            view,
        }
    }

    /// Updates the data in this texture to the provided bytes. The bytes must
    /// be in the format required by the texture format given during texture creation
    pub(crate) fn set_data(&self, data: &[u8]) {
        //TODO: Check somehow if data is the correct length
        let size = self.tex.size();
        let format = self.tex.format();
        let queue = super::queue();

        let bytes_per_pixel = format
            .block_copy_size(None)
            .expect("Compressed texture formats not yet supported");

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &self.tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_pixel * size.width),
                rows_per_image: None,
            },
            size,
        );
    }

    /// Returns the [wgpu::TextureView] associated with this texture
    #[inline]
    pub(crate) const fn get_view(&self) -> &wgpu::TextureView {
        &self.view
    }
}

pub const fn convert_texture_format(asset_format: TextureFormat) -> wgpu::TextureFormat {
    match asset_format {
        TextureFormat::Rgba8 => wgpu::TextureFormat::Rgba8Unorm,
        TextureFormat::Rgba8Srgb => wgpu::TextureFormat::Rgba8UnormSrgb,
        TextureFormat::Rgba32 => wgpu::TextureFormat::Rgba32Float,
    }
}
