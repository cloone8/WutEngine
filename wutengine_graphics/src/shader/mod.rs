use core::ops::RangeInclusive;
use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use wgpu::{ShaderModule, ShaderModuleDescriptor};
use wutengine_asset::Asset;

pub(crate) mod cache;
mod vertexlayout;

pub use vertexlayout::ShaderVertexLayout;
use wutengine_shadercompiler::{CompileStage, ShaderOutput};

use crate::GRAPHICS_MANAGER;
use crate::resource::GpuResource;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderSource {
    pub name: String,
    pub source: String,
    pub available_keywords: HashMap<String, RangeInclusive<i64>>,
    pub vertex_layout: ShaderVertexLayout,
}

impl Asset for ShaderSource {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledShader {
    pub name: String,
    pub keyword_hash: u64,
    pub source: naga::Module,

    #[serde(skip)]
    pub(crate) renderer_data: GpuResource<ShaderModule>,
}

pub fn get(
    shader: &ShaderSource,
    keywords: &HashMap<String, i64>,
    keyword_hash: u64,
) -> Option<Arc<ShaderModule>> {
    if let Some(cached) = GRAPHICS_MANAGER
        .shader_cache
        .find(&shader.name, keyword_hash)
    {
        return Some(cached);
    }

    let wutengine_shader = match compile_wutengine_shader(shader, keywords) {
        Ok(sh) => sh,
        Err(e) => {
            log::error!("Failed to compile WutEngine shader: {}", e);
            return None;
        }
    };

    let native_shader = compile_native_shader(wutengine_shader);

    Some(
        GRAPHICS_MANAGER
            .shader_cache
            .insert(shader.name.clone(), keyword_hash, native_shader),
    )
}

fn compile_wutengine_shader(
    shader_source: &ShaderSource,
    keywords: &HashMap<String, i64>,
) -> Result<CompiledShader, wutengine_shadercompiler::Error> {
    log::debug!("Compiling shader variant");

    let compile_result = wutengine_shadercompiler::compile_single_shader(
        &shader_source.source,
        keywords.iter().map(|(k, v)| (k.as_str(), *v)).collect(),
        CompileStage::Full,
    )?;

    let compiled = if let ShaderOutput::Compiled {
        source,
        keyword_hash,
        keywords: _,
    } = compile_result
    {
        CompiledShader {
            name: shader_source.name.clone(),
            keyword_hash,
            renderer_data: GpuResource::default(),
            source: *source,
        }
    } else {
        unreachable!("Shader is fully compiled after compilation");
    };

    Ok(compiled)
}

fn compile_native_shader(wutengine_shader: CompiledShader) -> wgpu::ShaderModule {
    GRAPHICS_MANAGER
        .device
        .create_shader_module(ShaderModuleDescriptor {
            label: Some(
                format!(
                    "{}::{:032x}",
                    &wutengine_shader.name, wutengine_shader.keyword_hash
                )
                .as_str(),
            ),
            source: wgpu::ShaderSource::Naga(std::borrow::Cow::Owned(wutengine_shader.source)),
        })
}
