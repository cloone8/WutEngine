//! Shader compilation. The conversion of a [Shader](super::Shader) into a [CompiledShader](super::CompiledShader)

use core::num::NonZero;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

use nohash_hasher::IntSet;
use wutengine_shadercompiler::{
    CAMERA_PARAMS_BIND_GROUP_INDEX, INSTANCE_PARAMS_BIND_GROUP_INDEX,
    MATERIAL_PARAMS_BIND_GROUP_INDEX,
};

use crate::graphics::internal_bind_groups::{
    get_camera_bind_group_layout, get_instance_bind_group_layout,
};
use crate::graphics::shader::{CompiledShaderId, WutEngineShaderHasher};
use crate::graphics::{BindGroup, GFX_DEVICE, cache};
use crate::util::unreachable_dbg;

use super::{Shader, ShaderBufferParameterType, ShaderParameter, ShaderVertexAttributeType};

#[derive(Debug, derive_more::Display, derive_more::Error, derive_more::From)]
pub(crate) enum CompileErr {
    CrossCompile(wutengine_shadercompiler::CompileErr),
}

/// Compiles `shader` with the provided set of active keywords and inserts it into the shader cache. If the shader
/// has already been compiled previously, returns the cached copy.
pub(crate) fn compile(
    shader: &Shader,
    keywords: &HashMap<String, u64>,
) -> Result<Arc<CompiledShader>, Box<CompileErr>> {
    profiling::function_scope!();

    let variant_id = crate::graphics::shader::calculate_variant_id(shader.id, &keywords);

    if let Some(cached) = cache::shader::find(&variant_id) {
        return Ok(cached);
    }

    let variant_id_string = variant_id.to_string();

    profiling::scope!("Compile shader from source", variant_id_string.as_str());

    log::debug!("Compiling shader {variant_id}");

    let vertex_attr_conditions: Vec<Option<&str>> = Vec::from_iter(
        shader
            .vertex_attributes
            .iter()
            .map(|p| p.condition.as_ref().map(|c| c.0.as_str())),
    );

    let user_param_conditions: Vec<Option<&str>> = Vec::from_iter(
        shader
            .user_params
            .iter()
            .map(|p| p.get_condition().map(|c| c.0.as_str())),
    );

    let output = wutengine_shadercompiler::compile::<_, WutEngineShaderHasher>(
        wutengine_shadercompiler::CompInput {
            id: shader.id,
            source: shader.get_source(),
            keywords: &keywords,
            parameters: &user_param_conditions,
            vertex_attributes: &vertex_attr_conditions,
            per_camera_block: include_str!("camera_group.wgsl"),
            per_instance_block: include_str!("instance_group.wgsl"),
        },
    )
    .map_err(|e| Box::new(e.into()))?;

    assert_eq!(
        variant_id, output.variant_id,
        "Incorrect variant ID returned"
    );

    let native_module = {
        profiling::scope!("Compile native shader module", variant_id_string.as_str());

        let module = GFX_DEVICE.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&variant_id_string),
            source: wgpu::ShaderSource::Naga(Cow::Owned(*output.module)),
        });

        log_shader_compilation_info(&module);

        module
    };

    let user_bind_group_layout: wgpu::BindGroupLayout = create_user_params_bind_group_layout(
        format!("{variant_id_string} material bind group layout").as_str(),
        &shader.user_params,
        &output.remaining_params,
    );

    let pipeline_layout = {
        profiling::scope!("Create native pipeline layout", variant_id_string.as_str());

        GFX_DEVICE.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(format!("{variant_id_string} pipeline layout").as_str()),
            bind_group_layouts: &sort_layouts(
                get_camera_bind_group_layout(),
                &user_bind_group_layout,
                get_instance_bind_group_layout(),
            ),
            immediate_size: 0,
        })
    };

    let parameters = output
        .remaining_params
        .into_iter()
        .map(|idx| shader.user_params[idx].clone())
        .collect();

    let vertex_attributes = output
        .remaining_vertex_attributes
        .into_iter()
        .map(|idx| {
            let attr = &shader.vertex_attributes[idx];
            (attr.ty, attr.location)
        })
        .collect();

    let compiled = CompiledShader {
        id: output.variant_id,
        module: native_module,
        pipeline_layout,
        user_bind_group_layout: user_bind_group_layout.clone(),
        parameters,
        vertex_attributes,
    };

    Ok(cache::shader::insert(variant_id, compiled))
}

fn sort_layouts<'a>(
    cam: &'a wgpu::BindGroupLayout,
    mat: &'a wgpu::BindGroupLayout,
    instance: &'a wgpu::BindGroupLayout,
) -> [&'a wgpu::BindGroupLayout; 3] {
    core::array::from_fn(|i| match i as u32 {
        CAMERA_PARAMS_BIND_GROUP_INDEX => cam,
        MATERIAL_PARAMS_BIND_GROUP_INDEX => mat,
        INSTANCE_PARAMS_BIND_GROUP_INDEX => instance,
        _ => unsafe { unreachable_dbg!() },
    })
}

fn create_user_params_bind_group_layout(
    name: &str,
    params: &[ShaderParameter],
    after_compile_filter: &IntSet<usize>,
) -> wgpu::BindGroupLayout {
    profiling::function_scope!();

    let params_with_filter = params
        .iter()
        .enumerate()
        .filter(|(index, _)| after_compile_filter.contains(index))
        .map(|(_, p)| p);

    let buffer_entry = wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: Some(
                NonZero::new(BindGroup::total_buffer_size(params_to_buf_iter(
                    params_with_filter.clone(),
                )) as u64)
                .unwrap(),
            ),
        },
        count: None,
    };

    let mut all_entries = vec![buffer_entry];

    for param in params_with_filter {
        let ShaderParameter::Opaque { ty, .. } = param else {
            continue;
        };

        let binding = all_entries.len();

        let opaque_entry = wgpu::BindGroupLayoutEntry {
            binding: binding as u32,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: ty.to_wgpu_binding_type(),
            count: None,
        };

        all_entries.push(opaque_entry);
    }

    GFX_DEVICE.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(name),
        entries: &all_entries,
    })
}

fn params_to_buf_iter<'a>(
    params: impl IntoIterator<Item = &'a ShaderParameter>,
) -> impl IntoIterator<Item = ShaderBufferParameterType> {
    params.into_iter().filter_map(|p| {
        if let ShaderParameter::Buffer { ty, .. } = p {
            Some(*ty)
        } else {
            None
        }
    })
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

#[derive(Debug)]
pub(crate) struct CompiledShader {
    pub(crate) id: CompiledShaderId,
    pub(crate) module: wgpu::ShaderModule,
    pub(crate) pipeline_layout: wgpu::PipelineLayout,
    pub(crate) user_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) parameters: Vec<ShaderParameter>,
    pub(crate) vertex_attributes: HashMap<ShaderVertexAttributeType, u32>,
}
