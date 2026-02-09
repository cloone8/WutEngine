//! Shader compilation. The conversion of a [Shader](super::Shader) into a [CompiledShader](super::CompiledShader)

use core::ops::RangeInclusive;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

use nohash_hasher::{IntMap, IntSet};
use wutengine_shadercompiler::Input;

use crate::graphics::GFX_DEVICE;
use crate::graphics::shader::{CompiledShaderId, ShaderId, XXHashShaderHasher};

use super::{CompiledShader, Shader, ShaderParameter, ShaderParameterBinding};

#[derive(Debug, derive_more::Display, derive_more::Error)]
pub(crate) enum CompileErr {
    #[display(
        "Shader keyword \"{}\" has a name with invalid characters. Only a-zA-Z and \"_\" are allowed.",
        _0
    )]
    InvalidKeywordName(#[error(not(source))] String),

    #[display(
        "Shader parameter \"{}\" has a name with invalid characters. Only a-zA-Z and \"_\" are allowed.",
        _0
    )]
    InvalidParameterName(#[error(not(source))] String),

    #[display("Unknown shader keyword was given: {}", _0)]
    UnknownKeyword(#[error(not(source))] String),

    #[display(
        "Shader keyword \"{}\" with allowed value range {}-{} was given an invalid value: {}", _0, _2.start(), _2.end(), _1
    )]
    InvalidKeywordValue(String, u64, RangeInclusive<u64>),

    #[display(
        "A duplicate parameter binding was encountered: {}. Parameters with this binding: {}, {}",
        _0,
        _1,
        _2
    )]
    DuplicateParameterBinding(ShaderParameterBinding, String, String),

    #[display("An error was encountered during shader compilation: {}", _0)]
    ShaderCompilationError(Box<wutengine_shadercompiler::Error>),
}

impl From<Box<wutengine_shadercompiler::Error>> for Box<CompileErr> {
    #[inline]
    fn from(value: Box<wutengine_shadercompiler::Error>) -> Self {
        Box::new(CompileErr::ShaderCompilationError(value))
    }
}

/// Compiles `shader` with the provided set of active keywords and inserts it into the shader cache. If the shader
/// has already been compiled previously, returns the cached copy.
pub(crate) fn compile(
    shader: &Shader,
    keywords: &HashMap<String, u64>,
) -> Result<Arc<CompiledShader>, Box<CompileErr>> {
    profiling::function_scope!();

    let cache_key = shader.create_compiled_shader_id(keywords);

    if let Some(cached) = super::cache::shader::find(&cache_key) {
        return Ok(cached);
    }

    log::debug!("Compiling shader: variant {cache_key} of {}", shader.name);
    log::trace!("Variant {cache_key} keywords:\n{}", dump_keywords(keywords));

    check_shader_valid(shader, keywords)?;

    // Shader not yet in cache. Compile it

    let compile_output =
        wutengine_shadercompiler::compile::<ShaderId, XXHashShaderHasher>(Input {
            source_id: shader.id,
            source: &shader.source,
            active_keywords: keywords,
            all_bindings: &to_shadercompiler_bindings(&shader.parameters),
        })?;

    let module = GFX_DEVICE.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Compiled shader"),
        source: wgpu::ShaderSource::Naga(Cow::Owned(compile_output.module)),
    });

    if cfg!(debug_assertions) {
        log_shader_compilation_info(&module);
    }

    let variant_name = format!("{}#{:016x}", shader.name, compile_output.keyword_hash);
    let bind_group_layouts =
        create_bind_group_layout(shader, &variant_name, &compile_output.remaining_bindings);

    let compiled = CompiledShader {
        source_hash: compile_output.source_id_hash,
        keyword_hash: compile_output.keyword_hash,
        id: CompiledShaderId(
            ((compile_output.source_id_hash as u128) << 64) | (compile_output.keyword_hash as u128),
        ),
        pipeline_layout: create_pipeline_layout(&variant_name, &bind_group_layouts),
        bind_group_layouts,
        module,
        name: variant_name,
    };

    Ok(super::cache::shader::insert(cache_key, compiled))
}

fn check_shader_valid(shader: &Shader, keywords: &HashMap<String, u64>) -> Result<(), CompileErr> {
    profiling::function_scope!();

    for (keyword, value) in keywords {
        let keyword_str = keyword.as_str();
        if !keyword_name_valid(keyword_str) {
            return Err(CompileErr::InvalidKeywordName(keyword_str.to_owned()));
        }

        let allowed_values = shader
            .allowed_keywords
            .get(keyword)
            .ok_or(CompileErr::UnknownKeyword(keyword_str.to_owned()))?;

        if !allowed_values.contains(value) {
            return Err(CompileErr::InvalidKeywordValue(
                keyword_str.to_owned(),
                *value,
                allowed_values.clone(),
            ));
        }
    }

    let mut all_bindings = IntMap::default();

    for (name, param) in &shader.parameters {
        let name_str = name.as_str();

        if !param_name_valid(name_str) {
            return Err(CompileErr::InvalidParameterName(name_str.to_owned()));
        }

        let existing = all_bindings.insert(param.binding, name_str);

        if let Some(existing) = existing {
            return Err(CompileErr::DuplicateParameterBinding(
                param.binding,
                existing.to_owned(),
                name_str.to_owned(),
            ));
        }
    }

    Ok(())
}

fn param_name_valid(name: &str) -> bool {
    for c in name.chars() {
        if !(c.is_ascii_alphabetic() || c == '_') {
            return false;
        }
    }

    true
}

fn keyword_name_valid(name: &str) -> bool {
    for c in name.chars() {
        if !(c.is_ascii_alphabetic() || c == '_') {
            return false;
        }
    }

    true
}

fn dump_keywords(keywords: &HashMap<String, u64>) -> String {
    let mut s = String::new();

    for (keyword, value) in keywords {
        s = format!("{s}{keyword} => {value}\n")
    }

    s
}

fn to_shadercompiler_bindings(
    params: &HashMap<String, ShaderParameter>,
) -> Vec<wutengine_shadercompiler::Binding> {
    let mut unique_bindings = IntSet::default();

    for param in params.values() {
        let not_yet_present = unique_bindings.insert(param.binding.into());

        assert!(not_yet_present, "Duplicate binding in shader");
    }

    unique_bindings.into_iter().collect()
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
    shader: &Shader,
    variant_name: &str,
    remaining_bindings: &[wutengine_shadercompiler::Binding],
) -> Vec<wgpu::BindGroupLayout> {
    profiling::function_scope!();

    let remaining_bindings: HashMap<&str, _> = shader
        .parameters
        .iter()
        .filter(|(_, param)| remaining_bindings.contains(&(param.binding.into())))
        .map(|(param_name, param)| (param_name.as_str(), param))
        .collect();

    let Some(highest_group) = remaining_bindings
        .values()
        .map(|param| param.binding.group)
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
            .filter(|param| param.binding.group == bind_group_idx)
            .map(|param| param.to_wgpu_layout_entry())
            .collect();

        group_layout_entries.push(layout_entries);
        bind_group_layout_labels.push(format!("{variant_name} Bind Group {bind_group_idx} Layout"));
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
