//! Uniform functionality for OpenGL shaders

use core::num::NonZero;
use std::collections::HashMap;

use wutengine_graphics::shader::uniform::Uniform;

use crate::error::checkerr;
use crate::opengl::types::GLuint;
use crate::opengl::{self, Gl};
use crate::shader::reflection::{
    get_active_uniform_block_iv, get_uniform_block_index, get_uniform_location,
};

use super::GlShaderUniform;

/// Finds all uniforms in the given program, and returns information on them
#[profiling::function]
pub(crate) fn discover_uniforms(
    gl: &Gl,
    program: NonZero<GLuint>,
    source: &HashMap<String, Uniform>,
) -> HashMap<String, GlShaderUniform> {
    log::debug!("Discovering uniforms for shaderprogram {}", program);

    let mut output = HashMap::with_capacity(source.len());

    for (uform_name, uform_desc) in source {
        log::info!("Discovering uniform {}", uform_name);

        if let Some(discovered) = discover_uniform(gl, program, uform_name, uform_desc) {
            output.insert(uform_name.clone(), discovered);
        } else {
            log::error!(
                "Could not find binding for uniform {}. Will not be assignable",
                uform_name
            )
        }
    }

    log::info!("Discovered uniforms: {:#?}", output);

    output
}

fn discover_uniform(
    gl: &Gl,
    program: NonZero<GLuint>,
    name: &String,
    desc: &Uniform,
) -> Option<GlShaderUniform> {
    if desc.ty.is_texture_type() {
        discover_texture(gl, program, name, desc)
    } else {
        discover_block(gl, program, name, desc)
    }
}

fn discover_texture(
    gl: &Gl,
    program: NonZero<GLuint>,
    name: &String,
    desc: &Uniform,
) -> Option<GlShaderUniform> {
    assert!(desc.ty.is_texture_type());

    let binding = match desc.binding.try_as_texture() {
        Some((sampler, _)) => match sampler {
            Some(b) => b,
            None => {
                log::error!(
                    "Texture uniform {} is missing sampler binding. Cannot resolve",
                    name
                );
                return None;
            }
        },
        None => {
            log::error!(
                "Texture type uniform {} has non-texture binding {}. Cannot resolve",
                name,
                desc.binding
            );
            return None;
        }
    };

    assert_eq!(0, binding.group);

    let loc = get_uniform_location(gl, program, &binding.name);

    if loc.is_none() {
        log::error!(
            "Could not find location for texture uniform {} with binding {}",
            name,
            binding
        );

        return None;
    }

    let location = loc.unwrap();

    Some(GlShaderUniform::Sampler {
        location,
        binding: binding.binding,
    })
}

fn discover_block(
    gl: &Gl,
    program: NonZero<GLuint>,
    name: &String,
    desc: &Uniform,
) -> Option<GlShaderUniform> {
    assert!(!desc.ty.is_texture_type());

    let binding = match desc.binding.try_as_standard() {
        Some(b) => b,
        None => {
            log::error!(
                "Standard type uniform {} has texture binding {}. Cannot resolve",
                name,
                desc.binding
            );
            return None;
        }
    };

    assert_eq!(0, binding.group);

    let index = match get_uniform_block_index(gl, program, &binding.name) {
        Some(i) => i,
        None => {
            log::error!(
                "Could not find index for uniform block {} with binding {}",
                name,
                binding
            );
            return None;
        }
    };

    let block_size =
        get_active_uniform_block_iv(gl, program, index, opengl::UNIFORM_BLOCK_DATA_SIZE) as usize;

    unsafe {
        gl.UniformBlockBinding(program.get(), index, binding.binding as GLuint);
    }

    checkerr!(gl);

    Some(GlShaderUniform::Block {
        index,
        binding: binding.binding,
        size_bytes: block_size,
        ty: desc.ty.clone(),
    })
}
