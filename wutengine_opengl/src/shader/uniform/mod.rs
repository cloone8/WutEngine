//! Shader uniforms and their handling

use wutengine_graphics::shader::uniform::UniformType;

use crate::opengl::types::GLint;

pub(crate) mod const_blocks;
pub(crate) mod discovery;
pub(crate) mod std140;

/// The description of a single OpenGL shader uniform, and
/// how to bind to it
#[derive(Debug, Clone)]
pub(crate) enum GlShaderUniform {
    /// A sampler uniform. Returns its location, which can be used by [crate::opengl::Gl::Uniform1i]
    Sampler {
        /// The location in the shader
        location: GLint,
    },

    /// An interface block uniform. Contains its binding point and the datatype
    Block {
        /// The binding point of the block
        binding: usize,

        /// The data type of the block
        ty: UniformType,
    },
}
