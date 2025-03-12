//! Module for OpenGL shader functionality, and mapping from the abstract WutEngine [Shader] type

use core::ffi::CStr;
use core::num::NonZero;
use core::ptr::{null, null_mut};
use std::collections::HashMap;
use std::ffi::CString;

use glam::Mat4;
use thiserror::Error;
use uniform::{GlShaderUniform, discover_uniforms, set_texture_from_buf};
use wutengine_graphics::material::MaterialParameter;
use wutengine_graphics::renderer::RendererTextureId;
use wutengine_graphics::shader::SharedShaderUniform;
use wutengine_graphics::shader::{Shader, ShaderVertexLayout};

use crate::error::checkerr;
use crate::opengl::types::{GLchar, GLenum, GLint, GLuint};
use crate::opengl::{self, Gl};
use crate::texture::GlTexture;

mod uniform;

/// A fully-linked and ready to use OpenGL shader program..
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

/// An error while creating a [GlShaderProgram]
#[derive(Debug, Error)]
pub(crate) enum CreateErr {
    /// No shader source
    #[error("Shader has no source code")]
    Empty,

    /// No handle
    #[error("OpenGL did not return a shader program handle")]
    Handle,

    /// Compile error
    #[error("Could not compile shader stage")]
    Compile(#[from] CompileErr),

    /// Link error
    #[error("Could not link shader program")]
    Link(String),
}

#[derive(Debug, Error)]
pub(crate) enum CompileErr {
    /// No handle
    #[error("OpenGL did not return a shader handle")]
    Handle,

    /// Non-ascii chars in source code
    #[error("Source code contained non-ascii or NUL character")]
    SourceEncoding,

    /// Compile error
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
        let gl_uniforms = discover_uniforms(gl, program);

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

    /// Sets the default values for all uniforms on this shader _*NOT*_ in `set_uniforms`.
    /// The program must currently be in use, by calling [Self::use_program]
    pub(crate) fn set_uniform_defaults<P>(
        &mut self,
        gl: &Gl,
        set_uniforms: &HashMap<String, P>,
        first_free_tex_unit: &mut GLenum,
        default_texture: &mut GlTexture,
    ) {
        let mut fake_mappings = HashMap::new();

        for (uform_name, uform) in &self.uniforms {
            if set_uniforms.contains_key(uform_name) {
                // No need to set a default because we're gonna set this value later
                continue;
            }

            let default_val = match uform.uniform_type {
                opengl::BOOL => MaterialParameter::DEFAULT_BOOL,
                opengl::FLOAT_VEC4 => MaterialParameter::DEFAULT_VEC4,
                opengl::FLOAT_MAT4 => MaterialParameter::DEFAULT_MAT4,
                opengl::SAMPLER_2D => unsafe {
                    set_texture_from_buf(gl, default_texture, uform.location, first_free_tex_unit);
                    continue;
                },
                _ => panic!("Missing default value for type {}", uform.uniform_type),
            };

            let cur_tex_unit = *first_free_tex_unit;

            let ok = uniform::set_uniform_value(
                gl,
                &default_val,
                uform,
                first_free_tex_unit,
                &mut fake_mappings,
            );

            assert!(
                ok,
                "Failed to set default uniform value of uniform {} to {:#?}",
                uform_name, default_val
            );

            debug_assert_eq!(
                cur_tex_unit, *first_free_tex_unit,
                "Set a texture when we didn't expect to"
            );
        }
    }

    /// Sets the given uniforms on this program. The program must currently be in use, by calling [Self::use_program]
    pub(crate) fn set_uniforms(
        &mut self,
        gl: &Gl,
        uniforms: &HashMap<String, MaterialParameter>,
        first_free_tex_unit: &mut GLenum,
        texture_mappings: &mut HashMap<RendererTextureId, GlTexture>,
    ) {
        uniform::set_uniforms(
            gl,
            self.handle.expect("Program destroyed"),
            uniforms,
            &self.uniforms,
            first_free_tex_unit,
            texture_mappings,
        );
    }

    /// Sets the model/view/projection matrix uniforms on this shader
    pub(crate) fn set_mvp(&mut self, gl: &Gl, model: Mat4, view: Mat4, projection: Mat4) {
        assert!(self.handle.is_some(), "ShaderProgram already destroyed");

        let model_uform = self.uniforms.get(SharedShaderUniform::ModelMat.as_str());
        let view_uform = self.uniforms.get(SharedShaderUniform::ViewMat.as_str());
        let projection_uform = self
            .uniforms
            .get(SharedShaderUniform::ProjectionMat.as_str());

        // Fake variables. We're not using the texture units here anyway
        let mut tex_unit = 0;
        let mut tex_maps = HashMap::new();

        if let Some(model_uform) = model_uform {
            uniform::set_uniform_value(
                gl,
                &MaterialParameter::Mat4(model),
                model_uform,
                &mut tex_unit,
                &mut tex_maps,
            );
        } else {
            log::debug!(
                "Model uniform not found on shaderprogram {}",
                self.handle.unwrap()
            );
        }

        if let Some(view_uform) = view_uform {
            uniform::set_uniform_value(
                gl,
                &MaterialParameter::Mat4(view),
                view_uform,
                &mut tex_unit,
                &mut tex_maps,
            );
        } else {
            log::debug!(
                "View uniform not found on shaderprogram {}",
                self.handle.unwrap()
            );
        }

        if let Some(projection_uform) = projection_uform {
            uniform::set_uniform_value(
                gl,
                &MaterialParameter::Mat4(projection),
                projection_uform,
                &mut tex_unit,
                &mut tex_maps,
            );
        } else {
            log::debug!(
                "Projection uniform not found on shaderprogram {}",
                self.handle.unwrap()
            );
        }
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
