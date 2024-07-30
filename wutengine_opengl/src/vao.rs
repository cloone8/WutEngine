use core::num::NonZero;

use thiserror::Error;

use crate::gltypes::GlMeshBuffers;
use crate::opengl::types::GLuint;
use crate::opengl::{self, Gl};
use crate::shader::program::ShaderProgram;

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

    pub fn set_vertex_attrs_for(&mut self, gl: &Gl, mesh: &GlMeshBuffers, program: &ShaderProgram) {
        for attribute in mesh.layout.get_present_attributes() {
            log::trace!("Checking attribute presence: {:?}", attribute);

            let location_index = unsafe {
                gl.GetAttribLocation(program.assert_linked().get(), attribute.as_c_str().as_ptr())
            };

            if location_index == -1 {
                log::trace!("Attribute not present");
                // Attribute not present on shader
                continue;
            }

            log::trace!("Attribute present at {}", location_index);

            let layout = mesh.layout.get_for_attribute(attribute).unwrap();

            log::trace!("Resolved layout: {:#?}", layout);

            unsafe {
                gl.VertexAttribPointer(
                    location_index as GLuint,
                    layout.size,
                    layout.gltype,
                    opengl::FALSE,
                    layout.stride,
                    layout.offset,
                );
                gl.EnableVertexAttribArray(location_index as GLuint);
            }
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
