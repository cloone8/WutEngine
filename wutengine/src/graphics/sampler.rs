//! Texture samplers

use alloc::sync::Arc;
use core::convert::Infallible;
use core::fmt::Debug;
use std::sync::LazyLock;
use wutengine_asset::assets::sampler::Filtering;
use wutengine_asset::assets::sampler::SerializedSampler;
use wutengine_asset::assets::sampler::WrapMode;
use wutengine_asset::assets::sampler::WrapModeType;

use crate::asset::Asset;
use crate::graphics::GFX_DEVICE;
use crate::graphics::cache::sampler::SamplerCacheKey;

use super::cache;

/// The default sampler. Linear-repeat
pub(crate) static DEFAULT_SAMPLER: LazyLock<Sampler> = LazyLock::new(|| {
    log::debug!("Loading default sampler");

    Sampler::new(Filtering::Linear, WrapModeType::Single(WrapMode::Repeat))
});

/// A texture sampler descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sampler {
    filtering: Filtering,
    wrapping: WrapModeType,
    native: Arc<wgpu::Sampler>,
}

impl Asset for Sampler {
    type Serialized = SerializedSampler;

    type FromSerializedErr = Infallible;

    fn from_serialized(serialized: &Self::Serialized) -> Result<Self, Self::FromSerializedErr>
    where
        Self: Sized,
    {
        Ok(Sampler::new(serialized.filtering, serialized.wrapping))
    }
}

macro_rules! predefined_sampler {
    ($filtering:expr, $wrapping:expr) => {{
        static SAMPLER: ::std::sync::LazyLock<Sampler> = ::std::sync::LazyLock::new(|| {
            let filt = $filtering;
            let wrap = WrapModeType::Single($wrapping);

            log::debug!("Loading predefined sampler: {}, {}", filt, wrap);

            Sampler::new(filt, wrap)
        });

        &SAMPLER
    }};
}

/// Some predefined common sampler types
impl Sampler {
    /// Linear filtering sampler that clamps to the texture edge
    pub fn linear_clamp() -> &'static Self {
        predefined_sampler!(Filtering::Linear, WrapMode::Clamp)
    }

    /// Linear filtering sampler that repeats the texture
    pub fn linear_repeat() -> &'static Self {
        predefined_sampler!(Filtering::Linear, WrapMode::Repeat)
    }

    /// Linear filtering sampler that mirror-repeats the texture edge
    pub fn linear_mirror() -> &'static Self {
        predefined_sampler!(Filtering::Linear, WrapMode::MirrorRepeat)
    }

    /// Nearest-neighbour filtering sampler that clamps to the texture edge
    pub fn nearest_clamp() -> &'static Self {
        predefined_sampler!(Filtering::Nearest, WrapMode::Clamp)
    }

    /// Nearest-neighbour filtering sampler that repeats the texture
    pub fn nearest_repeat() -> &'static Self {
        predefined_sampler!(Filtering::Nearest, WrapMode::Repeat)
    }

    /// Nearest-neighbour filtering sampler that mirror-repeats the texture edge
    pub fn nearest_mirror() -> &'static Self {
        predefined_sampler!(Filtering::Nearest, WrapMode::MirrorRepeat)
    }
}

impl Sampler {
    /// Creates a new sampler with the given sampling mode
    pub(crate) fn new(filtering: Filtering, wrapping: WrapModeType) -> Self {
        profiling::function_scope!();

        let cache_key = SamplerCacheKey {
            filtering,
            wrapping,
        };

        if let Some(cached) = cache::sampler::find(&cache_key) {
            return Self {
                filtering,
                wrapping,
                native: cached,
            };
        };

        log::debug!("Creating new sampler object");

        let desc = wgpu::wgt::SamplerDescriptor {
            label: None,
            address_mode_u: asset_wrap_mode_to_wgpu(wrapping.get_u()),
            address_mode_v: asset_wrap_mode_to_wgpu(wrapping.get_v()),
            address_mode_w: asset_wrap_mode_to_wgpu(wrapping.get_w()),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        };

        let new_sampler = GFX_DEVICE.create_sampler(&desc);

        Self {
            filtering,
            wrapping,
            native: cache::sampler::insert(cache_key, new_sampler),
        }
    }

    /// Returns the [wgpu::Sampler] matching this sampler object
    #[inline]
    pub(crate) fn get_wgpu(&self) -> &wgpu::Sampler {
        &self.native
    }
}

/// Converts the wrapping mode to a [wgpu::AddressMode]
pub const fn asset_wrap_mode_to_wgpu(asset_wrap_mode: WrapMode) -> wgpu::AddressMode {
    match asset_wrap_mode {
        WrapMode::Clamp => wgpu::AddressMode::ClampToEdge,
        WrapMode::Repeat => wgpu::AddressMode::Repeat,
        WrapMode::MirrorRepeat => wgpu::AddressMode::MirrorRepeat,
    }
}
