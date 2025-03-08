use core::ffi::CStr;
use core::num::NonZero;
use core::ptr::{null, null_mut};
use std::collections::HashMap;
use std::ffi::CString;

use thiserror::Error;
use wutengine_graphics::shader::{Shader, ShaderVertexLayout, Uniform};

use crate::error::checkerr;
use crate::opengl::types::{GLchar, GLenum, GLint, GLsizei, GLuint};
use crate::opengl::{self, Gl};

#[derive(Debug)]
pub(crate) struct GlShaderProgram {
    handle: Option<NonZero<GLuint>>,
    vertex_layout: ShaderVertexLayout,
    uniforms: HashMap<String, GlShaderUniform>,
}

impl Drop for GlShaderProgram {
    fn drop(&mut self) {
        if cfg!(debug_assertions) && self.handle.is_some() {
            log::warn!("GL shader dropped without being destroyed!");
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct GlShaderUniform {
    pub(crate) location: GLint,
    pub(crate) index: GLuint,
    pub(crate) uniform_type: GLenum,
    pub(crate) uniform_size: GLint,
}

#[derive(Debug, Error)]
pub(crate) enum CreateErr {
    #[error("Shader has no source code")]
    Empty,

    #[error("OpenGL did not return a shader program handle")]
    Handle,

    #[error("Could not compile shader stage")]
    Compile(#[from] CompileErr),

    #[error("Could not link shader program")]
    Link(String),
}

#[derive(Debug, Error)]
pub(crate) enum CompileErr {
    #[error("OpenGL did not return a shader handle")]
    Handle,

    #[error("Source code contained non-ascii or NUL character")]
    SourceEncoding,

    #[error("OpenGL shader compile error: {}", 0)]
    Compile(String),
}

impl GlShaderProgram {
    /// Creates, compiles, and links a new OpenGL shaderprogram from the given source
    pub(crate) fn new(gl: &Gl, source: &Shader) -> Result<Self, CreateErr> {
        if !source.source.has_any() {
            return Err(CreateErr::Empty);
        }

        let program: NonZero<GLuint> =
            NonZero::new(unsafe { gl.CreateProgram() }).ok_or(CreateErr::Handle)?;
        checkerr!(gl);

        // Compile the stages one-by-one, cleaning up the partial program if we encounter an error
        let vertex = if let Some(vtx_src) = &source.source.vertex {
            let compiled = compile_stage(gl, vtx_src, opengl::VERTEX_SHADER);

            if let Err(e) = compiled {
                destroy_program(gl, program);

                return Err(e.into());
            } else {
                Some(compiled.unwrap())
            }
        } else {
            None
        };

        let fragment = if let Some(frg_src) = &source.source.fragment {
            let compiled = compile_stage(gl, frg_src, opengl::FRAGMENT_SHADER);

            if let Err(e) = compiled {
                if let Some(vtx) = vertex {
                    destroy_stage(gl, vtx)
                }

                destroy_program(gl, program);

                return Err(e.into());
            } else {
                Some(compiled.unwrap())
            }
        } else {
            None
        };

        unsafe {
            // Attach the individual stages, if they exist
            if let Some(sh) = vertex {
                gl.AttachShader(program.get(), sh.get());
                checkerr!(gl);
            }

            if let Some(sh) = fragment {
                gl.AttachShader(program.get(), sh.get());
                checkerr!(gl);
            }

            // Do the actual linking
            gl.LinkProgram(program.get());
            checkerr!(gl);
        }

        // Now check if the compilation was succesful
        let mut success = opengl::FALSE as GLint;

        unsafe {
            gl.GetProgramiv(program.get(), opengl::LINK_STATUS, &raw mut success);
            checkerr!(gl);
        }

        if success != (opengl::TRUE as GLint) {
            let err = get_program_link_err(gl, program);

            destroy_program(gl, program);
            if let Some(sh) = vertex {
                destroy_stage(gl, sh)
            }
            if let Some(sh) = fragment {
                destroy_stage(gl, sh)
            }

            return Err(CreateErr::Link(err));
        }

        // Program succesfully compiled. Now find all uniforms as declared by the source shader
        let gl_uniforms = discover_uniforms(gl, program, &source.uniforms);

        Ok(Self {
            handle: Some(program),
            vertex_layout: source.vertex_layout.clone(),
            uniforms: gl_uniforms,
        })
    }

    /// Destroys this shader program
    pub(crate) fn destroy(mut self, gl: &Gl) {
        let handle = self.handle.take().unwrap();
        destroy_program(gl, handle);
    }

    /// Calls [Gl::UseProgram] with this shaderprogram
    pub(crate) fn use_program(&self, gl: &Gl) {
        let handle_int = self.handle.unwrap().get();

        unsafe {
            gl.UseProgram(handle_int);
        }
    }

    /// Returns the vertex layout for this shader
    pub(crate) const fn get_vertex_layout(&self) -> &ShaderVertexLayout {
        &self.vertex_layout
    }
}

fn compile_stage(gl: &Gl, source: &str, stage: GLuint) -> Result<NonZero<GLuint>, CompileErr> {
    if !source.is_ascii() {
        return Err(CompileErr::SourceEncoding);
    }

    let source_c = CString::new(source).map_err(|_| CompileErr::SourceEncoding)?;

    let shader: NonZero<GLuint> =
        NonZero::new(unsafe { gl.CreateShader(stage) }).ok_or(CompileErr::Handle)?;
    checkerr!(gl);

    let source_ptr = source_c.as_ptr();

    // Actually load the shader source into OpenGL and compile it
    unsafe {
        gl.ShaderSource(shader.get(), 1, &source_ptr, null());
        checkerr!(gl);
        gl.CompileShader(shader.get());
        checkerr!(gl);
    };

    // Now check if the compilation was succesful
    let mut success = opengl::FALSE as GLint;

    unsafe {
        gl.GetShaderiv(shader.get(), opengl::COMPILE_STATUS, &raw mut success);
        checkerr!(gl);
    }

    if success == (opengl::TRUE as GLint) {
        Ok(shader)
    } else {
        Err(CompileErr::Compile(get_shader_compile_err(gl, shader)))
    }
}

fn get_shader_compile_err(gl: &Gl, shader: NonZero<GLuint>) -> String {
    let mut buflen: GLint = 0;

    unsafe {
        gl.GetShaderiv(shader.get(), opengl::INFO_LOG_LENGTH, &raw mut buflen);
        checkerr!(gl);
    }

    assert!(buflen >= 0);

    let mut buf = vec![0u8; buflen as usize];

    unsafe {
        gl.GetShaderInfoLog(
            shader.get(),
            buflen,
            null_mut(),
            buf.as_mut_ptr() as *mut GLchar,
        );
        checkerr!(gl);
    }

    CStr::from_bytes_with_nul(&buf)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

fn get_program_link_err(gl: &Gl, program: NonZero<GLuint>) -> String {
    let mut buflen: GLint = 0;

    unsafe {
        gl.GetProgramiv(program.get(), opengl::INFO_LOG_LENGTH, &raw mut buflen);
        checkerr!(gl);
    }

    assert!(buflen >= 0);

    let mut buf = vec![0u8; buflen as usize];

    unsafe {
        gl.GetProgramInfoLog(
            program.get(),
            buflen,
            null_mut(),
            buf.as_mut_ptr() as *mut GLchar,
        );
        checkerr!(gl);
    }

    CStr::from_bytes_with_nul(&buf)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

fn destroy_stage(gl: &Gl, shader: NonZero<GLuint>) {
    unsafe {
        gl.DeleteShader(shader.get());
        checkerr!(gl);
    }
}

fn destroy_program(gl: &Gl, program: NonZero<GLuint>) {
    unsafe {
        gl.DeleteProgram(program.get());
        checkerr!(gl);
    }
}

fn discover_uniforms(
    gl: &Gl,
    program: NonZero<GLuint>,
    declared_uniforms: &HashMap<String, Uniform>,
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

    // Now actually query each uniform. If they match one if the input uniforms,
    // return its information.
    let mut found_uniforms = HashMap::with_capacity(declared_uniforms.len());

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

        if !declared_uniforms.contains_key(name) {
            log::debug!(
                "Uniform {} was not found in the expected uniform map, skipping",
                name
            );
            continue;
        }

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
            index,
            uniform_type,
            uniform_size,
        };

        let prev = found_uniforms.insert(name.to_owned(), found_uniform);

        debug_assert!(prev.is_none());
    }

    log::debug!("Found uniforms: {:#?}", found_uniforms);

    found_uniforms
}
