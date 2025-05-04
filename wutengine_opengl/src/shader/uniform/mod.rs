use std::collections::HashMap;

use wutengine_graphics::material::MaterialParameter;
use wutengine_graphics::shader::uniform::UniformType;

use crate::opengl::types::{GLboolean, GLfloat, GLint, GLintptr, GLsizeiptr, GLuint};

pub(crate) mod const_blocks;
pub(crate) mod discovery;
pub(crate) mod std140;

/// The description of a single OpenGL shader uniform
#[derive(Debug, Clone)]
pub(crate) enum GlShaderUniform {
    Sampler {
        location: GLint,
        binding: usize,
    },
    Block {
        index: GLuint,
        binding: usize,
        size_bytes: usize,
        ty: UniformType,
    },
}

impl GlShaderUniform {
    pub(crate) const fn get_binding(&self) -> usize {
        match self {
            GlShaderUniform::Sampler { binding, .. } => *binding,
            GlShaderUniform::Block { binding, .. } => *binding,
        }
    }
}
