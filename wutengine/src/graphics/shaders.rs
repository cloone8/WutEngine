//! GPU Shaders

use core::num::NonZero;
use core::ops::RangeInclusive;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

use nohash_hasher::IntSet;
use wutengine_shadercompiler::{Input, ShaderHasher};
use wutengine_util_macro::unique_id_type64;

use crate::graphics::cache;

use super::GFX_DEVICE;

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
    /// Compiles this shader with the given activated keywords
    pub(crate) fn compile(&self, keywords: &HashMap<String, u64>) -> Arc<CompiledShader> {
        profiling::function_scope!();

        if cfg!(debug_assertions) {
            for (keyword, value) in keywords {
                let allowed_values = self
                    .allowed_keywords
                    .get(keyword)
                    .expect("Unexpected keyword given");

                assert!(
                    allowed_values.contains(value),
                    "Invalid keyword value given"
                );
            }
        }

        let cache_key = self.create_shader_cache_key(keywords);

        if let Some(cached) = cache::shader::find(&cache_key) {
            return cached;
        }

        // Shader not yet in cache. Compile it

        let compile_output =
            wutengine_shadercompiler::compile::<ShaderId, XXHashShaderHasher>(Input {
                source_id: self.id,
                source: &self.source,
                active_keywords: keywords,
                all_bindings: &to_shadercompiler_bindings(&self.parameters),
            })
            .expect("Failed to compile shader");

        assert_eq!(
            cache_key.shader_id_hash, compile_output.source_id_hash,
            "Incorrect ID hash generated"
        );
        assert_eq!(
            cache_key.keyword_hash, compile_output.keyword_hash,
            "Incorrect keyword hash generated"
        );

        let module = GFX_DEVICE.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Compiled shader"),
            source: wgpu::ShaderSource::Naga(Cow::Owned(compile_output.module)),
        });

        if cfg!(debug_assertions) {
            Self::log_shader_compilation_info(&module);
        }

        let variant_name = format!("{}#{:016x}", self.name, compile_output.keyword_hash);
        let bind_group_layouts =
            self.create_bind_group_layout(&variant_name, &compile_output.remaining_bindings);

        let compiled = CompiledShader {
            source_hash: compile_output.source_id_hash,
            keyword_hash: compile_output.keyword_hash,
            pipeline_layout: Self::create_pipeline_layout(&variant_name, &bind_group_layouts),
            bind_group_layouts,
            module,
            name: variant_name,
        };

        cache::shader::insert(cache_key, compiled)
    }

    fn create_pipeline_layout(
        variant_name: &str,
        bind_group_layouts: &[wgpu::BindGroupLayout],
    ) -> wgpu::PipelineLayout {
        profiling::function_scope!();

        let as_borrowed: Vec<_> = bind_group_layouts.iter().collect();

        GFX_DEVICE.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(format!("{variant_name} Pipeline Layout").as_str()),
            bind_group_layouts: &as_borrowed,
            immediate_size: 0,
        })
    }

    fn create_bind_group_layout(
        &self,
        variant_name: &str,
        remaining_bindings: &[wutengine_shadercompiler::Binding],
    ) -> Vec<wgpu::BindGroupLayout> {
        profiling::function_scope!();

        let remaining_bindings: HashMap<&str, _> = self
            .parameters
            .iter()
            .filter(|(_, param)| remaining_bindings.contains(&(param.binding.into())))
            .map(|(param_name, param)| (param_name.as_str(), param))
            .collect();

        let Some(highest_group) = remaining_bindings
            .values()
            .map(|param| param.binding.0)
            .max()
        else {
            // No bind groups, so we can just create an empty layout
            return Vec::new();
        };

        let mut group_layout_entries: Vec<Vec<wgpu::BindGroupLayoutEntry>> =
            Vec::with_capacity(highest_group as usize);
        let mut bind_group_layout_labels = Vec::with_capacity(highest_group as usize);

        for bind_group_idx in 0..highest_group {
            let layout_entries: Vec<_> = remaining_bindings
                .values()
                .filter(|param| param.binding.0 == bind_group_idx)
                .map(|param| param.to_wgpu_layout_entry())
                .collect();

            group_layout_entries.push(layout_entries);
            bind_group_layout_labels
                .push(format!("{variant_name} Bind Group {bind_group_idx} Layout"));
        }

        let mut group_layout_descriptors: Vec<wgpu::BindGroupLayoutDescriptor> =
            Vec::with_capacity(highest_group as usize);

        for bind_group_idx in 0..highest_group {
            group_layout_descriptors.push(wgpu::BindGroupLayoutDescriptor {
                label: Some(bind_group_layout_labels[bind_group_idx as usize].as_str()),
                entries: &group_layout_entries[bind_group_idx as usize],
            });
        }

        group_layout_descriptors
            .into_iter()
            .map(|group_layout_descriptor| {
                GFX_DEVICE.create_bind_group_layout(&group_layout_descriptor)
            })
            .collect()
    }

    fn log_shader_compilation_info(module: &wgpu::ShaderModule) {
        profiling::function_scope!();

        let compinfo = pollster::block_on(module.get_compilation_info());

        for message in compinfo.messages {
            let location_string = if let Some(message_loc) = message.location {
                format!(
                    " @ {}:{}",
                    message_loc.line_number, message_loc.line_position
                )
            } else {
                String::new()
            };

            match message.message_type {
                wgpu::CompilationMessageType::Error => {
                    log::error!("Shader compile log{location_string}: {}", message.message)
                }
                wgpu::CompilationMessageType::Warning => {
                    log::warn!("Shader compile log{location_string}: {}", message.message)
                }
                wgpu::CompilationMessageType::Info => {
                    log::debug!("Shader compile log{location_string}: {}", message.message)
                }
            }
        }
    }

    /// Returns the cache key for the combination of this shader and the provided set of keywords
    pub(crate) fn create_shader_cache_key(
        &self,
        keywords: &HashMap<String, u64>,
    ) -> cache::shader::ShaderCompilationCacheKey {
        let shader_id_hash = XXHashShaderHasher::hash_source_id(self.id);
        let keyword_hash = XXHashShaderHasher::hash_keywords(keywords);

        cache::shader::ShaderCompilationCacheKey {
            shader_id_hash,
            keyword_hash,
        }
    }
}

fn to_shadercompiler_bindings(
    params: &HashMap<String, ShaderParameter>,
) -> Vec<wutengine_shadercompiler::Binding> {
    let mut unique_bindings = IntSet::default();

    for param in params.values() {
        let as_binding = wutengine_shadercompiler::Binding {
            group: param.binding.0,
            binding: param.binding.1,
        };

        let not_yet_present = unique_bindings.insert(as_binding);

        assert!(not_yet_present, "Duplicate binding in shader");
    }

    unique_bindings.into_iter().collect()
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

impl CompiledShader {
    /// Returns the unique ID of this compiled shader
    #[inline(always)]
    pub const fn get_id(&self) -> u128 {
        ((self.source_hash as u128) << 64) | (self.keyword_hash as u128)
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
