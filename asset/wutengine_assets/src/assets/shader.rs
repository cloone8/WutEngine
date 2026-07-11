//! Shader asset

use core::ops::RangeInclusive;
use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};
use wutengine_util_macro::VariantIndex;

use crate::SerializedAsset;

/// The data for a shader
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedShader {
    /// Human-readible name of the shader
    pub name: String,

    /// Vertex attributes required by the shader
    pub vertex_attributes: Vec<ShaderVertexAttribute>,

    /// Which default parameters the shader uses
    #[serde(default)]
    pub default_parameters: ShaderDefaultParameters,

    /// What keywords can be set, and their allowed values
    pub keywords: HashMap<String, ShaderKeyword>,

    /// What parameters the shader exposes
    pub parameters: Vec<ShaderParameter>,

    /// The source code for the shader
    pub source: ShaderSource,
}

impl SerializedAsset for SerializedShader {
    const ID: uuid::NonNilUuid =
        uuid::NonNilUuid::new(uuid::uuid!("32868890-f1de-427b-82f3-6bbb4508484e")).unwrap();
}

/// A vertex attribute used by a shader
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderVertexAttribute {
    /// The type of the attribute
    #[serde(flatten)]
    pub ty: ShaderVertexAttributeType,

    /// The binding location in the shader used by the attribute
    pub location: u32,

    /// Any conditions that need to be true for this attribute to exist
    pub condition: Option<ShaderParameterCondition>,
}

/// The type of a shader vertex attribute
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, VariantIndex,
)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
#[index_repr(u8)]
pub enum ShaderVertexAttributeType {
    /// Position data
    Position,

    /// UV data
    Uv {
        /// The UV channel
        channel: u8,
    },

    /// Color data
    Color,
}

impl ShaderVertexAttributeType {
    /// Returns this attribute as a [u16]
    #[inline]
    pub const fn as_u16(self) -> u16 {
        let channel = if let Self::Uv { channel } = self {
            channel
        } else {
            0
        };

        let variant = self.variant_index();

        ((variant as u16) << 8) | (channel as u16)
    }
}

impl core::hash::Hash for ShaderVertexAttributeType {
    #[inline(always)]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        state.write_u16(self.as_u16());
    }
}

impl nohash_hasher::IsEnabled for ShaderVertexAttributeType {}

impl core::fmt::Display for ShaderVertexAttributeType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Position => "Position".fmt(f),
            Self::Uv { channel } => write!(f, "UV{}", channel),
            Self::Color => "Color".fmt(f),
        }
    }
}

/// A configurable keyword for a shader
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderKeyword {
    /// The default value
    default: u64,

    /// The range of allowed values
    allowed: RangeInclusive<u64>,
}

/// An exposed parameter for a shader
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "lowercase")]
pub enum ShaderParameter {
    /// A buffer parameter. This includes all data types that have a concrete bit-value
    Buffer {
        /// The type of the parameter
        #[serde(rename = "type")]
        ty: ShaderBufferParameterType,

        /// The name of the parameter
        name: String,

        /// What condition needs to be true for this parameter to exist
        condition: Option<ShaderParameterCondition>,
    },

    /// An opaque parameter. This includes all data types that represent opaque handles, like textures, samplers,
    /// etc.
    Opaque {
        /// The type of the parameter
        #[serde(rename = "type")]
        ty: ShaderOpaqueParameterType,

        /// The name of the parameter
        name: String,

        /// What condition needs to be true for this parameter to exist
        condition: Option<ShaderParameterCondition>,
    },
}

impl ShaderParameter {
    /// Returns the condition for this parameter
    pub fn get_condition(&self) -> Option<&ShaderParameterCondition> {
        match self {
            Self::Buffer { condition, .. } => condition.as_ref(),
            Self::Opaque { condition, .. } => condition.as_ref(),
        }
    }
}

/// The source code of a shader
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "lowercase")]
pub enum ShaderSource {
    /// Inline source
    Inline {
        /// The shader WGSL code
        content: String,
    },

    /// Source in another file
    File {
        /// The path to the shader WGSL source file
        path: PathBuf,
    },
}

/// The condition string for a shader parameter
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct ShaderParameterCondition(pub String);

/// The set of shader default parameters
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShaderDefaultParameters {
    /// Uses the per-camera parameter block
    #[serde(default)]
    pub camera: bool,

    /// Uses the per-instance parameter block
    #[serde(default)]
    pub instance: bool,
}

impl Default for ShaderDefaultParameters {
    fn default() -> Self {
        Self {
            camera: true,
            instance: true,
        }
    }
}

/// The type of a shader buffer parameter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShaderBufferParameterType {
    /// 32-bit float
    Flt,

    /// 32-bit uint
    Uint,

    /// 32-bit int
    Int,

    /// 2-float vector
    Vec2f,

    /// 3-float vector
    Vec3f,

    /// 4-float vector
    Vec4f,

    /// 2-uint32 vector
    Vec2u,

    /// 3-uint32 vector
    Vec3u,

    /// 4-uint32 vector
    Vec4u,

    /// 2-int32 vector
    Vec2i,

    /// 3-int32 vector
    Vec3i,

    /// 4-int32 vector
    Vec4i,

    /// 4x4 float matrix
    Mat4x4,
}

/// The type of an opaque shader parameter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShaderOpaqueParameterType {
    /// A texture sampler
    Sampler,

    /// A 2D texture
    #[serde(rename = "texture_2d")]
    Texture2D,
}
