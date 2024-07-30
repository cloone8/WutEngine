use core::num::NonZero;

use wutengine_graphics::shader::{ShaderSet, ShaderStage};

use crate::opengl::types::GLuint;
use crate::opengl::Gl;

use super::{CompileErr, CreateErr, Fragment, Shader, ShaderType, Vertex};

#[derive(Debug)]
pub struct GlShaderSet {
    pub vertex: Option<Shader<Vertex>>,
    pub fragment: Option<Shader<Fragment>>,
}

impl GlShaderSet {
    pub fn compile_all(&mut self, gl: &Gl) -> Result<(), CompileErr> {
        if let Some(vtx) = &mut self.vertex {
            _ = vtx.get_compiled(gl)?;
        };

        if let Some(frag) = &mut self.fragment {
            _ = frag.get_compiled(gl)?;
        };

        Ok(())
    }

    pub fn attach_all(&mut self, gl: &Gl, program: NonZero<GLuint>) {
        unsafe {
            if let Some(sh) = self.vertex.as_mut() {
                gl.AttachShader(program.get(), sh.assert_compiled().get())
            }
            if let Some(sh) = self.fragment.as_mut() {
                gl.AttachShader(program.get(), sh.assert_compiled().get())
            }
        }
    }

    pub fn detach_all(&mut self, gl: &Gl, program: NonZero<GLuint>) {
        unsafe {
            if let Some(sh) = self.vertex.as_mut() {
                gl.DetachShader(program.get(), sh.assert_compiled().get())
            }
            if let Some(sh) = self.fragment.as_mut() {
                gl.DetachShader(program.get(), sh.assert_compiled().get())
            }
        }
    }

    pub fn destroy_all(&mut self, gl: &Gl) {
        if let Some(mut sh) = self.vertex.take() {
            sh.destroy(gl)
        }
        if let Some(mut sh) = self.fragment.take() {
            sh.destroy(gl)
        }
    }

    pub fn from_sources(gl: &Gl, sources: &ShaderSet) -> Result<Self, CreateErr> {
        Ok(Self {
            vertex: to_gl_shader(gl, sources.get_stage(ShaderStage::Vertex))?,
            fragment: to_gl_shader(gl, sources.get_stage(ShaderStage::Fragment))?,
        })
    }
}

fn to_gl_shader<T: ShaderType>(
    gl: &Gl,
    stage_src: Option<&str>,
) -> Result<Option<Shader<T>>, CreateErr> {
    match stage_src {
        Some(src) => Ok(Some(Shader::<T>::new(gl, src)?)),
        None => Ok(None),
    }
}
