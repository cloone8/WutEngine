//! Texture samplers

use std::sync::{Arc, LazyLock};

use crate::graphics::GFX_DEVICE;
use crate::graphics::cache::sampler::SamplerCacheKey;

use super::cache;

pub(crate) static DEFAULT_SAMPLER: LazyLock<wgpu::Sampler> = LazyLock::new(|| {
    log::debug!("Loading default sampler");

    let tmp_sampler = Sampler::new(Filtering::Linear, WrapModeType::Single(WrapMode::Repeat));

    tmp_sampler.get_wgpu().clone()
});

/// A texture sampler descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sampler(pub(super) Arc<wgpu::Sampler>);
// /// What filtering method the sampler uses when the texture takes
// /// up more space on the screen than it has pixels
// pub filter: Filtering,

// /// How the sampler treats out-of-bounds accesses
// pub wrapping: WrapModeType,

/// Some predefined common sampler types
impl Sampler {
    // /// Linear filtering sampler that clamps to the texture edge
    // pub const LINEAR_CLAMP: Self = Self {
    //     filter: Filtering::Linear,
    //     wrapping: WrapModeType::Single(WrapMode::Clamp),
    // };

    // /// Linear filtering sampler that repeats the texture
    // pub const LINEAR_REPEAT: Self = Self {
    //     filter: Filtering::Linear,
    //     wrapping: WrapModeType::Single(WrapMode::Repeat),
    // };

    // /// Linear filtering sampler that mirror-repeats the texture edge
    // pub const LINEAR_MIRROR: Self = Self {
    //     filter: Filtering::Linear,
    //     wrapping: WrapModeType::Single(WrapMode::MirrorRepeat),
    // };

    // /// Nearest-neighbour filtering sampler that clamps to the texture edge
    // pub const NEAREST_CLAMP: Self = Self {
    //     filter: Filtering::Nearest,
    //     wrapping: WrapModeType::Single(WrapMode::Clamp),
    // };

    // /// Nearest-neighbour filtering sampler that repeats the texture
    // pub const NEAREST_REPEAT: Self = Self {
    //     filter: Filtering::Nearest,
    //     wrapping: WrapModeType::Single(WrapMode::Repeat),
    // };

    // /// Nearest-neighbour filtering sampler that mirror-repeats the texture edge
    // pub const NEAREST_MIRROR: Self = Self {
    //     filter: Filtering::Nearest,
    //     wrapping: WrapModeType::Single(WrapMode::MirrorRepeat),
    // };
}

impl Sampler {
    pub(crate) fn new(filtering: Filtering, wrapping: WrapModeType) -> Self {
        profiling::function_scope!();

        let cache_key = SamplerCacheKey {
            filtering,
            wrapping,
        };

        if let Some(cached) = cache::sampler::find(&cache_key) {
            return Self(cached);
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

        Self(cache::sampler::insert(cache_key, new_sampler))
    }

    #[inline]
    pub(crate) fn get_wgpu(&self) -> &wgpu::Sampler {
        &self.0
    }
}

/// Filtering methods for a [Sampler]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Filtering {
    /// Linear filtering. Smoothly interpolates between the closest texels.
    #[default]
    Linear,

    /// Nearest neighbour filtering. Chooses the closest texels. Results in a pixelated look
    Nearest,
}

/// Out-of-bounds wrapping modes for a [Sampler]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
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
