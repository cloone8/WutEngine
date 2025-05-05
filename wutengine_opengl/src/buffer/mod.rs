//! Abstractions around OpenGL buffers

use core::num::NonZero;

use thiserror::Error;

use crate::debug;
use crate::error::checkerr;
use crate::opengl::Gl;
use crate::opengl::types::GLuint;

/// A generic OpenGL buffer
#[derive(Debug)]
#[repr(transparent)]
pub(crate) struct GlBuffer {
    handle: Option<NonZero<GLuint>>,
}

/// Error while creating a generic OpenGL buffer
#[derive(Debug, Clone, Copy, Error)]
pub(crate) enum CreateErr {
    /// Zero was returned
    #[error("OpenGL returned 0")]
    Zero,
}

#[profiling::all_functions]
impl GlBuffer {
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
        })
    }

    #[inline(always)]
    pub(crate) fn set_debug_label<F, S>(&self, gl: &Gl, name_fn: F)
    where
        F: FnOnce() -> Option<S>,
        S: Into<Vec<u8>>,
    {
        debug::add_debug_label(
            gl,
            self.handle.unwrap(),
            debug::DebugObjType::Buffer,
            name_fn,
        );
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

    #[inline(always)]
    pub(crate) const fn handle(&self) -> NonZero<GLuint> {
        self.handle.expect("GlBuffer already freed")
    }
}

impl Drop for GlBuffer {
    fn drop(&mut self) {
        if cfg!(debug_assertions) {
            if let Some(handle) = self.handle {
                log::warn!(
                    "GL buffer with handle {} dropped without being destroyed!",
                    handle
                );
            }
        }
    }
}
