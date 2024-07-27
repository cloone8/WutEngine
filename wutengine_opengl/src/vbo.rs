use core::{ffi::c_void, num::NonZero};

use thiserror::Error;

use crate::gltypes::GlVertex;
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

    pub fn bind(&mut self, gl: &Gl) {
        unsafe {
            let handle_int = self.handle.unwrap().get();

            gl.BindBuffer(opengl::ARRAY_BUFFER, handle_int);
        }
    }

    pub fn unbind(&mut self, gl: &Gl) {
        unsafe {
            gl.BindBuffer(opengl::ARRAY_BUFFER, 0);
        }
    }

    pub fn buffer_data<T: Into<GlVertex> + Copy>(&mut self, gl: &Gl, data: &[T]) {
        let gl_verts: Vec<GlVertex> = data.iter().copied().map(|v| v.into()).collect();

        unsafe {
            gl.BufferData(
                opengl::ARRAY_BUFFER,
                (gl_verts.len() * size_of::<GlVertex>()) as isize,
                gl_verts.as_ptr() as *const c_void,
                opengl::STATIC_DRAW,
            );
        }
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
