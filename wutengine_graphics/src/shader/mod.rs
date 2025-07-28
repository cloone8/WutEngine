use core::num::NonZeroU64;
use core::ops::RangeInclusive;
use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use wgpu::{BindGroupLayoutDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderStages};
use wutengine_asset::Asset;

pub(crate) mod cache;
pub mod constants;
mod vertexlayout;

pub use vertexlayout::ShaderVertexLayout;
use wutengine_shadercompiler::{CompileStage, ShaderOutput};

use crate::GRAPHICS_MANAGER;
use crate::resource::GpuResource;
use crate::shader::constants::{InstanceConstants, ViewportConstants};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderSource {
    pub name: String,
    pub source: String,
    pub available_keywords: HashMap<String, RangeInclusive<i64>>,
    pub vertex_layout: ShaderVertexLayout,
    pub constants: ShaderConstants,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ShaderConstants {
    /// The shader uses the viewport constants.
    pub viewport: bool,

    /// The shader uses the instance constants.
    pub instance: bool,
}

impl ShaderConstants {
    pub fn any(self) -> bool {
        self.viewport || self.instance
    }

    pub fn len(self) -> usize {
        self.viewport as usize + self.instance as usize
    }

    pub(crate) fn viewport_bind_group_layout() -> Arc<wgpu::BindGroupLayout> {
        //TODO: Cache this
        Arc::new(
            GRAPHICS_MANAGER
                .device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("Viewport constants"),
                    entries: &[Self::VIEWPORT_BIND_GROUP_LAYOUT_ENTRY],
                }),
        )
    }

    pub(crate) fn instance_bind_group_layout() -> Arc<wgpu::BindGroupLayout> {
        //TODO: Cache this
        Arc::new(
            GRAPHICS_MANAGER
                .device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("Instance constants"),
                    entries: &[Self::INSTANCE_BIND_GROUP_LAYOUT_ENTRY],
                }),
        )
    }

    const VIEWPORT_BIND_GROUP_LAYOUT_ENTRY: wgpu::BindGroupLayoutEntry =
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: Some(
                    NonZeroU64::new(size_of::<ViewportConstants>() as u64).unwrap(),
                ),
            },
            count: None,
        };

    const INSTANCE_BIND_GROUP_LAYOUT_ENTRY: wgpu::BindGroupLayoutEntry =
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: Some(
                    NonZeroU64::new(size_of::<InstanceConstants>() as u64).unwrap(),
                ),
            },
            count: None,
        };
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
            log::error!("Failed to compile WutEngine shader: {e}");
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
