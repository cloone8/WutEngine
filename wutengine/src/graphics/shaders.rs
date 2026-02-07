//! GPU Shaders

use core::ops::RangeInclusive;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

use wutengine_shadercompiler::{Input, ShaderHasher};

use crate::graphics::cache;
use crate::map;

use super::GFX_DEVICE;

/// A generic shader.
pub struct Shader {
    /// The identifier of the shader
    pub id: String,

    /// The raw WGSL source code
    pub source: String,

    /// The allowed set of keywords, and their valid ranges of values
    pub allowed_keywords: HashMap<String, RangeInclusive<u64>>,
}

impl Shader {
    pub(crate) fn compile(&self, keywords: &HashMap<String, u64>) -> Arc<CompiledShader> {
        profiling::function_scope!();

        //TODO: Check if keywords are in allowed keywords?

        let cache_key = self.create_shader_cache_key(keywords);

        if let Some(cached) = cache::shader::find(&cache_key) {
            return cached;
        }

        // Shader not yet in cache. Compile it

        let compile_output = wutengine_shadercompiler::compile::<XXHashShaderHasher>(Input {
            source_id: &self.id,
            source: &self.source,
            active_keywords: keywords,
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
            source: wgpu::ShaderSource::Naga(Cow::Owned(*compile_output.module)),
        });

        if cfg!(debug_assertions) {
            Self::log_shader_compilation_info(&module);
        }

        let compiled = CompiledShader {
            source_hash: compile_output.source_id_hash,
            keyword_hash: compile_output.keyword_hash,
            module,
        };

        cache::shader::insert(cache_key, compiled)
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

    pub(crate) fn create_shader_cache_key(
        &self,
        keywords: &HashMap<String, u64>,
    ) -> cache::shader::ShaderCompilationCacheKey {
        let shader_id_hash = XXHashShaderHasher::hash_source_id(&self.id);
        let keyword_hash = XXHashShaderHasher::hash_keywords(keywords);

        cache::shader::ShaderCompilationCacheKey {
            shader_id_hash,
            keyword_hash,
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
    /// Hashed identifier of the source shader
    pub(super) source_hash: u64,

    /// Hashed identifier of the keywords used while compiling
    pub(super) keyword_hash: u64,

    pub(super) module: wgpu::ShaderModule,
}

impl CompiledShader {
    /// Returns the unique ID of this compiled shader
    #[inline(always)]
    pub const fn get_id(&self) -> u128 {
        ((self.source_hash as u128) << 64) | (self.keyword_hash as u128)
    }

    #[inline(always)]
    pub(crate) const fn get_module(&self) -> &wgpu::ShaderModule {
        &self.module
    }
}

/// [wutengine_shadercompiler::ShaderHasher] implementation that uses XXHash3 (from [twox_hash])
pub(super) struct XXHashShaderHasher;

impl wutengine_shadercompiler::ShaderHasher for XXHashShaderHasher {
    #[inline]
    fn hash_source_id(id: &str) -> u64 {
        twox_hash::xxhash3_64::Hasher::oneshot(id.as_bytes())
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
