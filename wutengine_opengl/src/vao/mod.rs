//! OpenGL vertex array object functionality

use core::ffi::c_void;
use core::num::NonZero;

use thiserror::Error;
use wutengine_graphics::shader::ShaderVertexLayout;

use crate::debug::{self, DebugObjType};
use crate::error::checkerr;
use crate::mesh::MeshBufferLayout;
use crate::opengl::types::GLuint;
use crate::opengl::{self, Gl};

/// An OpenGL Vertex Array Object
#[derive(Debug)]
pub(crate) struct Vao {
    handle: Option<NonZero<GLuint>>,
    current_layout: Option<(MeshBufferLayout, ShaderVertexLayout)>,
}

/// An error while creating a [Vao]
#[derive(Debug, Error)]
pub(crate) enum CreateErr {
    /// OpenGL returned 0
    #[error("OpenGL returned 0")]
    Zero,
}

#[profiling::all_functions]
impl Vao {
    /// Creates a new, unbound VAO
    pub(crate) fn new(gl: &Gl) -> Result<Self, CreateErr> {
        let mut handle = 0;

        unsafe {
            gl.GenVertexArrays(1, &mut handle);
        }
        checkerr!(gl);

        let handle = NonZero::new(handle).ok_or(CreateErr::Zero)?;

        Ok(Self {
            handle: Some(handle),
            current_layout: None,
        })
    }

    #[inline(always)]
    pub(crate) fn set_debug_label<F, S>(&self, gl: &Gl, name_fn: F)
    where
        F: FnOnce() -> Option<S>,
        S: Into<Vec<u8>>,
    {
        debug::add_debug_label(gl, self.handle.unwrap(), DebugObjType::VertexArray, name_fn);
    }

    #[inline(always)]
    pub(crate) const fn handle(&self) -> NonZero<GLuint> {
        self.handle.expect("VAO already freed")
    }

    /// Checks if the currently configured layout
    /// matches the given mesh and shader layouts
    pub(crate) fn layout_matches(
        &self,
        mesh_layout: &MeshBufferLayout,
        shader_layout: &ShaderVertexLayout,
    ) -> bool {
        match &self.current_layout {
            Some((cur_mesh_layout, cur_shader_layout)) => {
                cur_mesh_layout == mesh_layout && cur_shader_layout == shader_layout
            }
            None => false,
        }
    }

    /// Sets the given layout and associates with the given mesh buffers.
    /// Binds and unbinds this VAO, so no buffer is bound after this call returns
    pub(crate) fn set_layout(
        &mut self,
        gl: &Gl,
        mesh_layout: &MeshBufferLayout,
        shader_layout: ShaderVertexLayout,
    ) {
        log::trace!("Setting VAO layout");

        let mesh_vtx_stride = mesh_layout.calculate_stride_for_layout();

        log::trace!("Mesh layout: {:#?}", mesh_layout);
        log::trace!("Mesh layout stride: {}", mesh_vtx_stride);
        log::trace!("Shader layout: {:#?}", shader_layout);

        if let (Some(shader_attr_pos), Some(mesh_attr_pos)) =
            (shader_layout.position, mesh_layout.position)
        {
            unsafe {
                gl.VertexAttribPointer(
                    shader_attr_pos as GLuint,
                    3,
                    opengl::FLOAT,
                    opengl::FALSE,
                    mesh_vtx_stride,
                    mesh_attr_pos as *const c_void,
                );
                checkerr!(gl);

                gl.EnableVertexAttribArray(shader_attr_pos as GLuint);
                checkerr!(gl);
            }
        }

        if let (Some(shader_attr_uv), Some(mesh_attr_uv)) = (shader_layout.uv, mesh_layout.uv) {
            unsafe {
                gl.VertexAttribPointer(
                    shader_attr_uv as GLuint,
                    2,
                    opengl::FLOAT,
                    opengl::FALSE,
                    mesh_vtx_stride,
                    mesh_attr_uv as *const c_void,
                );
                checkerr!(gl);

                gl.EnableVertexAttribArray(shader_attr_uv as GLuint);
                checkerr!(gl);
            }
        }

        self.current_layout = Some((mesh_layout.clone(), shader_layout));
    }

    /// Destroys this VAO
    pub(crate) fn destroy(mut self, gl: &Gl) {
        if let Some(handle) = self.handle.take() {
            let as_int = handle.get();

            unsafe {
                gl.DeleteVertexArrays(1, &as_int);
            }
            checkerr!(gl);
        }
    }
}

impl Drop for Vao {
    fn drop(&mut self) {
        if cfg!(debug_assertions) {
            if let Some(handle) = self.handle {
                log::warn!("VAO {} dropped without being destroyed!", handle);
            }
        }
    }
}
