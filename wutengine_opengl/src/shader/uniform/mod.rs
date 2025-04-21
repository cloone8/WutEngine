use std::collections::HashMap;

use wutengine_graphics::material::MaterialParameter;
use wutengine_graphics::shader::UniformType;

use crate::opengl::types::{GLboolean, GLfloat, GLint, GLintptr, GLsizeiptr, GLuint};

pub(crate) mod discovery;
pub(crate) mod std140;

/// The description of a single OpenGL shader uniform
#[derive(Debug, Clone)]
pub(crate) enum GlShaderUniform {
    Sampler {
        location: GLint,
    },
    Block {
        index: GLuint,
        desc: UniformBlockDescriptor,
    },
}

#[derive(Debug, Clone)]
pub(crate) enum UniformBlockDescriptor {
    Single {
        offset: GLintptr,
        bytes: usize,
    },
    Array {
        stride: usize,
        count: usize,
        element: Box<UniformBlockDescriptor>,
    },
    Struct(HashMap<String, UniformBlockDescriptor>),
}
