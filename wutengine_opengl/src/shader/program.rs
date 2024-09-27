use core::ffi::c_char;
use core::num::NonZero;
use std::collections::HashMap;
use std::ffi::CString;
use std::ptr::null_mut;

use thiserror::Error;
use wutengine_graphics::material::MaterialParameter;

use crate::opengl::types::{GLint, GLuint};
use crate::opengl::{self, Gl};
use crate::shader::CompileErr;

use super::set::GlShaderSet;
use super::uniform::UniformDescriptor;

#[derive(Debug)]
pub struct ShaderProgram {
    data: ShaderProgramData,
    uniforms: HashMap<String, UniformDescriptor>,
}

#[derive(Debug)]
enum ShaderProgramData {
    Linked {
        handle: NonZero<GLuint>,
    },
    Unlinked {
        handle: NonZero<GLuint>,
        shaders: GlShaderSet,
    },
    Destroyed,
}

#[derive(Debug, Error)]
pub enum CreateErr {
    #[error("OpenGL returned zero")]
    Zero,
}

#[derive(Debug, Error)]
pub enum LinkErr {
    #[error("Shader failed to compile")]
    ShaderCompile(#[from] CompileErr),

    #[error("ShaderProgram failed to link: {}", 0)]
    Link(String),
}

impl ShaderProgram {
    pub fn new(gl: &Gl, stages: GlShaderSet) -> Result<Self, CreateErr> {
        let handle = unsafe { gl.CreateProgram() };
        let handle = NonZero::new(handle).ok_or(CreateErr::Zero)?;

        Ok(Self {
            data: ShaderProgramData::Unlinked {
                handle,
                shaders: stages,
            },
            uniforms: HashMap::new(),
        })
    }

    pub fn destroy(mut self, gl: &Gl) {
        log::debug!("Destroying ShaderProgram");

        if matches!(self.data, ShaderProgramData::Destroyed) {
            panic!("ShaderProgram already destroyed");
        }

        if let ShaderProgramData::Unlinked { shaders, .. } = &mut self.data {
            shaders.destroy_all(gl);
        }

        self.destroy_program(gl);
    }

    fn destroy_program(&mut self, gl: &Gl) {
        let handle = self.get_handle();

        unsafe {
            gl.DeleteProgram(handle.get());
        }

        self.data = ShaderProgramData::Destroyed;
    }

    fn get_handle(&self) -> NonZero<GLuint> {
        match self.data {
            ShaderProgramData::Unlinked { handle, .. } => handle,
            ShaderProgramData::Linked { handle, .. } => handle,
            ShaderProgramData::Destroyed => {
                panic!("Trying to get the handle to a destroyed shader program")
            }
        }
    }

    fn get_shaders(&mut self) -> &mut GlShaderSet {
        match &mut self.data {
            ShaderProgramData::Linked { .. } => {
                panic!("Trying to get shader stages of an already linked shader program")
            }
            ShaderProgramData::Destroyed => {
                panic!("Trying to get shader stages of a destroyed shader program")
            }
            ShaderProgramData::Unlinked { shaders, .. } => shaders,
        }
    }

    fn link(&mut self, gl: &Gl) -> Result<(), LinkErr> {
        log::debug!("Linking ShaderProgram");

        let handle = self.get_handle();
        let shaders = self.get_shaders();

        shaders.compile_all(gl)?;
        shaders.attach_all(gl, handle);

        unsafe {
            gl.LinkProgram(handle.get());

            let mut success: GLint = opengl::FALSE as GLint;

            gl.GetProgramiv(handle.get(), opengl::LINK_STATUS, &mut success);

            if success != (opengl::TRUE as GLint) {
                let mut buflen: GLint = 512;

                gl.GetProgramiv(handle.get(), opengl::INFO_LOG_LENGTH, &mut buflen);

                let mut logbuf: Vec<u8> = vec![0; buflen as usize];

                gl.GetProgramInfoLog(
                    handle.get(),
                    buflen,
                    null_mut(),
                    logbuf.as_mut_ptr() as *mut c_char,
                );

                let logstr = CString::new(logbuf).unwrap();

                return Err(LinkErr::Link(logstr.to_string_lossy().to_string()));
            }
        }

        shaders.detach_all(gl, handle);
        shaders.destroy_all(gl);

        self.data = ShaderProgramData::Linked { handle };

        self.resolve_uniforms(gl);

        Ok(())
    }

    fn resolve_uniforms(&mut self, gl: &Gl) {
        log::debug!("Resolving uniforms");

        if !matches!(self.data, ShaderProgramData::Linked { .. }) {
            panic!("Cannot resolve uniforms for unlinked program");
        }

        let uniforms = unsafe { UniformDescriptor::get_for(gl, self.get_handle()) };

        log::debug!("Resolved uniforms: {:#?}", uniforms);

        for (name, descriptor) in uniforms {
            self.uniforms.insert(name, descriptor);
        }
    }

    pub fn ensure_linked(&mut self, gl: &Gl) -> Result<(), LinkErr> {
        if matches!(self.data, ShaderProgramData::Destroyed) {
            panic!("Trying to use a destroyed shader program");
        }

        if matches!(self.data, ShaderProgramData::Unlinked { .. }) {
            self.link(gl)
        } else {
            Ok(())
        }
    }

    pub fn assert_linked(&self) -> NonZero<GLuint> {
        if let ShaderProgramData::Linked { handle } = self.data {
            handle
        } else {
            panic!("Program not linked!");
        }
    }

    pub fn use_program(&mut self, gl: &Gl) -> Result<(), LinkErr> {
        if matches!(self.data, ShaderProgramData::Destroyed) {
            panic!("Trying to use a destroyed shader program");
        }

        if matches!(self.data, ShaderProgramData::Unlinked { .. }) {
            self.link(gl)?;
        }

        let handle = self.get_handle();

        unsafe {
            gl.UseProgram(handle.get());
        }

        Ok(())
    }

    pub unsafe fn set_uniforms(
        &mut self,
        gl: &Gl,
        parameters: &HashMap<impl AsRef<str>, MaterialParameter>,
    ) -> Result<(), SetUniformErr> {
        _ = self.assert_linked();

        for (name, value) in parameters {
            unsafe {
                self.set_uniform(gl, name, value)?;
            }
        }

        Ok(())
    }

    pub unsafe fn set_uniform(
        &mut self,
        gl: &Gl,
        name: impl AsRef<str>,
        value: &MaterialParameter,
    ) -> Result<(), SetUniformErr> {
        let uniform_descriptor = self
            .uniforms
            .get_mut(name.as_ref())
            .ok_or_else(|| SetUniformErr::UnknownParam(name.as_ref().to_owned()))?;

        let ok = unsafe { uniform_descriptor.set_with(gl, value) };

        if !ok {
            return Err(SetUniformErr::InvalidParam(name.as_ref().to_owned()));
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum SetUniformErr {
    #[error("Unknown parameter: {}", 0)]
    UnknownParam(String),

    #[error("Parameter {} had an invalid value", 0)]
    InvalidParam(String),
}

#[cfg(debug_assertions)]
impl Drop for ShaderProgram {
    fn drop(&mut self) {
        if matches!(self.data, ShaderProgramData::Destroyed) {
            log::warn!("ShaderProgram dropped without being destroyed!");
        }
    }
}
