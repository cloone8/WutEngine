//! Abstractions around OpenGL buffers

use core::ffi::c_void;
use core::marker::PhantomData;
use core::num::NonZero;

use thiserror::Error;

use crate::error::checkerr;
use crate::opengl::types::{GLenum, GLuint};
use crate::opengl::{self, Gl};

/// A marker trait to be implemented by some empty structs to type the different
/// OpenGL buffer types statically
pub(crate) trait GlBufferType {
    /// The [GLenum] to be used in [opengl::Gl::BindBuffer] calls
    const GL_BUFTYPE: GLenum;
}

/// OpenGL array buffer (`GL_ARRAY_BUFFER`)
#[derive(Debug)]
pub(crate) struct ArrayBuffer;

impl GlBufferType for ArrayBuffer {
    const GL_BUFTYPE: GLenum = opengl::ARRAY_BUFFER;
}

/// OpenGL element array buffer (`GL_ELEMENT_ARRAY_BUFFER`)
#[derive(Debug)]
pub(crate) struct ElementArrayBuffer;

impl GlBufferType for ElementArrayBuffer {
    const GL_BUFTYPE: GLenum = opengl::ELEMENT_ARRAY_BUFFER;
}

/// A generic OpenGL buffer
#[derive(Debug)]
pub(crate) struct GlBuffer<T> {
    handle: Option<NonZero<GLuint>>,
    phantom: PhantomData<T>,
}

/// Error while creating a generic OpenGL buffer
#[derive(Debug, Clone, Copy, Error)]
pub(crate) enum CreateErr {
    /// Zero was returned
    #[error("OpenGL returned 0")]
    Zero,
}

impl<B: GlBufferType> GlBuffer<B> {
    /// Creates a new OpenGL buffer with no data
    pub(crate) fn new(gl: &Gl) -> Result<Self, CreateErr> {
        let mut handle = 0;

        unsafe {
            gl.GenBuffers(1, &mut handle);
        }
        checkerr!(gl);

        let handle = NonZero::new(handle).ok_or(CreateErr::Zero)?;

        Ok(Self {
            handle: Some(handle),
            phantom: PhantomData,
        })
    }

    /// Binds this buffer
    pub(crate) fn bind(&mut self, gl: &Gl) {
        unsafe {
            let handle_int = self.handle.unwrap().get();

            gl.BindBuffer(B::GL_BUFTYPE, handle_int);
        }
        checkerr!(gl);
    }

    /// Unbinds this buffer
    pub(crate) fn unbind(&mut self, gl: &Gl) {
        unsafe {
            gl.BindBuffer(B::GL_BUFTYPE, 0);
        }
        checkerr!(gl);
    }

    /// Uploads the given data to this buffer
    pub(crate) fn buffer_data<T: Copy>(&mut self, gl: &Gl, data: &[T]) {
        unsafe {
            gl.BufferData(
                B::GL_BUFTYPE,
                std::mem::size_of_val(data) as isize,
                data.as_ptr() as *const c_void,
                opengl::STATIC_DRAW,
            );
        }
        checkerr!(gl);
    }

    /// Destroys this buffer
    pub(crate) fn destroy(mut self, gl: &Gl) {
        if let Some(handle) = self.handle.take() {
            let as_int = handle.get();

            unsafe {
                gl.DeleteBuffers(1, &as_int);
            }
            checkerr!(gl);
        }
    }
}

impl<B> Drop for GlBuffer<B> {
    fn drop(&mut self) {
        if cfg!(debug_assertions) && self.handle.is_some() {
            log::warn!(
                "GL buffer of type {} dropped without being destroyed!",
                std::any::type_name::<B>()
            );
        }
    }
}
