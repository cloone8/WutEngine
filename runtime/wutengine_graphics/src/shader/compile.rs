//! Shader compilation. The conversion of a [`Shader`](super::Shader) into a [`CompiledShader`](super::CompiledShader)

use alloc::borrow::Cow;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::fmt::Display;
use std::collections::HashMap;

use wutengine_assets::assets::shader::PrecompiledShader;
use wutengine_assets::assets::shader::ShaderHash;
use wutengine_assets::assets::shader::ShaderVertexAttributeType;
use wutengine_shadercompiler::KeywordErr;
use wutengine_shadercompiler::engine;

use crate::BindGroupLayout;
use crate::GFX_DEVICE;
use crate::UnsupportedBindingErr;
use crate::cache;
use crate::internal_bind_groups::get_camera_bind_group_layout;
use crate::internal_bind_groups::get_instance_bind_group_layout;
use crate::label;
use crate::shader::shader_attr_wgpu_vertex_format;

use super::Shader;

/// An error while compiling a [Shader] into a [`CompiledShader`]
#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub(crate) enum CompileErr {
    /// The keyword values don't fit the shader
    Keywords(KeywordErr),

    /// Compiling the source failed
    Compile(wutengine_shadercompiler::CompileErr),

    /// A precompiled shader doesn't fit the engine layout
    Layout(engine::LayoutErr),

    /// The shader has a binding the runtime can't provide
    UnsupportedBinding(UnsupportedBindingErr),
}

/// Registers a precompiled shader variant. When a material needs this variant, it is used instead of compiling the
/// shader source.
pub fn register_precompiled(shader: PrecompiledShader) {
    cache::shader::insert_precompiled(shader);
}

/// Compiles `shader` with the provided set of active keywords and inserts it into the shader cache. If the shader
/// has already been compiled previously, returns the cached copy.
pub(crate) fn compile(
    shader: &Shader,
    keywords: &HashMap<String, u64>,
) -> Result<Arc<CompiledShader>, Box<CompileErr>> {
    let variant = shader
        .info
        .variant(keywords.iter().map(|(k, v)| (k, *v)))
        .map_err(|e| Box::new(e.into()))?;

    if let Some(cached) = cache::shader::find(&variant.hash()) {
        return Ok(cached);
    }

    let precompiled = if let Some(precompiled) = cache::shader::take_precompiled(&variant.hash()) {
        // Precompiled shaders come from asset files, so check them like a fresh compile would
        engine::check_layout(&precompiled.bindings).map_err(|e| Box::new(e.into()))?;
        precompiled
    } else {
        profiling::scope!("Compile shader from source", shader.name());

        log::debug!(
            "No precompiled variant {} of shader `{}`, compiling from source",
            variant.hash(),
            shader.name()
        );

        // TODO: Imports other than the engine's are not resolved yet. Resolve them through the asset server
        // once it can find shaders by name.
        wutengine_shadercompiler::compile(&shader.source, &variant, None)
            .map_err(|e| Box::new(e.into()))?
    };

    let compiled = CompiledShader::try_from(precompiled).map_err(|e| Box::new(e.into()))?;

    Ok(cache::shader::insert(variant.hash(), compiled))
}

/// A compiled [`Shader`], with all keywords resolved
#[derive(Debug)]
pub struct CompiledShader {
    /// The hash identifying this variant
    pub id: ShaderHash,

    /// The human-readable name of the source shader
    pub source_name: String,

    /// The actual shader module
    pub module: wgpu::ShaderModule,

    /// The pipeline layout used by this shader
    pub pipeline_layout: wgpu::PipelineLayout,

    /// The layout of the material bind group
    pub user_bind_group_layout: BindGroupLayout,

    /// The vertex attributes used by the vertex stage of this shader
    /// Ordered so that the binding slots are consistent
    pub vertex_attributes: BTreeMap<ShaderVertexAttributeType, wgpu::VertexAttribute>,
}

impl TryFrom<PrecompiledShader> for CompiledShader {
    type Error = UnsupportedBindingErr;

    /// Creates the GPU objects for a precompiled shader
    fn try_from(precompiled: PrecompiledShader) -> Result<Self, Self::Error> {
        let label = format!("{}:{}", precompiled.name, precompiled.hash);

        let uses_group = |group| precompiled.bindings.iter().any(|b| b.group == group);
        let uses_camera = uses_group(engine::CAMERA_PARAMS_BIND_GROUP_INDEX);
        let uses_instance = uses_group(engine::INSTANCE_PARAMS_BIND_GROUP_INDEX);

        let user_bind_group_layout = BindGroupLayout::new(
            label!("{} material bind group layout", label),
            precompiled
                .bindings
                .iter()
                .filter(|b| b.group == engine::MATERIAL_PARAMS_BIND_GROUP_INDEX)
                .cloned(),
        )?;

        let native_module = {
            profiling::scope!("Compile native shader module", label.as_str());

            let module = GFX_DEVICE.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: label!(&label),
                source: wgpu::ShaderSource::Naga(Cow::Owned(*precompiled.module)),
            });

            log_shader_compilation_info(&module);

            module
        };

        let pipeline_layout = {
            profiling::scope!("Create native pipeline layout", label.as_str());

            let mut bind_group_layouts = [None; engine::NUM_BIND_GROUPS as usize];

            bind_group_layouts[engine::CAMERA_PARAMS_BIND_GROUP_INDEX as usize] =
                uses_camera.then(|| get_camera_bind_group_layout().native());
            bind_group_layouts[engine::MATERIAL_PARAMS_BIND_GROUP_INDEX as usize] =
                Some(user_bind_group_layout.native());
            bind_group_layouts[engine::INSTANCE_PARAMS_BIND_GROUP_INDEX as usize] =
                uses_instance.then(|| get_instance_bind_group_layout().native());

            GFX_DEVICE.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: label!("{} pipeline layout", label),
                bind_group_layouts: &bind_group_layouts,
                immediate_size: 0,
            })
        };

        let vertex_attributes = precompiled
            .vertex_inputs
            .into_iter()
            .map(|(location, input)| {
                (
                    input.attribute,
                    wgpu::VertexAttribute {
                        format: shader_attr_wgpu_vertex_format(input.attribute),
                        offset: 0, // We currently do only one attribute per buffer
                        shader_location: location,
                    },
                )
            })
            .collect();

        Ok(Self {
            id: precompiled.hash,
            source_name: precompiled.name,
            module: native_module,
            pipeline_layout,
            user_bind_group_layout,
            vertex_attributes,
        })
    }
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
                log::error!("Shader compile log{location_string}: {}", message.message);
            }
            wgpu::CompilationMessageType::Warning => {
                log::warn!("Shader compile log{location_string}: {}", message.message);
            }
            wgpu::CompilationMessageType::Info => {
                log::debug!("Shader compile log{location_string}: {}", message.message);
            }
        }
    }
}

impl Display for &CompiledShader {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}:{}", self.source_name, self.id)
    }
}
