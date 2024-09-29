//! Error checking routines for OpenGL calls

/// Expands to a loop that checks for any/all OpenGL errors for as long as
/// they are returned.
macro_rules! check_gl_err {
    ($gl:expr) => {
        if cfg!(debug_assertions) {
            loop {
                #[allow(unused_unsafe)]
                let __err = unsafe { $gl.GetError() };

                if __err == crate::opengl::NO_ERROR {
                    break;
                }

                log::error!(
                    "OpenGL error @ {}:{}: {:#X} {}",
                    file!(),
                    line!(),
                    __err,
                    crate::error::gl_err_to_str(__err)
                );
            }
        }
    };
}

pub(crate) use check_gl_err;

/// Converts an OpenGL error enum to a string, if it is a known error enum. Returns "(unknown)" otherwise.
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
