//! OpenGL extension detection and handling

use core::ffi::{CStr, c_char};

use crate::opengl::types::GLuint;
use crate::opengl::{self, Gl};

/// A struct containing some of the extensions
/// the OpenGL backend uses, and flags signalling whether they're supported or not
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct GlExtensions {
    /// GL_KHR_debug
    pub(crate) khr_debug: bool,

    /// GL_EXT_debug_label
    pub(crate) ext_debug_label: bool,

    /// GL_EXT_debug_marker
    pub(crate) ext_debug_marker: bool,
}

impl GlExtensions {
    /// Creates a new extensions object by probing the given bindings
    pub(crate) fn new(gl: &Gl) -> Self {
        let mut exts = Self::new_empty();

        let mut num_extensions = 0;
        unsafe {
            gl.GetIntegerv(opengl::NUM_EXTENSIONS, &raw mut num_extensions);
        }

        for i in 0..num_extensions {
            unsafe {
                let ext_string = gl.GetStringi(opengl::EXTENSIONS, i as GLuint);

                let ext_string_c = CStr::from_ptr::<'static>(ext_string as *const c_char);

                if ext_string_c == c"GL_KHR_debug" {
                    exts.khr_debug = true;
                } else if ext_string_c == c"GL_EXT_debug_label" {
                    exts.ext_debug_label = true;
                } else if ext_string_c == c"GL_EXT_debug_marker" {
                    exts.ext_debug_marker = true;
                }
            }
        }

        exts
    }

    const fn new_empty() -> Self {
        Self {
            khr_debug: false,
            ext_debug_label: false,
            ext_debug_marker: false,
        }
    }
}

static mut EXTENSIONS: GlExtensions = GlExtensions::new_empty();

/// Sets the global set of extensions. Not thread safe. Assumes only
/// a single OpenGL context is active at a time
pub(crate) unsafe fn set_global(extensions: GlExtensions) {
    unsafe {
        (&raw mut EXTENSIONS).write(extensions);
    }
}

/// Returns the global set of extensions
pub(crate) fn get_global() -> *const GlExtensions {
    &raw const EXTENSIONS
}
