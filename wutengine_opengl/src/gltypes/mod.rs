//! OpenGL types implemented as Rust types

pub(crate) mod array;
mod matrix;
mod vector;

use core::ffi::c_void;

pub(crate) use matrix::*;
pub(crate) use vector::*;

use crate::opengl::types::GLsizeiptr;

/// Convenience trait that can be implemented for types that
/// can return their size in bytes and pointers in an OpenGL API compatible
/// format
pub(crate) trait GlVec {
    /// Returns the size in bytes of the object
    /// as an OpenGL sizeiptr
    fn size_bytes(&self) -> GLsizeiptr;

    /// Returns a raw void pointer to the type
    fn void_ptr(&self) -> *const c_void;
}

impl<T> GlVec for [T] {
    #[inline(always)]
    fn size_bytes(&self) -> GLsizeiptr {
        std::mem::size_of_val(self) as GLsizeiptr
    }

    #[inline(always)]
    fn void_ptr(&self) -> *const c_void {
        self.as_ptr() as *const c_void
    }
}
