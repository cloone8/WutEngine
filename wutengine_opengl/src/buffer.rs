use core::ffi::c_void;
use core::marker::PhantomData;
use core::num::NonZero;

use thiserror::Error;

use crate::opengl::types::{GLenum, GLuint};
use crate::opengl::{self, Gl};

pub(crate) trait GlBufferType {
    const GL_BUFTYPE: GLenum;
}

#[derive(Debug)]
pub(crate) struct ArrayBuffer;

impl GlBufferType for ArrayBuffer {
    const GL_BUFTYPE: GLenum = opengl::ARRAY_BUFFER;
}

#[derive(Debug)]
pub(crate) struct ElementArrayBuffer;

impl GlBufferType for ElementArrayBuffer {
    const GL_BUFTYPE: GLenum = opengl::ELEMENT_ARRAY_BUFFER;
}

#[derive(Debug)]
pub(crate) struct GlBuffer<T> {
    handle: Option<NonZero<GLuint>>,
    phantom: PhantomData<T>,
}

#[derive(Debug, Clone, Copy, Error)]
pub(crate) enum CreateErr {
    #[error("OpenGL returned 0")]
    Zero,
}

impl<B: GlBufferType> GlBuffer<B> {
    pub(crate) fn new(gl: &Gl) -> Result<Self, CreateErr> {
        let mut handle = 0;

        unsafe {
            gl.GenBuffers(1, &mut handle);
        }

        let handle = NonZero::new(handle).ok_or(CreateErr::Zero)?;

        Ok(Self {
            handle: Some(handle),
            phantom: PhantomData,
        })
    }

    pub(crate) fn bind(&mut self, gl: &Gl) {
        unsafe {
            let handle_int = self.handle.unwrap().get();

            gl.BindBuffer(B::GL_BUFTYPE, handle_int);
        }
    }

    pub(crate) fn unbind(&mut self, gl: &Gl) {
        unsafe {
            gl.BindBuffer(B::GL_BUFTYPE, 0);
        }
    }

    pub(crate) fn buffer_data<T: Copy>(&mut self, gl: &Gl, data: &[T]) {
        unsafe {
            gl.BufferData(
                B::GL_BUFTYPE,
                std::mem::size_of_val(data) as isize,
                data.as_ptr() as *const c_void,
                opengl::STATIC_DRAW,
            );
        }
    }

    pub(crate) fn destroy(mut self, gl: &Gl) {
        if let Some(handle) = self.handle.take() {
            let as_int = handle.get();

            unsafe {
                gl.DeleteBuffers(1, &as_int);
            }
        }
    }
}

#[cfg(debug_assertions)]
impl<B> Drop for GlBuffer<B> {
    fn drop(&mut self) {
        if self.handle.is_some() {
            log::warn!(
                "GL buffer of type {} dropped without being destroyed!",
                std::any::type_name::<B>()
            );
        }
    }
}
