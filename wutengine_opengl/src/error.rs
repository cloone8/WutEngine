//! Error checking routines for OpenGL calls

/// Expands to a loop that checks for any/all OpenGL errors for as long as
/// they are returned.
macro_rules! checkerr {
    ($gl:expr) => {
        $crate::error::check_err_impl($gl, file!(), line!());
    };
}

pub(crate) use checkerr;

use crate::opengl::Gl;

/// Checks for any unread OpenGL errors and reports them with the provided file and line.
/// Reads up to 1000 errors to prevent infinite loops
#[cfg(debug_assertions)]
#[inline]
pub(crate) fn check_err_impl(gl: &Gl, file: &str, line: u32) {
    profiling::function_scope!();
    let mut err_count: usize = 0;

    loop {
        profiling::scope!("Check Single Error");
        let err = unsafe { gl.GetError() };

        if err == crate::opengl::NO_ERROR {
            break;
        }

        if err_count > 1000 {
            // Small safeguard
            panic!("More than 1000 OpenGL error read, quitting");
        }

        log::error!(
            "OpenGL error @ {}:{}: {:#X} {}",
            file,
            line,
            err,
            gl_err_to_str(err)
        );

        err_count += 1;
    }

    if err_count != 0 {
        panic!("Encountered OpenGL error.");
    }
}

#[cfg(not(debug_assertions))]
#[inline(always)]
pub(crate) fn check_err_impl(_gl: &Gl, _file: &str, _line: u32) {}

/// Converts an OpenGL error enum to a string, if it is a known error enum. Returns "(unknown)" otherwise.
#[allow(dead_code)]
pub(crate) const fn gl_err_to_str(err: crate::opengl::types::GLenum) -> &'static str {
    match err {
        crate::opengl::INVALID_ENUM => "Invalid Enum",
        crate::opengl::INVALID_VALUE => "Invalid Value",
        crate::opengl::INVALID_OPERATION => "Invalid Operation",
        crate::opengl::OUT_OF_MEMORY => "Out Of Memory",
        crate::opengl::INVALID_FRAMEBUFFER_OPERATION => "Invalid Framebuffer Operation",
        _ => "(unknown)",
    }
}
