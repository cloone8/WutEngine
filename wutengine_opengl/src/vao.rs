use core::num::NonZero;
use std::ptr::null;

use thiserror::Error;
use wutengine_graphics::mesh::MeshData;

use crate::gltypes::GlVertex;
use crate::opengl::types::GLuint;
use crate::opengl::{self, Gl};

#[derive(Debug)]
pub struct Vao {
    handle: Option<NonZero<GLuint>>,
}

#[derive(Debug, Error)]
pub enum CreateErr {
    #[error("OpenGL returned 0")]
    Zero,
}

impl Vao {
    pub fn new(gl: &Gl) -> Result<Self, CreateErr> {
        let mut handle = 0;

        unsafe {
            gl.GenVertexArrays(1, &mut handle);
        }

        let handle = NonZero::new(handle).ok_or(CreateErr::Zero)?;

        Ok(Self {
            handle: Some(handle),
        })
    }

    pub fn bind(&mut self, gl: &Gl) {
        unsafe {
            let handle_int = self.handle.unwrap().get();

            gl.BindVertexArray(handle_int);
        }
    }

    pub fn unbind(&mut self, gl: &Gl) {
        unsafe {
            gl.BindVertexArray(0);
        }
    }

    pub fn set_vertex_attrs_for(&mut self, gl: &Gl, mesh: &MeshData) {
        unsafe {
            gl.VertexAttribPointer(
                0,
                3,
                opengl::FLOAT,
                opengl::FALSE,
                size_of::<GlVertex>() as i32,
                null(),
            );
            gl.EnableVertexAttribArray(0);
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
impl Drop for Vao {
    fn drop(&mut self) {
        if self.handle.is_some() {
            log::warn!("VAO dropped without being destroyed!");
        }
    }
}
