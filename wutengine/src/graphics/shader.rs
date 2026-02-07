//! GPU Shaders

use core::fmt::Display;
use core::num::NonZero;
use core::ops::RangeInclusive;
use std::collections::HashMap;

use wutengine_shadercompiler::ShaderHasher;
use wutengine_util_macro::unique_id_type64;

use crate::graphics::cache;

mod compile;

pub(crate) use compile::compile;

unique_id_type64! {
    /// Unique identifier for a [Shader]
    pub ShaderId
}

/// A generic shader.
/// TODO: Find solution for if the shader is changed after some variants have already
/// been compiled and cached
#[derive(Debug)]
pub struct Shader {
    /// The identifier of the shader
    pub id: ShaderId,

    /// The human readable name of the shader
    pub name: String,

    /// The raw WGSL source code
    pub source: String,

    /// The allowed set of keywords, and their valid ranges of values
    pub allowed_keywords: HashMap<String, RangeInclusive<u64>>,

    /// The available parameters of this shader
    pub parameters: HashMap<String, ShaderParameter>,
}

impl Shader {
    /// Returns the cache key for the combination of this shader and the provided set of keywords
    pub(crate) fn create_compiled_shader_id(
        &self,
        keywords: &HashMap<String, u64>,
    ) -> CompiledShaderId {
        let shader_id_hash = XXHashShaderHasher::hash_source_id(self.id);
        let keyword_hash = XXHashShaderHasher::hash_keywords(keywords);

        CompiledShaderId::from_hashes(shader_id_hash, keyword_hash)
    }
}

/// A parameter on a [Shader]
#[derive(Debug, Clone)]
pub struct ShaderParameter {
    /// Bind group and binding within the group
    pub binding: (u32, u32),

    /// The type of the parameter
    pub ty: ShaderParameterType,

    /// The amount of array elements, if any.
    /// If [None], this is not an array
    pub array_count: Option<NonZero<u32>>,
}

impl ShaderParameter {
    #[inline]
    fn to_wgpu_layout_entry(&self) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: self.binding.1,
            visibility: wgpu::ShaderStages::all(),
            ty: self.ty.to_wgpu_binding_type(),
            count: self.array_count,
        }
    }
}

/// The type of a [ShaderParameter]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderParameterType {
    /// A texture sampler
    Sampler,

    /// A 2D texture
    Texture2D,
}

impl ShaderParameterType {
    #[inline]
    fn to_wgpu_binding_type(self) -> wgpu::BindingType {
        match self {
            Self::Sampler => wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            Self::Texture2D => wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
        }
    }
}

/// A [Shader] that's been through the WutEngine compilation process, resulting
/// in concrete source code. Can be used in graphics pipelines.
///
/// Note that "compiled" here means that the shader has been compiled by WutEngine only.
/// It still needs to go through compilation in the actual rendering backend.
#[derive(Debug, Clone)]
pub struct CompiledShader {
    /// Human readable name of this shader variant
    pub(super) name: String,

    /// The ID of this compiled shader.
    pub(super) id: CompiledShaderId,

    /// Hashed identifier of the source shader
    pub(super) source_hash: u64,

    /// Hashed identifier of the keywords used while compiling
    pub(super) keyword_hash: u64,

    /// The GPU-side shader module handle
    pub(super) module: wgpu::ShaderModule,

    /// The native GPU bind group layouts
    pub(super) bind_group_layouts: Vec<wgpu::BindGroupLayout>,

    /// The native GPU pipeline layout
    pub(super) pipeline_layout: wgpu::PipelineLayout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct CompiledShaderId(pub(crate) u128);

impl CompiledShaderId {
    #[inline]
    pub const fn from_hashes(source_shader_hash: u64, keyword_hash: u64) -> Self {
        Self(((source_shader_hash as u128) << 64) | (keyword_hash as u128))
    }
}

impl Display for CompiledShaderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:032x}", self.0)
    }
}

impl CompiledShader {
    /// Returns the unique ID of this compiled shader
    #[inline(always)]
    pub const fn get_id(&self) -> CompiledShaderId {
        self.id
    }

    /// Returns the raw [wgpu] module for this shader
    #[inline(always)]
    pub(crate) const fn get_module(&self) -> &wgpu::ShaderModule {
        &self.module
    }
}

/// [wutengine_shadercompiler::ShaderHasher] implementation that uses XXHash3 (from [twox_hash])
pub(super) struct XXHashShaderHasher;

impl wutengine_shadercompiler::ShaderHasher<ShaderId> for XXHashShaderHasher {
    #[inline]
    fn hash_source_id(id: ShaderId) -> u64 {
        let as_bytes = id.0.get().to_le_bytes();
        twox_hash::xxhash3_64::Hasher::oneshot(&as_bytes)
    }

    #[inline]
    fn hash_keywords<S: AsRef<str>>(keywords: &HashMap<S, u64>) -> u64 {
        let mut sorted = Vec::with_capacity(keywords.len());

        for (keyword, value) in keywords {
            let keyword_str = keyword.as_ref();
            sorted.push(format!("{keyword_str}={value}"));
        }

        sorted.sort_unstable();

        let joined = sorted.join("-");

        twox_hash::xxhash3_64::Hasher::oneshot(joined.as_bytes())
    }
}
