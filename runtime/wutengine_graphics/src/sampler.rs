//! Texture samplers

use alloc::sync::Arc;
use core::convert::Infallible;
use core::fmt::Debug;
use std::sync::LazyLock;
use wutengine_assets::FromSerializedAsset;
use wutengine_assets::assets::sampler::FilterMode;
use wutengine_assets::assets::sampler::SerializedSampler;
use wutengine_assets::assets::sampler::WrapMode;
use wutengine_assets::assets::sampler::WrapModeType;

use crate::GFX_DEVICE;
use crate::cache::sampler::SamplerCacheKey;
use crate::label;

use super::cache;

/// The default sampler. Linear-repeat
pub(crate) static DEFAULT_SAMPLER: LazyLock<Sampler> = LazyLock::new(|| {
    log::debug!("Loading default sampler");

    Sampler::new(
        FilterMode::Linear,
        FilterMode::Linear,
        WrapModeType::Single(WrapMode::Repeat),
    )
});

/// A texture sampler descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sampler {
    tex_filtering: FilterMode,
    mip_filtering: FilterMode,
    wrapping: WrapModeType,
    native: Arc<wgpu::Sampler>,
}

impl FromSerializedAsset for Sampler {
    type Error = Infallible;

    type Serialized = SerializedSampler;

    fn from_serialized_asset(serialized: Self::Serialized) -> Result<Self, Self::Error> {
        Ok(Sampler::new(
            serialized.texture_filtering,
            serialized.mipmap_filtering,
            serialized.wrapping,
        ))
    }
}

macro_rules! predefined_sampler {
    ($filtering:expr, $wrapping:expr) => {{
        static SAMPLER: ::std::sync::LazyLock<Sampler> = ::std::sync::LazyLock::new(|| {
            let filt = $filtering;
            let wrap = WrapModeType::Single($wrapping);

            log::debug!("Loading predefined sampler: {}, {}", filt, wrap);

            Sampler::new(filt, filt, wrap)
        });

        &SAMPLER
    }};
}

/// Some predefined common sampler types
impl Sampler {
    /// Linear filtering sampler that clamps to the texture edge
    pub fn linear_clamp() -> &'static Self {
        predefined_sampler!(FilterMode::Linear, WrapMode::Clamp)
    }

    /// Linear filtering sampler that repeats the texture
    pub fn linear_repeat() -> &'static Self {
        predefined_sampler!(FilterMode::Linear, WrapMode::Repeat)
    }

    /// Linear filtering sampler that mirror-repeats the texture edge
    pub fn linear_mirror() -> &'static Self {
        predefined_sampler!(FilterMode::Linear, WrapMode::MirrorRepeat)
    }

    /// Nearest-neighbour filtering sampler that clamps to the texture edge
    pub fn nearest_clamp() -> &'static Self {
        predefined_sampler!(FilterMode::Nearest, WrapMode::Clamp)
    }

    /// Nearest-neighbour filtering sampler that repeats the texture
    pub fn nearest_repeat() -> &'static Self {
        predefined_sampler!(FilterMode::Nearest, WrapMode::Repeat)
    }

    /// Nearest-neighbour filtering sampler that mirror-repeats the texture edge
    pub fn nearest_mirror() -> &'static Self {
        predefined_sampler!(FilterMode::Nearest, WrapMode::MirrorRepeat)
    }
}

impl Sampler {
    /// Creates a new sampler with the given sampling mode
    pub fn new(
        tex_filtering: FilterMode,
        mip_filtering: FilterMode,
        wrapping: WrapModeType,
    ) -> Self {
        let cache_key = SamplerCacheKey {
            tex_filtering,
            mip_filtering,
            wrapping,
        };

        if let Some(cached) = cache::sampler::find(&cache_key) {
            return Self {
                tex_filtering,
                mip_filtering,
                wrapping,
                native: cached,
            };
        };

        profiling::scope!("Create new sampler");

        log::debug!("Creating new sampler object");

        let tex_filter = asset_filter_mode_to_wgpu_filter_mode(tex_filtering);
        let mip_filter = asset_filter_mode_to_wgpu_mipmap_filter_mode(mip_filtering);

        let desc = wgpu::wgt::SamplerDescriptor {
            label: label!(),
            address_mode_u: asset_wrap_mode_to_wgpu(wrapping.get_u()),
            address_mode_v: asset_wrap_mode_to_wgpu(wrapping.get_v()),
            address_mode_w: asset_wrap_mode_to_wgpu(wrapping.get_w()),
            mag_filter: tex_filter,
            min_filter: tex_filter,
            mipmap_filter: mip_filter,
            ..Default::default()
        };

        let new_sampler = GFX_DEVICE.create_sampler(&desc);

        Self {
            tex_filtering,
            mip_filtering,
            wrapping,
            native: cache::sampler::insert(cache_key, new_sampler),
        }
    }

    /// Returns the [`wgpu::Sampler`] matching this sampler object
    #[inline]
    pub fn get_wgpu(&self) -> &wgpu::Sampler {
        &self.native
    }
}

/// Converts the wrapping mode to a [`wgpu::AddressMode`]
pub const fn asset_wrap_mode_to_wgpu(asset_wrap_mode: WrapMode) -> wgpu::AddressMode {
    match asset_wrap_mode {
        WrapMode::Clamp => wgpu::AddressMode::ClampToEdge,
        WrapMode::Repeat => wgpu::AddressMode::Repeat,
        WrapMode::MirrorRepeat => wgpu::AddressMode::MirrorRepeat,
    }
}

/// Converts the filtering mode to a [`wgpu::FilterMode`]
pub const fn asset_filter_mode_to_wgpu_filter_mode(
    asset_filter_mode: FilterMode,
) -> wgpu::FilterMode {
    match asset_filter_mode {
        FilterMode::Linear => wgpu::FilterMode::Linear,
        FilterMode::Nearest => wgpu::FilterMode::Nearest,
    }
}

/// Converts the filtering mode to a [`wgpu::MipmapFilterMode`]
pub const fn asset_filter_mode_to_wgpu_mipmap_filter_mode(
    asset_filter_mode: FilterMode,
) -> wgpu::MipmapFilterMode {
    match asset_filter_mode {
        FilterMode::Linear => wgpu::MipmapFilterMode::Linear,
        FilterMode::Nearest => wgpu::MipmapFilterMode::Nearest,
    }
}
