//! Module for OpenGL shader functionality, and mapping from the abstract WutEngine [Shader] type

use core::num::NonZero;
use core::ptr::null;
use std::collections::HashMap;
use std::ffi::CString;

use reflection::{get_program_link_err, get_shader_compile_err, get_uniform_block_index};
use thiserror::Error;
use uniform::GlShaderUniform;
use uniform::discovery::discover_uniforms;
use wutengine_graphics::shader::CompiledShader;
use wutengine_graphics::shader::ShaderVertexLayout;
use wutengine_graphics::shader::builtins::ShaderBuiltins;
use wutengine_graphics::shader::uniform::SingleUniformBinding;

use crate::debug;
use crate::error::checkerr;
use crate::opengl::types::{GLint, GLuint};
use crate::opengl::{self, Gl};

mod reflection;
pub(crate) mod uniform;

/// A fully-linked and ready to use OpenGL shader program..
#[derive(Debug)]
pub(crate) struct GlShaderProgram {
    handle: Option<NonZero<GLuint>>,
    vertex_layout: ShaderVertexLayout,

    /// The builtins this shader uses, and how they are bound
    pub(crate) builtins: HashMap<ShaderBuiltins, Vec<SingleUniformBinding>>,

    /// The non-builtin uniforms, and how they are bound
    pub(crate) uniforms: HashMap<String, GlShaderUniform>,
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

#[profiling::all_functions]
impl GlShaderProgram {
    /// Creates, compiles, and links a new OpenGL shaderprogram from the given source
    pub(crate) fn new(gl: &Gl, source: &CompiledShader) -> Result<Self, CreateErr> {
        let _dbg_marker =
            debug::debug_marker_group(gl, || format!("Compile ShaderProgram {}", source.id));

        if !source.source.has_any() {
            return Err(CreateErr::Empty);
        }

        let program: NonZero<GLuint> =
            NonZero::new(unsafe { gl.CreateProgram() }).ok_or(CreateErr::Handle)?;
        checkerr!(gl);

        // Compile the stages one-by-one, cleaning up the partial program if we encounter an error
        let vertex = if let Some(vtx_src) = &source.source.vertex {
            let compiled = compile_stage(gl, &vtx_src.source, opengl::VERTEX_SHADER);

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
            let compiled = compile_stage(gl, &frg_src.source, opengl::FRAGMENT_SHADER);

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

        // Everything should be linked now, so we can destroy the individual stages
        if let Some(sh) = vertex {
            destroy_stage(gl, sh)
        }
        if let Some(sh) = fragment {
            destroy_stage(gl, sh)
        }

        if success != (opengl::TRUE as GLint) {
            let err = get_program_link_err(gl, program);

            destroy_program(gl, program);

            return Err(CreateErr::Link(err));
        }

        unsafe {
            gl.UseProgram(program.get());
        }

        checkerr!(gl);

        debug::add_debug_label(gl, program, debug::DebugObjType::ShaderProgram, || {
            Some(format!("shaderprogram:{}", source.id))
        });

        // Program succesfully compiled. Now find all builtins and uniforms as declared by the source shader

        let builtins = discover_builtins(gl, program, source);
        let gl_uniforms = discover_uniforms(gl, program, &source.uniforms);

        Ok(Self {
            handle: Some(program),
            vertex_layout: source.vertex_layout.clone(),
            builtins,
            uniforms: gl_uniforms,
        })
    }

    #[inline(always)]
    pub(crate) const fn handle(&self) -> NonZero<GLuint> {
        self.handle.expect("ShaderProgram already freed")
    }

    /// Destroys this shader program
    pub(crate) fn destroy(mut self, gl: &Gl) {
        let handle = self.handle.take().unwrap();
        destroy_program(gl, handle);
    }

    /// Returns the vertex layout for this shader
    #[profiling::skip]
    pub(crate) const fn get_vertex_layout(&self) -> &ShaderVertexLayout {
        &self.vertex_layout
    }
}

#[profiling::function]
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

#[profiling::function]
fn destroy_stage(gl: &Gl, shader: NonZero<GLuint>) {
    unsafe {
        gl.DeleteShader(shader.get());
        checkerr!(gl);
    }
}

#[profiling::function]
fn destroy_program(gl: &Gl, program: NonZero<GLuint>) {
    unsafe {
        gl.DeleteProgram(program.get());
        checkerr!(gl);
    }
}

#[profiling::function]
fn discover_builtins(
    gl: &Gl,
    program: NonZero<GLuint>,
    shader: &CompiledShader,
) -> HashMap<ShaderBuiltins, Vec<SingleUniformBinding>> {
    let meta = shader.target_meta.as_opengl().unwrap();

    let mut mapped_builtins = HashMap::new();

    discover_builtins_stage(gl, program, &meta.builtins_vertex, &mut mapped_builtins);
    discover_builtins_stage(gl, program, &meta.builtins_fragment, &mut mapped_builtins);

    mapped_builtins
}

fn discover_builtins_stage(
    gl: &Gl,
    program: NonZero<GLuint>,
    stage_builtins: &HashMap<ShaderBuiltins, SingleUniformBinding>,
    output: &mut HashMap<ShaderBuiltins, Vec<SingleUniformBinding>>,
) {
    for (&builtin, binding) in stage_builtins {
        let block_index = match get_uniform_block_index(gl, program, &binding.name) {
            Some(i) => i,
            None => {
                log::error!(
                    "Could not find index for builtin block {:#?} with binding {}",
                    builtin,
                    binding
                );
                continue;
            }
        };

        unsafe {
            gl.UniformBlockBinding(program.get(), block_index, binding.binding as GLuint);
        }

        checkerr!(gl);

        output.entry(builtin).or_default().push(binding.clone());
    }
}
