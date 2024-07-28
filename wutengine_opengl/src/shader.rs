use core::ffi::{c_char, CStr};
use core::fmt::Debug;
use core::num::NonZero;
use std::cell::RefCell;
use std::ffi::CString;
use std::marker::PhantomData;
use std::ptr::{null, null_mut};

use include_dir::Dir;
use thiserror::Error;

use crate::embedded;
use crate::opengl::types::GLint;
use crate::opengl::{self, types::GLuint, Gl};
use crate::shaderprogram::ShaderSet;

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

    pub fn destroy(mut self, gl: &Gl) {
        log::debug!("Destroying shader");

        self.data.destroy(gl);
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

/// Public API
impl ShaderData {
    pub fn new<T: ShaderType>(gl: &Gl, src: &str) -> Result<Self, CreateErr> {
        let handle = unsafe { gl.CreateShader(T::GL_SHADER_TYPE) };
        let handle = NonZero::new(handle).ok_or(CreateErr::Zero)?;

        Ok(Self::Uncompiled {
            handle,
            source: CString::new(src).unwrap(),
        })
    }

    pub fn destroy(&mut self, gl: &Gl) {
        log::debug!("Destroying shaderdata: {:?}", self);

        let handle = match self {
            Self::Uncompiled { handle, .. } => *handle,
            Self::Compiled { handle } => *handle,
            Self::Destroyed => panic!("Trying to destroy an already destroyed shader"),
        };

        let as_int = handle.get();

        unsafe {
            gl.DeleteShader(as_int);
        }

        *self = Self::Destroyed;
    }
}

/// Internal API
impl ShaderData {
    pub fn is_compiled(&self) -> bool {
        matches!(self, Self::Compiled { .. })
    }

    pub fn get_handle(&self) -> NonZero<GLuint> {
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

    pub fn do_compile(&mut self, gl: &Gl) -> Result<(), CompileErr> {
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

                let logstr = CString::new(logbuf).unwrap();

                return Err(CompileErr::Gl(logstr.to_string_lossy().to_string()));
            } else {
                *self = ShaderData::Compiled { handle };
                Ok(())
            }
        }
    }
}

#[cfg(debug_assertions)]
impl Drop for ShaderData {
    fn drop(&mut self) {
        if !matches!(self, Self::Destroyed) {
            log::warn!("Shader {:#?} dropped without being destroyed!", self);
        }
    }
}

fn load_builtin_from_dir(gl: &Gl, shader_dir: &Dir) -> Result<ShaderSet, CreateErr> {
    let base_path = shader_dir.path().to_path_buf();

    let vertex = if let Some(file) = shader_dir.get_file(base_path.join("vertex.glsl")) {
        let source = file.contents_utf8().expect("Non UTF-8 shaderfile content");
        Some(Shader::<Vertex>::new(gl, source)?)
    } else {
        None
    };

    let fragment = if let Some(file) = shader_dir.get_file(base_path.join("fragment.glsl")) {
        let source = file.contents_utf8().expect("Non UTF-8 shaderfile content");
        Some(Shader::<Fragment>::new(gl, source)?)
    } else {
        None
    };

    Ok(ShaderSet { vertex, fragment })
}

pub fn load_builtin(gl: &Gl, identifier: &str) -> Option<Result<ShaderSet, CreateErr>> {
    log::debug!("Loading builtin shader: {}", identifier);

    let shader_dir = embedded::SHADERS.get_dir(identifier)?;

    Some(load_builtin_from_dir(gl, shader_dir))
}
