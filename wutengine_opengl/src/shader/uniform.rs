//! Uniform functionality for OpenGL shaders

use core::ffi::CStr;
use core::num::NonZero;
use std::collections::HashMap;

use glam::Mat4;
use wutengine_graphics::material::MaterialParameter;
use wutengine_graphics::renderer::RendererTextureId;

use crate::error::checkerr;
use crate::gltypes::GlMat4f;
use crate::opengl::types::{GLchar, GLenum, GLint, GLsizei, GLuint};
use crate::opengl::{self, Gl};
use crate::texture::GlTexture;

/// The description of a single OpenGL shader uniform
#[derive(Debug, Clone, Copy)]
pub(crate) struct GlShaderUniform {
    /// The uniform location, as given by [Gl::GetUniformLocation]
    pub(crate) location: GLint,

    /// The uniform index
    pub(crate) _index: GLuint,

    /// The uniform type
    pub(crate) uniform_type: GLenum,

    /// The uniform size (in elements of type [Self::uniform_type])
    pub(crate) uniform_size: GLint,
}

/// Finds all uniforms in the given program, and returns information on them
pub(super) fn discover_uniforms(
    gl: &Gl,
    program: NonZero<GLuint>,
) -> HashMap<String, GlShaderUniform> {
    log::debug!("Discovering uniforms for shaderprogram {}", program);

    // First, find the total amount of uniforms currently active in this program
    let mut active_uniforms: GLint = 0;

    unsafe {
        gl.GetProgramiv(
            program.get(),
            opengl::ACTIVE_UNIFORMS,
            &raw mut active_uniforms,
        );
        checkerr!(gl);
    }

    if active_uniforms < 0 {
        log::error!(
            "OpenGL returned a negative amount of uniforms ({}) for program {}",
            active_uniforms,
            program
        );
        return HashMap::new();
    }

    log::trace!(
        "Shaderprogram {} has {} active uniforms",
        program,
        active_uniforms
    );

    // Now, find the max name length of any active uniform
    let mut max_uniform_name_len: GLint = 0;

    unsafe {
        gl.GetProgramiv(
            program.get(),
            opengl::ACTIVE_UNIFORM_MAX_LENGTH, // Includes the null-terminator
            &raw mut max_uniform_name_len,
        );
        checkerr!(gl);
    }

    if max_uniform_name_len < 0 {
        log::error!(
            "OpenGL returned a negative max uniform name length ({}) for program {}",
            max_uniform_name_len,
            program
        );
        return HashMap::new();
    }

    log::trace!(
        "Shaderprogram {} has a max uniform name length of {}",
        program,
        max_uniform_name_len
    );

    // Set up a buffer for the name
    let mut name_buf = vec![0u8; max_uniform_name_len as usize];

    // Now actually query each uniform.
    let mut found_uniforms = HashMap::new();

    for index in 0..(active_uniforms as GLuint) {
        let mut actual_name_len: GLsizei = 0; // Name length _excluding_ null-terminator
        let mut uniform_size: GLint = 0;
        let mut uniform_type: GLenum = 0;

        unsafe {
            gl.GetActiveUniform(
                program.get(),
                index,
                name_buf.len() as GLsizei,
                &raw mut actual_name_len,
                &raw mut uniform_size,
                &raw mut uniform_type,
                name_buf.as_mut_ptr() as *mut GLchar,
            );
            checkerr!(gl);
        }

        let name_cstr =
            CStr::from_bytes_with_nul(&name_buf[..(actual_name_len + 1) as usize]).unwrap();

        let name = name_cstr.to_str().unwrap();

        log::trace!(
            "Found uniform at index {} with name \"{}\", type {}, and size {}",
            index,
            name,
            uniform_type,
            uniform_size
        );

        // Find the uniform location, as that needs to be done seperately
        let uniform_location =
            unsafe { gl.GetUniformLocation(program.get(), name_cstr.as_ptr() as *const GLchar) };

        checkerr!(gl);

        if uniform_location < 0 {
            log::error!(
                "Could not get uniform location for uniform {} in program {}. Returned location: {}",
                name,
                program,
                uniform_location
            );
            continue;
        }

        log::trace!(
            "Uniform {} location in program {} is {}",
            name,
            program,
            uniform_location
        );

        let found_uniform = GlShaderUniform {
            location: uniform_location,
            _index: index,
            uniform_type,
            uniform_size,
        };

        let prev = found_uniforms.insert(name.to_owned(), found_uniform);

        debug_assert!(prev.is_none());
    }

    log::debug!("Found uniforms: {:#?}", found_uniforms);

    found_uniforms
}

/// Tries to set the given uniform material parameters on the given shader program.
/// Any parameters that are not found, will not be set
pub(super) fn set_uniforms(
    gl: &Gl,
    program: NonZero<GLuint>,
    to_set: &HashMap<String, MaterialParameter>,
    program_uniforms: &HashMap<String, GlShaderUniform>,
    first_free_texture_unit: &mut GLenum,
    texture_mappings: &mut HashMap<RendererTextureId, GlTexture>,
) {
    for (uniform_name, uniform_value) in to_set {
        let program_uniform = match program_uniforms.get(uniform_name) {
            Some(pu) => pu,
            None => {
                log::warn!(
                    "Could not find uniform {} on program {}",
                    uniform_name,
                    program
                );
                continue;
            }
        };

        let ok = set_uniform_value(
            gl,
            uniform_value,
            program_uniform,
            first_free_texture_unit,
            texture_mappings,
        );

        if !ok {
            log::warn!(
                "Failed to set uniform {} with type {} and size {} to {:#?} on program {}",
                uniform_name,
                program_uniform.uniform_type,
                program_uniform.uniform_size,
                uniform_value,
                program
            );
        }
    }
}

/// Tries to set the given uniform to the given value, if the types are compatible
pub(super) fn set_uniform_value(
    gl: &Gl,
    value: &MaterialParameter,
    uniform: &GlShaderUniform,
    first_free_texture_unit: &mut GLenum,
    texture_mappings: &mut HashMap<RendererTextureId, GlTexture>,
) -> bool {
    if uniform.uniform_size != 1 {
        todo!("Array uniforms not yet implemented");
    }

    let ok = match uniform.uniform_type {
        opengl::BOOL => set_bool(gl, value, uniform.location),
        opengl::FLOAT_VEC4 => set_float_vec4(gl, value, uniform.location),
        opengl::FLOAT_MAT4 => set_float_mat4(gl, value, uniform.location),
        opengl::SAMPLER_2D => set_texture_2d(
            gl,
            value,
            uniform.location,
            first_free_texture_unit,
            texture_mappings,
        ),

        _ => {
            log::error!("Unknown uniform type: {}", uniform.uniform_type);
            false
        }
    };

    checkerr!(gl);

    ok
}

fn set_bool(gl: &Gl, value: &MaterialParameter, location: GLint) -> bool {
    unsafe {
        match value {
            MaterialParameter::Boolean(val) => {
                let as_int = if *val { 1 } else { 0 };

                gl.Uniform1i(location, as_int);
                true
            }
            MaterialParameter::Color(_) => false,
            MaterialParameter::Vec4(_) => false,
            MaterialParameter::Mat4(_) => false,
            MaterialParameter::Texture(_) => false,
        }
    }
}

fn set_float_vec4(gl: &Gl, value: &MaterialParameter, location: GLint) -> bool {
    unsafe {
        match value {
            MaterialParameter::Color(color) => {
                gl.Uniform4f(location, color.r, color.g, color.b, color.a);
                true
            }
            MaterialParameter::Vec4(vec) => {
                gl.Uniform4f(location, vec.x, vec.y, vec.z, vec.w);
                true
            }
            MaterialParameter::Mat4(_) => false,
            MaterialParameter::Texture(_) => false,
            MaterialParameter::Boolean(val) => {
                let as_flt = if *val { 1.0 } else { 0.0 };

                gl.Uniform4f(location, as_flt, as_flt, as_flt, as_flt);
                true
            }
        }
    }
}

fn set_float_mat4(gl: &Gl, value: &MaterialParameter, location: GLint) -> bool {
    unsafe {
        match value {
            MaterialParameter::Color(_) => false,
            MaterialParameter::Vec4(_) => false,
            MaterialParameter::Mat4(mat4) => {
                let mat_gl = GlMat4f::from(*mat4);
                gl.UniformMatrix4fv(location, 1, opengl::FALSE, &raw const mat_gl as *const f32);

                true
            }
            MaterialParameter::Texture(_) => false,
            MaterialParameter::Boolean(val) => {
                let mat = if *val { Mat4::IDENTITY } else { Mat4::ZERO };
                let mat_gl = GlMat4f::from(mat);

                gl.UniformMatrix4fv(location, 1, opengl::FALSE, &raw const mat_gl as *const f32);

                true
            }
        }
    }
}

fn set_texture_2d(
    gl: &Gl,
    value: &MaterialParameter,
    location: GLint,
    texture_unit: &mut GLenum,
    texture_mappings: &mut HashMap<RendererTextureId, GlTexture>,
) -> bool {
    match value {
        MaterialParameter::Boolean(_) => false,
        MaterialParameter::Color(_) => false,
        MaterialParameter::Vec4(_) => false,
        MaterialParameter::Mat4(_) => false,
        MaterialParameter::Texture(tex_id) => {
            let tex_buf = texture_mappings.get_mut(tex_id);

            if tex_buf.is_none() {
                log::warn!(
                    "Tried to use texture {}, but its buffers were not found",
                    tex_id
                );
                return false;
            }

            unsafe {
                set_texture_from_buf(gl, tex_buf.unwrap(), location, texture_unit);
            }

            true
        }
    }
}

pub(super) unsafe fn set_texture_from_buf(
    gl: &Gl,
    buffer: &mut GlTexture,
    location: GLint,
    texture_unit: &mut GLenum,
) {
    match buffer {
        GlTexture::Tex2D(gl_tex2d) => {
            let cur_tex_unit = *texture_unit;
            unsafe {
                gl.ActiveTexture(opengl::TEXTURE0 + cur_tex_unit);
                gl_tex2d.bind(gl);
                gl.Uniform1i(location, cur_tex_unit as GLint);
            }

            *texture_unit += 1;
        }
    }
}
