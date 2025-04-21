//! OpenGL types implemented as Rust types

mod matrix;
mod vector;

pub(crate) use matrix::*;
pub(crate) use vector::*;

use crate::opengl;
use crate::opengl::types::{GLenum, GLuint};

pub(crate) fn size_of_gl(ty: GLenum) -> usize {
    match ty {
        opengl::UNSIGNED_INT => size_of::<GLuint>(),
        opengl::FLOAT => size_of::<f32>(),
        opengl::FLOAT_VEC3 => size_of::<GlVec3f>(),
        opengl::FLOAT_VEC4 => size_of::<GlVec4f>(),
        opengl::FLOAT_MAT4 => size_of::<GlMat4f>(),
        _ => panic!("Unknown gl type: 0x{:X}", ty),
    }
}
