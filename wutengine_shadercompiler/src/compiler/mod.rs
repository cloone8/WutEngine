//! Shader compilation module. Takes care of compiling preprocessed WGSL shaders into [naga] for WGPU

use naga::front::wgsl::ParseError;

use crate::ShaderOutput;

/// Compiles a [ShaderOutput::Preprocessed] shader to a [ShaderOutput::Compiled] shader by
/// compiling it to [naga] IR
#[profiling::function]
pub fn compile_to_naga_ir<'a>(shader: ShaderOutput<'a>) -> Result<ShaderOutput<'a>, ParseError> {
    if let ShaderOutput::Preprocessed {
        source,
        keyword_hash,
        keywords,
    } = shader
    {
        Ok(ShaderOutput::Compiled {
            source: Box::new(naga::front::wgsl::parse_str(&source)?),
            keyword_hash,
            keywords,
        })
    } else {
        panic!("Got non-preprocessed shader");
    }
}
