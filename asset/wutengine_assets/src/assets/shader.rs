//! Shader asset

use core::fmt::Display;
use core::num::ParseIntError;

use nohash_hasher::IntMap;
use serde::Deserialize;
use serde::Serialize;
use wutengine_util_macro::VariantIndex;

use crate::SerializedAsset;

/// The source of a shader, in the WutEngine shader format (WGSL plus `#` directives)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedShader {
    /// The shader source code
    pub source: String,
}

impl SerializedAsset for SerializedShader {
    const ID: uuid::NonNilUuid =
        uuid::NonNilUuid::new(uuid::uuid!("32868890-f1de-427b-82f3-6bbb4508484e")).unwrap();
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

/// A single compiled variant of a shader
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrecompiledShader {
    /// The hash identifying this variant
    pub hash: ShaderHash,

    /// The shader name, from its `#name` directive
    pub name: String,

    /// The vertex-stage inputs, by location
    pub vertex_inputs: IntMap<u32, VertexInput>,

    /// The resource bindings
    pub bindings: Vec<Binding>,

    /// The raw parsed module
    pub module: Box<naga::Module>,
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

/// A resource binding of a shader
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Binding {
    /// The name of the global variable
    pub name: String,

    /// The bind group index
    pub group: u32,

    /// The binding index within the group
    pub binding: u32,

    /// What is bound
    pub kind: BindingKind,
}

/// The kind of resource in a [`Binding`]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BindingKind {
    /// A uniform or storage buffer
    Buffer {
        /// Uniform or storage
        space: BufferSpace,

        /// The size of the bound type in bytes
        size: u32,

        /// The members of the buffer. A struct is flattened one level, any other type is a single member at
        /// offset 0 named after the binding
        members: Vec<BufferMember>,
    },

    /// A sampler
    Sampler {
        /// Whether this is a comparison sampler
        comparison: bool,
    },

    /// A sampled texture
    Texture {
        /// The texture dimension
        dimension: TextureDimension,

        /// Whether this is a texture array
        arrayed: bool,

        /// The type of the sampled texels
        sample_type: TextureSampleType,

        /// Whether the texture is multisampled
        multisampled: bool,
    },
}

/// The address space of a buffer binding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BufferSpace {
    /// A uniform buffer
    Uniform,

    /// A storage buffer
    Storage {
        /// Whether the shader can write to it
        writable: bool,
    },
}

/// A value within a buffer binding
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BufferMember {
    /// The name of the member
    pub name: String,

    /// The base type
    pub base_type: BufferBaseType,

    /// If an array, the array length. 1 otherwise
    pub array_length: u32,

    /// The offset within the buffer (in bytes)
    pub offset: u32,

    /// The size of the member in the buffer (in bytes)
    pub size: u32,
}

/// The dimension of a texture binding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextureDimension {
    /// 1D
    D1,
    /// 2D
    D2,
    /// 3D
    D3,
    /// Cube map
    Cube,
}

/// The type of the texels sampled from a texture binding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextureSampleType {
    /// Floating point
    Float,
    /// Signed integer
    Sint,
    /// Unsigned integer
    Uint,
    /// Depth
    Depth,
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

/// Input for the vertex stage of the shager
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VertexInput {
    /// The attribute
    pub attribute: ShaderVertexAttributeType,
}
