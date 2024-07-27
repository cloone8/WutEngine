use core::ffi::c_char;
use core::num::NonZero;
use std::ffi::CString;
use std::ptr::null_mut;

use thiserror::Error;

use crate::opengl::types::{GLint, GLuint};
use crate::opengl::{self, Gl};
use crate::shader::{CompileErr, Fragment, Shader, Vertex};

#[derive(Debug)]
pub struct ShaderSet {
    pub vertex: Option<Shader<Vertex>>,
    pub fragment: Option<Shader<Fragment>>,
}

impl ShaderSet {
    fn compile_all(&mut self, gl: &Gl) -> Result<(), CompileErr> {
        if let Some(vtx) = &mut self.vertex {
            _ = vtx.get_compiled(gl)?;
        };

        if let Some(frag) = &mut self.fragment {
            _ = frag.get_compiled(gl)?;
        };

        Ok(())
    }

    fn attach_all(&mut self, gl: &Gl, program: NonZero<GLuint>) {
        unsafe {
            if let Some(sh) = self.vertex.as_mut() {
                gl.AttachShader(program.get(), sh.assert_compiled().get())
            }
            if let Some(sh) = self.fragment.as_mut() {
                gl.AttachShader(program.get(), sh.assert_compiled().get())
            }
        }
    }

    fn detach_all(&mut self, gl: &Gl, program: NonZero<GLuint>) {
        unsafe {
            if let Some(sh) = self.vertex.as_mut() {
                gl.DetachShader(program.get(), sh.assert_compiled().get())
            }
            if let Some(sh) = self.fragment.as_mut() {
                gl.DetachShader(program.get(), sh.assert_compiled().get())
            }
        }
    }

    fn destroy_all(&mut self, gl: &Gl) {
        if let Some(sh) = self.vertex.take() {
            sh.destroy(gl)
        }
        if let Some(sh) = self.fragment.take() {
            sh.destroy(gl)
        }
    }
}

#[derive(Debug)]
pub enum ShaderProgram {
    Linked {
        handle: NonZero<GLuint>,
    },
    Unlinked {
        handle: NonZero<GLuint>,
        shaders: ShaderSet,
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
    pub fn new(gl: &Gl, stages: ShaderSet) -> Result<Self, CreateErr> {
        let handle = unsafe { gl.CreateProgram() };
        let handle = NonZero::new(handle).ok_or(CreateErr::Zero)?;

        Ok(Self::Unlinked {
            handle,
            shaders: stages,
        })
    }

    pub fn destroy(mut self, gl: &Gl) {
        log::debug!("Destroying ShaderProgram");

        if matches!(self, Self::Destroyed) {
            panic!("ShaderProgram already destroyed");
        }

        if let Self::Unlinked { shaders, .. } = &mut self {
            shaders.destroy_all(gl);
        }

        self.destroy_program(gl);
    }

    fn destroy_program(&mut self, gl: &Gl) {
        let handle = self.get_handle();

        unsafe {
            gl.DeleteProgram(handle.get());
        }

        *self = Self::Destroyed;
    }

    fn get_handle(&self) -> NonZero<GLuint> {
        match self {
            Self::Unlinked { handle, .. } => *handle,
            Self::Linked { handle, .. } => *handle,
            Self::Destroyed => panic!("Trying to get the handle to a destroyed shader program"),
        }
    }

    fn get_shaders(&mut self) -> &mut ShaderSet {
        match self {
            ShaderProgram::Linked { .. } => {
                panic!("Trying to get shader stages of an already linked shader program")
            }
            ShaderProgram::Destroyed => {
                panic!("Trying to get shader stages of a destroyed shader program")
            }
            ShaderProgram::Unlinked { shaders, .. } => shaders,
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

        *self = ShaderProgram::Linked { handle };

        Ok(())
    }

    pub fn use_program(&mut self, gl: &Gl) -> Result<(), LinkErr> {
        if matches!(self, Self::Destroyed) {
            panic!("Trying to use a destroyed shader program");
        }

        if matches!(self, Self::Unlinked { .. }) {
            self.link(gl)?;
        }

        let handle = self.get_handle();

        unsafe {
            gl.UseProgram(handle.get());
        }

        Ok(())
    }
}

#[cfg(debug_assertions)]
impl Drop for ShaderProgram {
    fn drop(&mut self) {
        if matches!(self, Self::Destroyed) {
            log::warn!("ShaderProgram dropped without being destroyed!");
        }
    }
}
