//! Shader compilation. The conversion of a [Shader](super::Shader) into a [CompiledShader](super::CompiledShader)

use core::num::NonZero;
use std::collections::HashMap;
use std::sync::Arc;

use nohash_hasher::IntSet;

use crate::graphics::shader::{CompiledShaderId, WutEngineShaderHasher};
use crate::graphics::{BindGroup, GFX_DEVICE, cache};

use super::{Shader, ShaderBufferParameterType, ShaderParameter, ShaderVertexAttributeType};

#[derive(Debug, derive_more::Display, derive_more::Error)]
pub(crate) enum CompileErr {}

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

    profiling::scope!(
        "Compile shader from source",
        variant_id.to_string().as_str()
    );

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
            per_camera_block: include_str!("camera.wgsl"),
            per_instance_block: include_str!("instance.wgsl"),
        },
    )
    .unwrap();

    assert_eq!(
        variant_id, output.variant_id,
        "Incorrect variant ID returned"
    );

    let user_bind_group_layout: wgpu::BindGroupLayout = create_user_params_bind_group_layout(
        "Material BGL",
        &shader.user_params,
        &output.remaining_params,
    );

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
        module: output.module,
        user_bind_group_layout: user_bind_group_layout.clone(),
        parameters,
        vertex_attributes,
    };

    Ok(cache::shader::insert(variant_id, compiled))
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
    pub(crate) module: Box<naga::Module>,
    pub(crate) user_bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) parameters: Vec<ShaderParameter>,
    pub(crate) vertex_attributes: HashMap<ShaderVertexAttributeType, u32>,
}
