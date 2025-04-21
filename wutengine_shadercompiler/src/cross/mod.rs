//! Cross compilation module

use opengl::{CrossOpenGLErr, cross_to_opengl};
use thiserror::Error;
use wutengine_graphics::shader::{Shader, ShaderTarget};

pub(crate) mod opengl;

#[derive(Debug, Error)]
pub enum CrossCompileErr {
    /// Invalid cross-compile source
    #[error("Invalid cross-compile source: {0}. Must be raw")]
    CrossCompileSource(ShaderTarget),

    /// Invalid cross-compile target
    #[error("Invalid cross-compile target: {0}")]
    CrossCompileTarget(ShaderTarget),

    /// OpenGL cross-compile specific error
    #[error("Error cross compiling to OpenGL: {0}")]
    OpenGL(#[from] CrossOpenGLErr),
}

pub(crate) fn do_cross_compile(
    shader: &mut Shader,
    target: ShaderTarget,
) -> Result<(), CrossCompileErr> {
    log::info!(
        "Starting cross compile for shader {} to target {}",
        shader.id,
        target
    );

    if shader.target != ShaderTarget::Raw {
        return Err(CrossCompileErr::CrossCompileSource(shader.target));
    }

    match target {
        ShaderTarget::Raw => return Err(CrossCompileErr::CrossCompileTarget(target)),
        ShaderTarget::OpenGL => cross_to_opengl(shader)?,
    };

    Ok(())
}
