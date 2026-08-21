//! Shader asset

use core::fmt::Display;
use core::num::ParseIntError;
use core::ops::RangeInclusive;
use core::str::FromStr;
use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
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

    /// Normal vector data
    Normal,

    /// UV data
    Uv {
        /// The UV channel
        channel: u8,
    },

    /// Color data
    Color,
}

impl ShaderVertexAttributeType {
    /// Returns this attribute as a [`u16`]
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
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        state.write_u16(self.as_u16());
    }
}

impl nohash_hasher::IsEnabled for ShaderVertexAttributeType {}

impl core::fmt::Display for ShaderVertexAttributeType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Position => "Position".fmt(f),
            Self::Normal => "Normal".fmt(f),
            Self::Uv { channel } => write!(f, "UV{channel}"),
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

    /// A raw uniform buffer
    UniformBuffer,

    /// A raw read-only storage buffer
    ReadStorageBuffer,

    /// A raw read-write storage buffer
    RWStorageBuffer,
}

/// The data for a shader
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrecompiledShader {
    pub hash: ShaderHash,

    /// The raw parsed module
    pub module: Box<naga::Module>,

    /// The parameters
    pub parameters: Vec<Parameter>,
}

/// Thin wrapper over a 128-bit shader hash
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ShaderHash(pub u128);

impl ShaderHash {
    /// Returns the hash as an integer
    #[inline]
    pub const fn as_int(self) -> u128 {
        self.0
    }
}

impl Display for ShaderHash {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:032x}", self.0)
    }
}

impl From<u128> for ShaderHash {
    #[inline]
    fn from(value: u128) -> Self {
        Self(value)
    }
}

impl TryFrom<&str> for ShaderHash {
    type Error = ParseIntError;

    #[inline]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        u128::from_str_radix(value, 16).map(Self)
    }
}

impl Serialize for ShaderHash {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_string())
        } else {
            serializer.serialize_u128(self.0)
        }
    }
}

impl<'de> Deserialize<'de> for ShaderHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            u128::from_str_radix(<&str as Deserialize>::deserialize(deserializer)?, 16)
                .map_err(|e| serde::de::Error::custom(format!("Failed to parse u128: {e}")))
                .map(Self)
        } else {
            u128::deserialize(deserializer).map(Self)
        }
    }
}

impl SerializedAsset for PrecompiledShader {
    const ID: uuid::NonNilUuid =
        uuid::NonNilUuid::new(uuid::uuid!("a7caf7c7-59b4-4b91-b3aa-61f32944a8ac")).unwrap();

    const PREFER_BINARY_SERIALIZATION: bool = true;
}

/// Information on an exposed parameter in a shader
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Parameter {
    /// An opaque parameter, requiring a resource binding
    Opaque {
        /// The name of the parameter
        name: String,

        /// The parameter group
        group: u32,
        /// The parameter binding
        binding: u32,

        /// The base type
        base_type: OpaqueBaseType,
    },

    /// An in-buffer parameter, residing within a location in a buffer
    BufferMember {
        /// The name of the parameter
        name: String,

        /// The parameter group
        group: u32,
        /// The parameter binding
        binding: u32,

        /// The base type
        base_type: BufferBaseType,

        /// The size in bytes of the base type
        base_size: u32,

        /// If an array, the array length
        array_length: u32,

        /// The offset within the buffer (in bytes)
        offset: u32,

        /// The size of the member in the buffer (in bytes)
        size: u32,
    },
}

/// Base types for opaque parameters
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OpaqueBaseType {
    /// A sampler
    Sampler,

    /// An image
    Image,
}

/// Base types for buffer parameters
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BufferBaseType {
    /// A complex, unsupported type
    Complex = 0,

    /// Float
    Float,

    /// Signed integer
    Sint,

    /// Unsigned integer
    Uint,

    /// Boolean
    Bool,

    /// 2-component float vector
    Vec2f,
    /// 2-component float vector
    Vec3f,
    /// 4-component float vector
    Vec4f,

    /// 2-component signed int vector
    Vec2i,
    /// 3-component signed int vector
    Vec3i,
    /// 4-component signed int vector
    Vec4i,

    /// 2-component unsigned int vector
    Vec2u,
    /// 3-component unsigned int vector
    Vec3u,
    /// 4-component unsigned int vector
    Vec4u,

    /// 2-component bool vector
    Vec2b,
    /// 3-component bool vector
    Vec3b,
    /// 4-component bool vector
    Vec4b,

    /// 2x2 float vector
    Mat2x2,
    /// 2x3 float vector
    Mat2x3,
    /// 2x4 float vector
    Mat2x4,
    /// 3x2 float vector
    Mat3x2,
    /// 3x3 float vector
    Mat3x3,
    /// 3x4 float vector
    Mat3x4,
    /// 4x2 float vector
    Mat4x2,
    /// 4x3 float vector
    Mat4x3,
    /// 4x4 float vector
    Mat4x4,
}
