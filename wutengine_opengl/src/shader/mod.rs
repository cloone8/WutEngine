use core::ffi::{c_char, CStr};
use core::fmt::Debug;
use core::num::NonZero;
use std::ffi::CString;
use std::marker::PhantomData;
use std::ptr::{null, null_mut};

use thiserror::Error;

use crate::opengl::types::GLint;
use crate::opengl::{self, types::GLuint, Gl};

pub mod attribute;
pub mod program;
pub mod set;
pub mod uniform;

pub unsafe trait ShaderType: Debug {
    const GL_SHADER_TYPE: GLuint;
}

#[derive(Debug)]
pub struct Vertex;

#[derive(Debug)]
pub struct Fragment;

unsafe impl ShaderType for Vertex {
    const GL_SHADER_TYPE: GLuint = opengl::VERTEX_SHADER;
}
unsafe impl ShaderType for Fragment {
    const GL_SHADER_TYPE: GLuint = opengl::FRAGMENT_SHADER;
}

#[derive(Debug, Error)]
pub enum CreateErr {
    #[error("OpenGL returned 0")]
    Zero,
}

#[derive(Debug, Error)]
pub enum CompileErr {
    #[error("OpenGL returned error message during shader compilation: {}", 0)]
    Gl(String),

    #[error("Shader was already compiled")]
    AlreadyCompiled,
}

#[derive(Debug)]
pub struct Shader<T: ShaderType> {
    data: ShaderData,
    phantom: PhantomData<T>,
}

impl<T: ShaderType> Shader<T> {
    pub fn new(gl: &Gl, source: &str) -> Result<Self, CreateErr> {
        Ok(Self {
            data: ShaderData::new::<T>(gl, source)?,
            phantom: PhantomData,
        })
    }

    pub fn get_compiled(&mut self, gl: &Gl) -> Result<NonZero<GLuint>, CompileErr> {
        if !self.data.is_compiled() {
            self.data.do_compile(gl)?;

            debug_assert!(self.data.is_compiled(), "Should be compiled now!");
        }

        Ok(self.data.get_handle())
    }

    pub fn assert_compiled(&self) -> NonZero<GLuint> {
        assert!(self.data.is_compiled());

        self.data.get_handle()
    }
    pub fn destroy(&mut self, gl: &Gl) {
        log::trace!("Destroying shader: {:?}", self);

        let handle = match self.data {
            ShaderData::Uncompiled { handle, .. } => handle,
            ShaderData::Compiled { handle } => handle,
            ShaderData::Destroyed => panic!("Trying to destroy an already destroyed shader"),
        };

        let as_int = handle.get();

        unsafe {
            gl.DeleteShader(as_int);
        }

        self.data = ShaderData::Destroyed;
    }
}

#[derive(Debug)]
enum ShaderData {
    Uncompiled {
        handle: NonZero<GLuint>,
        source: CString,
    },
    Compiled {
        handle: NonZero<GLuint>,
    },
    Destroyed,
}

impl ShaderData {
    fn new<T: ShaderType>(gl: &Gl, src: &str) -> Result<Self, CreateErr> {
        let handle = unsafe { gl.CreateShader(T::GL_SHADER_TYPE) };
        let handle = NonZero::new(handle).ok_or(CreateErr::Zero)?;

        Ok(Self::Uncompiled {
            handle,
            source: CString::new(src).unwrap(),
        })
    }

    fn is_compiled(&self) -> bool {
        matches!(self, Self::Compiled { .. })
    }

    fn get_handle(&self) -> NonZero<GLuint> {
        match self {
            Self::Uncompiled { handle, .. } => *handle,
            Self::Compiled { handle, .. } => *handle,
            Self::Destroyed => panic!("Trying to get the handle to a destroyed shader"),
        }
    }

    fn get_source(&self) -> Option<&CStr> {
        match self {
            Self::Uncompiled { source, .. } => Some(source.as_c_str()),
            Self::Compiled { .. } => None,
            Self::Destroyed => panic!("Trying to get the source to a destroyed shader"),
        }
    }

    fn do_compile(&mut self, gl: &Gl) -> Result<(), CompileErr> {
        if matches!(self, Self::Destroyed) {
            panic!("Trying to compile a destroyed shader");
        }

        log::debug!("Compiling shader");

        if self.is_compiled() {
            return Err(CompileErr::AlreadyCompiled);
        }

        let handle = self.get_handle();
        let source = self.get_source().unwrap();

        unsafe {
            gl.ShaderSource(handle.get(), 1, &source.as_ptr(), null());
            gl.CompileShader(handle.get());

            let mut success: GLint = opengl::FALSE as GLint;

            gl.GetShaderiv(handle.get(), opengl::COMPILE_STATUS, &mut success);

            if success != (opengl::TRUE as GLint) {
                let mut buflen: GLint = 512;

                gl.GetShaderiv(handle.get(), opengl::INFO_LOG_LENGTH, &mut buflen);

                let mut logbuf: Vec<u8> = vec![0; buflen as usize];

                gl.GetShaderInfoLog(
                    handle.get(),
                    buflen,
                    null_mut(),
                    logbuf.as_mut_ptr() as *mut c_char,
                );

                let logstr = CString::from_vec_with_nul(logbuf).unwrap();

                return Err(CompileErr::Gl(logstr.to_string_lossy().to_string()));
            } else {
                *self = ShaderData::Compiled { handle };
                Ok(())
            }
        }
    }
}

#[cfg(debug_assertions)]
impl<T: ShaderType> Drop for Shader<T> {
    fn drop(&mut self) {
        if !matches!(self.data, ShaderData::Destroyed) {
            log::warn!("Shader {:#?} dropped without being destroyed!", self);
        }
    }
}
