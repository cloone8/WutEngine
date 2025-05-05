//! The WutEngine shader cross-compilation library.
//! Compiles shaders from the raw source (WGSL with extra keywords)
//! into platform specific code.

use std::collections::HashMap;

use cross::{CrossCompileErr, do_cross_compile};
use preprocess::{PreprocessErr, preprocess};
use thiserror::Error;
use wutengine_graphics::shader::{CompiledShader, RawShader, ShaderTarget, ShaderVariantId};

mod cross;
mod preprocess;

/// An error while compiling a shader
#[derive(Debug, Error)]
pub enum CompileErr {
    /// An error while cross compiling to the target platform
    #[error("Error while cross compiling: {0}")]
    Cross(#[from] CrossCompileErr),

    /// An error while preprocessing the raw shader source
    #[error("Error while preprocessing: {0}")]
    Preprocess(#[from] PreprocessErr),
}

/// Compiles `shader` into a native shader for the provided target, setting the
/// provided keyword values. Any keyword not given in `keywords` is set to 0
pub fn compile(
    shader: &RawShader,
    target: ShaderTarget,
    keywords: &HashMap<String, u32>,
) -> Result<CompiledShader, CompileErr> {
    log::info!(
        "Starting compile job for shader {} with target {} and keywords {:#?}",
        shader.ident,
        target,
        keywords
    );

    let mut working_shader = shader.clone();

    log::debug!("Compiling shader {:#?}", working_shader);

    preprocess(&mut working_shader, keywords)?;

    log::debug!("After preprocessing {:#?}", working_shader);

    let meta = do_cross_compile(&mut working_shader, target)?;

    log::debug!("After cross compiling {:#?}", working_shader);

    let compiled = CompiledShader {
        id: ShaderVariantId::new_with_keywords(shader.ident.clone(), keywords.clone()),
        target,
        target_meta: meta,
        source: working_shader.source,
        vertex_layout: working_shader.vertex_layout,
        builtins: working_shader.builtins,
        uniforms: working_shader.uniforms,
    };

    log::debug!("Final shader {:#?}", compiled);

    Ok(compiled)
}
