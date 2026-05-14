//! Texture samplers

use core::fmt::{Debug, Display};
use std::convert::Infallible;
use std::sync::{Arc, LazyLock};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::asset::{Asset, SerializedAsset};
use crate::graphics::GFX_DEVICE;
use crate::graphics::cache::sampler::SamplerCacheKey;

use super::cache;

/// The default sampler. Linear-repeat
pub(crate) static DEFAULT_SAMPLER: LazyLock<Sampler> = LazyLock::new(|| {
    log::debug!("Loading default sampler");

    Sampler::new(Filtering::Linear, WrapModeType::Single(WrapMode::Repeat))
});

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SerializedSampler {
    pub filtering: Filtering,
    pub wrapping: WrapModeType,
}

impl SerializedAsset for SerializedSampler {
    type AssetType = Sampler;
}

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
            address_mode_u: wrapping.get_u().to_wgpu(),
            address_mode_v: wrapping.get_v().to_wgpu(),
            address_mode_w: wrapping.get_w().to_wgpu(),
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

/// Filtering methods for a [Sampler]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, derive_more::Display, Serialize, Deserialize,
)]
pub enum Filtering {
    /// Linear filtering. Smoothly interpolates between the closest texels.
    #[default]
    Linear,

    /// Nearest neighbour filtering. Chooses the closest texels. Results in a pixelated look
    Nearest,
}

/// Out-of-bounds wrapping modes for a [Sampler]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WrapModeType {
    /// One wrapping mode for each axis
    Single(WrapMode),

    /// A seperate wrapping mode for each axis
    PerAxis {
        /// Wrapping in the U (X) axis
        u: WrapMode,

        /// Wrapping in the V (Y) axis
        v: WrapMode,

        /// Wrapping in the W (Z) axis
        w: WrapMode,
    },
}

impl Display for WrapModeType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Single(wrap_mode) => Display::fmt(wrap_mode, f),
            Self::PerAxis { u, v, w } => write!(f, "u: {u}, v: {v}, w: {w}"),
        }
    }
}

impl Default for WrapModeType {
    fn default() -> Self {
        Self::Single(Default::default())
    }
}

impl WrapModeType {
    /// Returns the wrapping mode for the U axis
    #[inline]
    pub const fn get_u(self) -> WrapMode {
        match self {
            Self::Single(wrap_mode) => wrap_mode,
            Self::PerAxis { u, .. } => u,
        }
    }

    /// Returns the wrapping mode for the V axis
    #[inline]
    pub const fn get_v(self) -> WrapMode {
        match self {
            Self::Single(wrap_mode) => wrap_mode,
            Self::PerAxis { v, .. } => v,
        }
    }

    /// Returns the wrapping mode for the W axis
    #[inline]
    pub const fn get_w(self) -> WrapMode {
        match self {
            Self::Single(wrap_mode) => wrap_mode,
            Self::PerAxis { w, .. } => w,
        }
    }
}

/// A wrapping more for [Sampler] out-of-bounds accesses
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, derive_more::Display, Serialize, Deserialize,
)]
pub enum WrapMode {
    /// Clamp to the border pixel
    #[default]
    Clamp,

    /// Repeat the texture
    Repeat,

    /// Repeat the texture, but mirrors the texture each repetition
    MirrorRepeat,
}

impl WrapMode {
    /// Converts the wrapping mode to a [wgpu::AddressMode]
    const fn to_wgpu(self) -> wgpu::AddressMode {
        match self {
            Self::Clamp => wgpu::AddressMode::ClampToEdge,
            Self::Repeat => wgpu::AddressMode::Repeat,
            Self::MirrorRepeat => wgpu::AddressMode::MirrorRepeat,
        }
    }
}
