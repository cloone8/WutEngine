//! GPU Shaders

use core::fmt::Display;
use core::hash::Hash;
use core::num::NonZero;
use core::ops::RangeInclusive;
use std::collections::HashMap;

use descriptor::ShaderParameter;
use wutengine_shadercompiler::ShaderHasher;
use wutengine_util_macro::unique_id_type64;

use crate::graphics::cache;

mod compile;
mod descriptor;
mod types;

pub use descriptor::*;

pub use types::*;

pub(crate) use compile::*;

unique_id_type64! {
    /// Unique identifier for a [Shader]
    pub(crate) ShaderId
}

/// A generic shader.
///
/// TODO: Find solution for if the shader is changed after some variants have already
/// been compiled and cached
///
/// TODO: Allow duplicate bindings if only one remains after variant compilation?
/// Seems to be more hassle that its worth
#[derive(Debug)]
pub(crate) struct Shader {
    /// The identifier of the shader
    pub(crate) id: ShaderId,

    /// The human readable name of the shader
    pub(crate) name: String,

    /// The raw WGSL source code
    pub(crate) source: String,

    /// The allowed set of keywords, and their valid ranges of values
    pub(crate) allowed_keywords: HashMap<String, RangeInclusive<u64>>,

    /// The available parameters of this shader
    pub(crate) parameters: Vec<ShaderParameter>,
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

// /// A parameter on a [Shader]
// #[derive(Debug, Clone)]
// pub struct ShaderParameter {
//     /// Bind group and binding within the group
//     pub binding: ParameterBinding,

//     /// The type of the parameter
//     pub ty: ShaderParameterType,

//     /// The amount of array elements, if any.
//     /// If [None], this is not an array
//     pub array_count: Option<NonZero<u32>>,
// }

/// The binding location for a [ShaderParameter]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ParameterBinding {
    /// The binding group index
    pub group: u32,

    /// The binding within the group
    pub binding: u32,
}

impl ParameterBinding {
    /// Creates new [ParameterBinding] from a group and binding index
    #[inline]
    pub const fn new(group: u32, binding: u32) -> Self {
        Self { group, binding }
    }
}

impl Display for ParameterBinding {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "ParameterBinding(group={}, binding={})",
            self.group, self.binding
        )
    }
}

impl Hash for ParameterBinding {
    #[inline(always)]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        let as_u64 = ((self.group as u64) << 32) | (self.binding as u64);
        state.write_u64(as_u64);
    }
}

impl nohash_hasher::IsEnabled for ParameterBinding {}

impl From<ParameterBinding> for wutengine_shadercompiler::Binding {
    #[inline(always)]
    fn from(value: ParameterBinding) -> Self {
        Self {
            group: value.group,
            binding: value.binding,
        }
    }
}

impl From<(u32, u32)> for ParameterBinding {
    #[inline]
    fn from(value: (u32, u32)) -> Self {
        Self::new(value.0, value.1)
    }
}

// impl ShaderParameter {
//     #[inline]
//     fn to_wgpu_layout_entry(&self) -> wgpu::BindGroupLayoutEntry {
//         wgpu::BindGroupLayoutEntry {
//             binding: self.binding.binding,
//             visibility: wgpu::ShaderStages::all(),
//             ty: self.ty.to_wgpu_binding_type(),
//             count: self.array_count,
//         }
//     }
// }

// /// The type of a [ShaderParameter]
// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, derive_more::Display)]
// pub enum ShaderParameterType {
//     /// A texture sampler
//     Sampler,

//     /// A 2D texture
//     Texture2D,
// }

// impl ShaderParameterType {
//     #[inline]
//     fn to_wgpu_binding_type(self) -> wgpu::BindingType {
//         match self {
//             Self::Sampler => wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
//             Self::Texture2D => wgpu::BindingType::Texture {
//                 sample_type: wgpu::TextureSampleType::Float { filterable: true },
//                 view_dimension: wgpu::TextureViewDimension::D2,
//                 multisampled: false,
//             },
//         }
//     }
// }

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

    /// The GPU-side shader module handle
    pub(super) module: wgpu::ShaderModule,

    /// The native GPU bind group layouts
    pub(super) user_param_group_layout: wgpu::BindGroupLayout,

    /// The native GPU pipeline layout
    pub(super) pipeline_layout: wgpu::PipelineLayout,
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
