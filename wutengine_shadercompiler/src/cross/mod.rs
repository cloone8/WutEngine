//! Cross compilation module

use opengl::{CrossOpenGLErr, cross_to_opengl};
use thiserror::Error;
use wutengine_graphics::shader::{RawShader, ShaderTarget, ShaderTargetMeta};

pub(crate) mod opengl;

#[derive(Debug, Error)]
pub enum CrossCompileErr {
    /// Invalid cross-compile target
    #[error("Invalid cross-compile target: {0}")]
    CrossCompileTarget(ShaderTarget),

    /// OpenGL cross-compile specific error
    #[error("Error cross compiling to OpenGL: {0}")]
    OpenGL(#[from] CrossOpenGLErr),
}

pub(crate) fn do_cross_compile(
    shader: &mut RawShader,
    target: ShaderTarget,
) -> Result<ShaderTargetMeta, CrossCompileErr> {
    log::info!(
        "Starting cross compile for shader {} to target {}",
        shader.ident,
        target
    );

    let meta = match target {
        ShaderTarget::OpenGL => ShaderTargetMeta::OpenGL(cross_to_opengl(shader)?),
    };

    Ok(meta)
}
