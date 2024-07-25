use core::{ffi::c_void, num::NonZero};

use thiserror::Error;

use crate::opengl::{self, types::GLuint, Gl};

//TODO: Type-state? Vbo<Unbound> -> Vbo<Bound> etc.
#[derive(Debug)]
pub struct Vbo {
    handle: Option<NonZero<GLuint>>,
}

#[derive(Debug, Clone, Copy, Error)]
pub enum CreateErr {
    #[error("OpenGL returned 0")]
    Zero,
}

impl Vbo {
    pub fn new(gl: &Gl) -> Result<Self, CreateErr> {
        let mut handle = 0;

        unsafe {
            gl.GenBuffers(1, &mut handle);
        }

        let handle = NonZero::new(handle).ok_or(CreateErr::Zero)?;

        Ok(Self {
            handle: Some(handle),
        })
    }

    fn bind(&mut self, gl: &Gl) {
        unsafe {
            let handle_int = self.handle.unwrap().get();

            gl.BindBuffer(opengl::ARRAY_BUFFER, handle_int);
        }
    }

    fn unbind(&mut self, gl: &Gl) {
        unsafe {
            gl.BindBuffer(opengl::ARRAY_BUFFER, 0);
        }
    }

    pub fn buffer_data<T>(&mut self, gl: &Gl, data: &[T]) {
        self.bind(gl);

        unsafe {
            gl.BufferData(
                opengl::ARRAY_BUFFER,
                std::mem::size_of_val(data) as isize,
                data.as_ptr() as *const c_void,
                opengl::STATIC_DRAW,
            );
        }

        self.unbind(gl);
    }

    pub fn destroy(mut self, gl: &Gl) {
        if let Some(handle) = self.handle.take() {
            let as_int = handle.get();

            unsafe {
                gl.DeleteBuffers(1, &as_int);
            }
        }
    }
}

#[cfg(debug_assertions)]
impl Drop for Vbo {
    fn drop(&mut self) {
        if self.handle.is_some() {
            log::warn!("VBO dropped without being destroyed!");
        }
    }
}
