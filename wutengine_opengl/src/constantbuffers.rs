//! Buffers for the WutEngine shader constant blocks

use core::ptr::null;

use crate::buffer::{CreateErr, GlBuffer};
use crate::error::checkerr;
use crate::opengl::types::GLsizeiptr;
use crate::opengl::{self, Gl};
use crate::shader::uniform::const_blocks::{
    WutEngineInstanceConstants, WutEngineViewportConstants,
};

/// Buffers for the constants used in most shaders
#[derive(Debug)]
pub(crate) struct ConstantBuffers {
    /// Per-viewport constant buffer
    pub(crate) viewport_constants: GlBuffer,

    /// Per-instance constant buffer
    pub(crate) instance_constants: GlBuffer,
}

impl ConstantBuffers {
    /// Constructs the constant buffers and allocates their GPU memory
    pub(crate) fn new(gl: &Gl) -> Result<Self, CreateErr> {
        let vpc = GlBuffer::new(gl)?;
        let ic = match GlBuffer::new(gl) {
            Ok(ic) => ic,
            Err(e) => {
                vpc.destroy(gl);
                return Err(e);
            }
        };

        unsafe {
            gl.BindBuffer(opengl::UNIFORM_BUFFER, vpc.handle().get());
            gl.BufferData(
                opengl::UNIFORM_BUFFER,
                size_of::<WutEngineViewportConstants>() as GLsizeiptr,
                null(),
                opengl::DYNAMIC_DRAW,
            );

            checkerr!(gl);

            gl.BindBuffer(opengl::UNIFORM_BUFFER, ic.handle().get());
            gl.BufferData(
                opengl::UNIFORM_BUFFER,
                size_of::<WutEngineInstanceConstants>() as GLsizeiptr,
                null(),
                opengl::DYNAMIC_DRAW,
            );

            checkerr!(gl);

            gl.BindBuffer(opengl::UNIFORM_BUFFER, 0);
        }

        vpc.set_debug_label(gl, || Some("viewport_constants"));
        ic.set_debug_label(gl, || Some("instance_constants"));

        Ok(Self {
            viewport_constants: vpc,
            instance_constants: ic,
        })
    }

    /// Destroys the constant buffers
    pub(crate) fn destroy(self, gl: &Gl) {
        self.viewport_constants.destroy(gl);
        self.instance_constants.destroy(gl);
    }
}
