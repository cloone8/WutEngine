//! Texture functionality

use core::convert::Infallible;
use std::sync::LazyLock;

use wutengine_assets::{
    FromSerializedAsset,
    assets::texture::{SerializedTexture, TextureConfig, TextureFormat},
};

use crate::label;

/// The default texture. Used for missing texture parameters
pub(crate) static DEFAULT_TEXTURE: LazyLock<Texture> = LazyLock::new(|| {
    log::debug!("Loading default texture");

    let tex = Texture::new(
        &TextureConfig {
            width: 512,
            height: 512,
            format: TextureFormat::Rgba8Srgb,
        },
        1,
    );

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

impl FromSerializedAsset for Texture {
    type Error = Infallible;

    type Serialized = SerializedTexture;

    fn from_serialized_asset(serialized: Self::Serialized) -> Result<Self, Self::Error> {
        let mip_count = match &serialized.mips {
            Some(mips) => (mips.len() + 1) as u32,
            None => 1,
        };

        let texture = Texture::new(&serialized.config, mip_count);

        //TODO: Check if the loaded image is actually the format as declared in `serialized.config`
        texture.set_data_at_mip(&serialized.data, 0);

        if mip_count > 1 {
            let mips = serialized.mips.as_ref().unwrap();

            for (i, mip) in mips.iter().enumerate() {
                let mip_level = (i + 1) as u32;

                texture.set_data_at_mip(&mip.data, mip_level);
            }
        }

        Ok(texture)
    }
}

impl Texture {
    /// Creates a new texture with the given format, without initial content
    pub fn new(config: &TextureConfig, mip_levels: u32) -> Self {
        profiling::function_scope!();

        assert!(config.width >= 1, "Width must be at least 1");
        assert!(config.height >= 1, "Height must be at least 1");

        //TODO: Check if given mip levels make sense

        let format_wgpu = convert_texture_format(config.format);

        let tex = super::device().create_texture(&wgpu::TextureDescriptor {
            label: label!(),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: mip_levels,
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
    pub fn new_from_existing(view: wgpu::TextureView) -> Self {
        Self {
            tex: view.texture().clone(),
            view,
        }
    }

    /// Updates the data in this texture to the provided bytes. The bytes must
    /// be in the format required by the texture format given during texture creation
    ///
    /// Updates the entire texture. To update a subregion, see [Self::set_partial_data]
    #[inline]
    pub fn set_data(&self, data: &[u8]) {
        self.set_data_at_mip(data, 0);
    }

    /// Updates the data in this texture at the given mip level to the provided bytes. The bytes must
    /// be in the format required by the texture format given during texture creation
    ///
    /// Updates the entire texture. To update a subregion, see [Self::set_partial_data_at_mip]
    pub fn set_data_at_mip(&self, data: &[u8], mip_level: u32) {
        profiling::function_scope!();

        let base_size = self.tex.size();
        let mip_scale = 2u32.pow(mip_level);

        let mip_size = wgpu::Extent3d {
            width: base_size.width / mip_scale,
            height: base_size.height / mip_scale,
            depth_or_array_layers: base_size.depth_or_array_layers, // TODO: This only works for 2D textures because we do not mip the depth
        };

        self.set_partial_data_at_mip(data, wgpu::Origin3d::ZERO, mip_size, mip_level);
    }

    /// Updates a subregion of the data in this texture to the provided bytes. The bytes must
    /// be in the format required by the texture format given during texture creation
    ///
    /// Updates the given subregion of the texture. To update the full texture, see [Self::set_data]
    #[inline]
    pub fn set_partial_data(&self, data: &[u8], origin: wgpu::Origin3d, size: wgpu::Extent3d) {
        self.set_partial_data_at_mip(data, origin, size, 0);
    }

    /// Updates a subregion of the data in this texture at the given mip level to the provided bytes. The bytes must
    /// be in the format required by the texture format given during texture creation
    ///
    /// Updates the given subregion of the texture. To update the full texture, see [Self::set_data_at_mip]
    pub fn set_partial_data_at_mip(
        &self,
        data: &[u8],
        origin: wgpu::Origin3d,
        size: wgpu::Extent3d,
        mip_level: u32,
    ) {
        profiling::function_scope!();

        let max_mips = self.tex.mip_level_count();

        if mip_level > max_mips {
            log::error!(
                "Cannot set data for mip level {mip_level} for a texture with a maximum of {max_mips} mip levels"
            );
            return;
        }

        //TODO: Check somehow if data is the correct length
        let format = self.tex.format();
        let queue = super::queue();

        let bytes_per_pixel = format
            .block_copy_size(None)
            .expect("Compressed texture formats not yet supported");

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &self.tex,
                mip_level,
                origin,
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

/// Converts a [WutEngine texture format](wutengine_assets::assets::texture::TextureFormat) to a [wgpu::TextureFormat]
pub const fn convert_texture_format(asset_format: TextureFormat) -> wgpu::TextureFormat {
    match asset_format {
        TextureFormat::Rgba8 => wgpu::TextureFormat::Rgba8Unorm,
        TextureFormat::Rgba8Srgb => wgpu::TextureFormat::Rgba8UnormSrgb,
        TextureFormat::Rgba32 => wgpu::TextureFormat::Rgba32Float,
    }
}
