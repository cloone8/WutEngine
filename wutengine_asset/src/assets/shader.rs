use core::ops::RangeInclusive;
use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::SerializedAsset;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedShader {
    pub name: String,
    pub vertex_attributes: Vec<ShaderVertexAttribute>,

    #[serde(default)]
    pub default_parameters: ShaderDefaultParameters,

    pub keywords: HashMap<String, ShaderKeyword>,
    pub parameters: Vec<ShaderParameter>,
    pub source: ShaderSource,
}

impl SerializedAsset for SerializedShader {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderVertexAttribute {
    #[serde(flatten)]
    pub ty: ShaderVertexAttributeType,
    pub location: u32,
    pub condition: Option<ShaderParameterCondition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum ShaderVertexAttributeType {
    Position,
    Uv { channel: u8 },
}

impl core::fmt::Display for ShaderVertexAttributeType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Position => "Position".fmt(f),
            Self::Uv { channel } => write!(f, "UV{}", channel),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderKeyword {
    default: u64,
    allowed: RangeInclusive<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "lowercase")]
pub enum ShaderParameter {
    Buffer {
        #[serde(rename = "type")]
        ty: ShaderBufferParameterType,

        name: String,

        condition: Option<ShaderParameterCondition>,
    },
    Opaque {
        #[serde(rename = "type")]
        ty: ShaderOpaqueParameterType,

        name: String,

        condition: Option<ShaderParameterCondition>,
    },
}

impl ShaderParameter {
    pub fn get_condition(&self) -> Option<&ShaderParameterCondition> {
        match self {
            Self::Buffer { condition, .. } => condition.as_ref(),
            Self::Opaque { condition, .. } => condition.as_ref(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "lowercase")]
pub enum ShaderSource {
    Inline { content: String },
    File { path: PathBuf },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct ShaderParameterCondition(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShaderDefaultParameters {
    #[serde(default)]
    pub camera: bool,

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShaderBufferParameterType {
    Flt,
    Uint,
    Int,
    Vec2f,
    Vec3f,
    Vec4f,
    Vec2u,
    Vec3u,
    Vec4u,
    Vec2i,
    Vec3i,
    Vec4i,
    Mat4x4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShaderOpaqueParameterType {
    Sampler,
    #[serde(rename = "texture_2d")]
    Texture2D,
}
