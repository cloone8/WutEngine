//! The WutEngine shader cross-compilation library.
//! Compiles shaders from the raw source (WGSL with extra keywords)
//! into platform specific code.

use cross::{CrossCompileErr, do_cross_compile};
use thiserror::Error;
use wutengine_graphics::shader::{Shader, ShaderTarget};

mod cross;

/// An error while compiling a shader
#[derive(Debug, Error)]
pub enum CompileErr {
    #[error("Error while cross compiling: {0}")]
    Cross(#[from] CrossCompileErr),
}

/// Compile options
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// Whether to cross compile the shader from a raw shader to the given target
    pub target: Option<ShaderTarget>,
}

pub fn compile(shader: &Shader, options: &Options) -> Result<Shader, CompileErr> {
    log::info!(
        "Starting compile job for shader {} with options {:#?}",
        shader.id,
        options
    );

    let mut output = shader.clone();

    if let Some(target) = options.target {
        do_cross_compile(&mut output, target)?;
    }

    Ok(output)
}
