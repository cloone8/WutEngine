//! Sampler asset

use core::fmt::Display;

use crate::SerializedAsset;

/// Data for a sampler
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
pub struct SerializedSampler {
    /// Filtering mode the sampler uses for filtering in the same mip level
    pub texture_filtering: FilterMode,

    /// Filtering mode the sampler uses for filtering between different mip levels
    pub mipmap_filtering: FilterMode,

    /// Wrapping mode the sampler uses
    pub wrapping: WrapModeType,
}

impl SerializedAsset for SerializedSampler {
    const ID: uuid::NonNilUuid =
        uuid::NonNilUuid::new(uuid::uuid!("e39313cc-5ee5-4d8d-9ea3-51638a0dbc3e")).unwrap();
}

/// Filtering methods for a [SerializedSampler]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, derive_more::Display)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
pub enum FilterMode {
    /// Linear filtering. Smoothly interpolates between the closest texels.
    #[default]
    Linear,

    /// Nearest neighbour filtering. Chooses the closest texels. Results in a pixelated look
    Nearest,
}

/// Out-of-bounds wrapping modes for a [SerializedSampler]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, derive_more::Display)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize))]
#[cfg_attr(feature = "deserialize", derive(serde::Deserialize))]
pub enum WrapMode {
    /// Clamp to the border pixel
    #[default]
    Clamp,

    /// Repeat the texture
    Repeat,

    /// Repeat the texture, but mirrors the texture each repetition
    MirrorRepeat,
}
