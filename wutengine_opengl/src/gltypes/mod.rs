//! OpenGL types implemented as Rust types

pub(crate) mod array;
mod matrix;
mod vector;

use core::ffi::c_void;

pub(crate) use matrix::*;
pub(crate) use vector::*;

use crate::opengl;
use crate::opengl::types::{GLenum, GLsizeiptr, GLuint};

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

pub(crate) trait GlVec {
    fn size_bytes(&self) -> GLsizeiptr;
    fn void_ptr(&self) -> *const c_void;
}

impl<T> GlVec for [T] {
    #[inline(always)]
    fn size_bytes(&self) -> GLsizeiptr {
        (self.len() * size_of::<T>()) as GLsizeiptr
    }

    #[inline(always)]
    fn void_ptr(&self) -> *const c_void {
        self.as_ptr() as *const c_void
    }
}
