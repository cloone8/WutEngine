//! GPU Shaders

use core::convert::Infallible;
use core::fmt::Display;
use core::hash::Hash;
use core::ops::RangeInclusive;
use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use wutengine_util_macro::unique_id_type64;

mod compile;
mod types;

pub use types::*;

pub(crate) use compile::*;

use crate::asset::{Asset, SerializedAsset};

unique_id_type64! {
    /// Unique identifier for a [Shader]
    pub(crate) ShaderId
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shader {
    #[serde(skip)]
    pub(crate) id: ShaderId,
    pub(crate) name: String,
    pub(crate) vertex_attributes: Vec<ShaderVertexAttribute>,

    #[serde(default)]
    pub(crate) default_parameters: ShaderDefaultParameters,

    pub(crate) keywords: HashMap<String, ShaderKeyword>,
    pub(crate) parameters: Vec<ShaderParameter>,
    pub(crate) source: ShaderSource,
}

impl Asset for Shader {
    type Serialized = Self;

    type FromSerializedErr = Infallible;

    fn from_serialized(serialized: &Self::Serialized) -> Result<Self, Self::FromSerializedErr>
    where
        Self: Sized,
    {
        Ok(serialized.clone())
    }
}

impl SerializedAsset for Shader {
    type AssetType = Self;
}

impl Shader {
    pub fn load_source(&mut self) -> Result<(), std::io::Error> {
        if let ShaderSource::File { path } = &self.source {
            let content = std::fs::read_to_string(path)?;
            self.source = ShaderSource::Inline { content };
        }

        Ok(())
    }

    pub fn get_source(&self) -> &str {
        if let ShaderSource::Inline { content } = &self.source {
            content.as_str()
        } else {
            panic!("Invalid source");
        }
    }
}

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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
pub struct ShaderParameterCondition(pub(crate) String);

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

/// Unique ID for a [CompiledShader]. Generated based on the source [Shader] and the active keywords
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct CompiledShaderId(pub(crate) u128);

impl CompiledShaderId {
    /// Constructs a new [CompiledShaderId] based on the hashes of the source and keywords
    #[inline]
    pub const fn from_hashes(source_shader_hash: u64, keyword_hash: u64) -> Self {
        Self(((source_shader_hash as u128) << 64) | (keyword_hash as u128))
    }

    #[inline]
    pub const fn source_hash(self) -> u64 {
        (self.0 >> 64) as u64
    }

    #[inline]
    pub const fn keyword_hash(self) -> u64 {
        self.0 as u64
    }
}

impl Display for CompiledShaderId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:032x}", self.0)
    }
}

pub(crate) fn calculate_variant_id(
    shader_id: ShaderId,
    keywords: &HashMap<impl AsRef<str>, u64>,
) -> CompiledShaderId {
    CompiledShaderId::from_hashes(
        hash_shader_source_id(shader_id),
        hash_shader_keywords(keywords),
    )
}

fn hash_shader_source_id(id: ShaderId) -> u64 {
    let as_bytes = id.0.get().to_le_bytes();
    twox_hash::xxhash3_64::Hasher::oneshot(&as_bytes)
}

fn hash_shader_keywords(keywords: &HashMap<impl AsRef<str>, u64>) -> u64 {
    let mut sorted = Vec::with_capacity(keywords.len());

    for (keyword, value) in keywords {
        let keyword_str = keyword.as_ref();
        sorted.push(format!("{keyword_str}={value}"));
    }

    sorted.sort_unstable();

    let joined = sorted.join("-");

    twox_hash::xxhash3_64::Hasher::oneshot(joined.as_bytes())
}

/// [wutengine_shadercompiler::ShaderHasher] implementation that uses XXHash3 (from [twox_hash])
pub(super) struct WutEngineShaderHasher;

impl wutengine_shadercompiler::ShaderHasher<ShaderId> for WutEngineShaderHasher {
    type VariantId = CompiledShaderId;

    #[inline]
    fn hash_source_id(id: ShaderId) -> u64 {
        hash_shader_source_id(id)
    }

    #[inline]
    fn hash_keywords<S: AsRef<str>>(keywords: &HashMap<S, u64>) -> u64 {
        hash_shader_keywords(keywords)
    }

    #[inline]
    fn variant_id_from_hashes(source_id_hash: u64, keyword_hash: u64) -> Self::VariantId {
        CompiledShaderId::from_hashes(source_id_hash, keyword_hash)
    }
}
