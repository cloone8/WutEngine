//! GPU Shaders

use core::{fmt::Display, hash::Hash};
use std::collections::HashMap;

use wutengine_assets::{
    FromSerializedAsset,
    assets::shader::{
        SerializedShader, ShaderDefaultParameters, ShaderKeyword, ShaderParameter,
        ShaderVertexAttribute,
    },
};
use wutengine_util_macro::unique_id_type64;

mod compile;
mod types;

pub(crate) use compile::*;
pub use types::*;

unique_id_type64! {
    /// Unique identifier for a [Shader]
    pub(crate) ShaderId
}

/// A general shader asset, used when configuring Materials
#[derive(Debug, Clone)]
pub struct Shader {
    /// The ID for this shader
    pub(crate) id: ShaderId,

    /// The human-readable name of this shader
    pub(crate) name: String,

    /// The vertex attributes used by this shader
    pub(crate) vertex_attributes: Vec<ShaderVertexAttribute>,

    /// The default parameters used by this shader
    pub(crate) default_parameters: ShaderDefaultParameters,

    /// The keywords allowed to be set on this shader
    #[expect(unused, reason = "Will be used later")]
    pub(crate) keywords: HashMap<String, ShaderKeyword>,

    /// The configurable user-defined parameters on this shader
    pub(crate) parameters: Vec<ShaderParameter>,

    /// The source code for this shader
    pub(crate) source: String,
}

impl FromSerializedAsset for Shader {
    type Error = std::io::Error;

    type Serialized = SerializedShader;

    fn from_serialized_asset(serialized: Self::Serialized) -> Result<Self, Self::Error> {
        Ok(Self {
            id: ShaderId::new(),
            name: serialized.name,
            vertex_attributes: serialized.vertex_attributes,
            default_parameters: serialized.default_parameters,
            keywords: serialized.keywords,
            parameters: serialized.parameters,
            source: match serialized.source {
                wutengine_assets::assets::shader::ShaderSource::Inline { content } => content,
                wutengine_assets::assets::shader::ShaderSource::File { path } => {
                    std::fs::read_to_string(path)?
                }
            },
        })
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

    /// Returns the bits corresponding to the hash of the source shader
    #[inline]
    pub const fn source_hash(self) -> u64 {
        (self.0 >> 64) as u64
    }

    /// Returns the bits corresponding to the hash of the used keywords
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

/// Given a [ShaderId] and a set of keywords, calculates
/// the ID that the resulting compiled shader would have
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
