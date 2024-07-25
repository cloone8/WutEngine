use core::num::NonZero;
use std::{marker::PhantomData, ptr::NonNull};

use thiserror::Error;

use crate::opengl::{self, types::GLuint, Gl};

pub unsafe trait ShaderType {
    const GL_SHADER_TYPE: GLuint;
}

pub struct Vertex;
pub struct Fragment;

unsafe impl ShaderType for Vertex {
    const GL_SHADER_TYPE: GLuint = opengl::VERTEX_SHADER;
}
unsafe impl ShaderType for Fragment {
    const GL_SHADER_TYPE: GLuint = opengl::FRAGMENT_SHADER;
}

#[derive(Debug, Clone, Copy, Error)]
pub enum CreateErr {
    #[error("OpenGL returned 0")]
    Zero,
}

pub struct Shader<T: ShaderType> {
    handle: Option<NonZero<GLuint>>,
    phantom: PhantomData<T>,
}

impl<T: ShaderType> Shader<T> {
    pub fn new(gl: &Gl) -> Result<Self, CreateErr> {
        let handle = unsafe { gl.CreateShader(T::GL_SHADER_TYPE) };
        let handle = NonZero::new(handle).ok_or(CreateErr::Zero)?;

        Ok(Self {
            handle: Some(handle),
            phantom: PhantomData,
        })
    }

    pub fn destroy(mut self, gl: &Gl) {
        if let Some(handle) = self.handle.take() {
            let as_int = handle.get();

            unsafe {
                gl.DeleteShader(as_int);
            }
        }
    }
}

#[cfg(debug_assertions)]
impl<T: ShaderType> Drop for Shader<T> {
    fn drop(&mut self) {
        if self.handle.is_some() {
            log::warn!("Shader dropped without being destroyed!");
        }
    }
}
