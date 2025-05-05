//! Debug functionality

use core::ffi::c_void;
use core::fmt::Debug;
use core::num::NonZero;
use core::sync::atomic::{AtomicU32, Ordering};
use std::ffi::CString;

use crate::error::checkerr;
use crate::opengl::Gl;
use crate::opengl::types::{GLchar, GLenum, GLsizei, GLuint};
use crate::{extensions, opengl};

/// A callback function compatible with the OpenGL debug callback signature.
/// Formats the logs slightly and outputs them with the normal Log function
pub(crate) extern "system" fn opengl_log_callback(
    source: GLenum,
    gltype: GLenum,
    _id: GLuint,
    severity: GLenum,
    length: GLsizei,
    message: *const GLchar,
    user_param: *mut c_void,
) {
    let window_id_string = user_param as *mut String;

    let window_id = unsafe { window_id_string.as_ref().unwrap().as_str() };

    let level = match (gltype, severity) {
        (opengl::DEBUG_TYPE_ERROR, _) => log::Level::Error,
        (_, opengl::DEBUG_SEVERITY_HIGH) => log::Level::Warn,
        (_, opengl::DEBUG_SEVERITY_MEDIUM) => log::Level::Info,
        (_, opengl::DEBUG_SEVERITY_LOW) => log::Level::Debug,
        (_, opengl::DEBUG_SEVERITY_NOTIFICATION) => log::Level::Trace,
        (_, _) => log::Level::Info,
    };

    let source = match source {
        opengl::DEBUG_SOURCE_API => "API",
        opengl::DEBUG_SOURCE_WINDOW_SYSTEM => "Window System",
        opengl::DEBUG_SOURCE_SHADER_COMPILER => "Shader Compiler",
        opengl::DEBUG_SOURCE_THIRD_PARTY => "Third Party",
        opengl::DEBUG_SOURCE_APPLICATION => "Application",
        opengl::DEBUG_SOURCE_OTHER => "Other",
        _ => "<unknown>",
    };

    let log_type = opengl_debug_log_type(gltype);

    let msg = if length != 0 {
        std::str::from_utf8(unsafe {
            std::slice::from_raw_parts(message as *const u8, length as usize)
        })
        .expect("Invalid UTF8 in log string")
    } else {
        ""
    };

    log::log!(
        level,
        "OpenGL({} {}) <{}>: {}",
        window_id,
        source,
        log_type,
        msg
    );
}

const fn opengl_debug_log_type(t: GLenum) -> &'static str {
    match t {
        opengl::DEBUG_TYPE_ERROR => "error",
        opengl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "deprecated",
        opengl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "undefined behaviour",
        opengl::DEBUG_TYPE_PORTABILITY => "portability",
        opengl::DEBUG_TYPE_PERFORMANCE => "performance",
        opengl::DEBUG_TYPE_OTHER => "other",
        opengl::DEBUG_TYPE_MARKER => "marker",
        opengl::DEBUG_TYPE_PUSH_GROUP => "marker_group_push",
        opengl::DEBUG_TYPE_POP_GROUP => "marker_group_pop",
        _ => "unknown",
    }
}

/// A debug object type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DebugObjType {
    /// Buffer object
    Buffer,

    /// Shader object (not shader program)
    #[expect(unused, reason = "later")]
    Shader,

    /// Shader program object
    ShaderProgram,

    /// Vertex array object
    VertexArray,

    /// Texture object
    Texture,

    /// Renderbuffer object
    #[expect(unused, reason = "later")]
    RenderBuffer,

    /// Framebuffer object
    #[expect(unused, reason = "later")]
    FrameBuffer,
}

impl DebugObjType {
    const fn to_khr_debug(self) -> GLenum {
        match self {
            DebugObjType::Buffer => opengl::BUFFER,
            DebugObjType::Shader => opengl::SHADER,
            DebugObjType::ShaderProgram => opengl::PROGRAM,
            DebugObjType::VertexArray => opengl::VERTEX_ARRAY,
            DebugObjType::Texture => opengl::TEXTURE,
            DebugObjType::RenderBuffer => opengl::RENDERBUFFER,
            DebugObjType::FrameBuffer => opengl::FRAMEBUFFER,
        }
    }

    const fn to_ext_label(self) -> GLenum {
        match self {
            DebugObjType::Buffer => opengl::BUFFER_OBJECT_EXT,
            DebugObjType::Shader => opengl::SHADER_OBJECT_EXT,
            DebugObjType::ShaderProgram => opengl::PROGRAM_OBJECT_EXT,
            DebugObjType::VertexArray => opengl::VERTEX_ARRAY_OBJECT_EXT,
            DebugObjType::Texture => opengl::TEXTURE,
            DebugObjType::RenderBuffer => opengl::RENDERBUFFER,
            DebugObjType::FrameBuffer => opengl::FRAMEBUFFER,
        }
    }
}

/// Adds an OpenGL debug label to the given object, if we are compiling
/// with debug assertions in a context with the debug label extension
#[inline(always)]
pub(crate) fn add_debug_label<F, S>(
    gl: &Gl,
    obj: NonZero<GLuint>,
    objtype: DebugObjType,
    name_fn: F,
) where
    F: FnOnce() -> Option<S>,
    S: Into<Vec<u8>>,
{
    if cfg!(not(debug_assertions)) {
        // Useless work for release builds
        return;
    }

    let (can_use_khr, can_use_ext_label) = unsafe {
        let extensions = extensions::get_global().read();
        (
            extensions.khr_debug && gl.ObjectLabel.is_loaded(),
            extensions.ext_debug_label && gl.LabelObjectEXT.is_loaded(),
        )
    };

    if !can_use_khr && !can_use_ext_label {
        // Labelling objects is not supported in this context
        return;
    }

    let name = name_fn();

    if name.is_none() {
        return;
    }

    let as_c = CString::new(name.unwrap());

    if as_c.is_err() {
        log::warn!("Could not add OpenGL debug label because it is not valid ascii");
        return;
    }

    let as_c = as_c.unwrap();

    unsafe {
        if can_use_khr {
            gl.ObjectLabel(
                objtype.to_khr_debug(),
                obj.get(),
                as_c.count_bytes() as GLsizei,
                as_c.as_ptr(),
            );
        } else if can_use_ext_label {
            gl.LabelObjectEXT(
                objtype.to_ext_label(),
                obj.get(),
                as_c.count_bytes() as GLsizei,
                as_c.as_ptr(),
            );
        }
    }

    checkerr!(gl);
}

/// Emits an OpenGL debug marker, if we are compiling
/// with debug assertions in a context with the debug marker extension
#[inline(always)]
#[expect(unused, reason = "later")]
pub(crate) fn debug_event_marker<F, S>(gl: &Gl, name_fn: F)
where
    F: FnOnce() -> Option<S>,
    S: Into<Vec<u8>>,
{
    if cfg!(not(debug_assertions)) {
        // Useless work for release builds
        return;
    }

    let (can_use_khr, can_use_ext_marker) = unsafe {
        let extensions = extensions::get_global().read();
        (
            extensions.khr_debug && gl.DebugMessageInsert.is_loaded(),
            extensions.ext_debug_marker && gl.InsertEventMarkerEXT.is_loaded(),
        )
    };

    if !can_use_khr && !can_use_ext_marker {
        // Event markers are not supported in this context
        return;
    }

    let name = name_fn();

    if name.is_none() {
        return;
    }

    let as_c = CString::new(name.unwrap());

    if as_c.is_err() {
        log::warn!("Could not add OpenGL event marker because it is not valid ascii");
        return;
    }

    let as_c = as_c.unwrap();

    unsafe {
        if can_use_khr {
            static MSG_ID: AtomicU32 = AtomicU32::new(0);

            gl.DebugMessageInsert(
                opengl::DEBUG_SOURCE_APPLICATION,
                opengl::DEBUG_TYPE_MARKER,
                MSG_ID.fetch_add(1, Ordering::Relaxed) as GLuint,
                opengl::DEBUG_SEVERITY_NOTIFICATION,
                as_c.count_bytes() as GLsizei,
                as_c.as_ptr(),
            );
        } else if can_use_ext_marker {
            gl.InsertEventMarkerEXT(as_c.count_bytes() as GLsizei, as_c.as_ptr());
        }
    }

    checkerr!(gl);
}

/// An OpenGL debug marker group, if we are compiling
/// with debug assertions in a context with the debug marker extension.
/// Create with [Self::new]. The group is automatically popped when the struct runs
/// its [Drop] implementation
pub(crate) struct GlDebugMarkerGroup<'gl> {
    #[cfg(debug_assertions)]
    gl: &'gl Gl,
    #[cfg(debug_assertions)]
    was_pushed: DebugMarkerPusher,

    #[cfg(not(debug_assertions))]
    _ph: core::marker::PhantomData<&'gl Gl>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg(debug_assertions)]
enum DebugMarkerPusher {
    None,
    KhrDebug,
    ExtDebugMarker,
}

/// Pushes a new auto-popping OpenGL debug marker group in debug builds
#[inline(always)]
pub(crate) fn debug_marker_group<F, S>(gl: &Gl, name_fn: F) -> GlDebugMarkerGroup
where
    F: FnOnce() -> S,
    S: Into<Vec<u8>>,
{
    GlDebugMarkerGroup::new(gl, name_fn)
}

impl<'gl> GlDebugMarkerGroup<'gl> {
    /// Returns a new OpenGL marker group struct.
    /// Automatically pops the debug group when dropped.
    /// Only active in release builds
    #[cfg(not(debug_assertions))]
    #[inline(always)]
    pub(crate) fn new<F, S>(_gl: &'gl Gl, _name_fn: F) -> Self
    where
        F: FnOnce() -> S,
        S: Into<Vec<u8>>,
    {
        Self {
            _ph: core::marker::PhantomData,
        }
    }

    /// Returns a new OpenGL marker group struct.
    /// Automatically pops the debug group when dropped.
    /// Only active in release builds
    #[cfg(debug_assertions)]
    #[inline(always)]
    pub(crate) fn new<F, S>(gl: &'gl Gl, name_fn: F) -> Self
    where
        F: FnOnce() -> S,
        S: Into<Vec<u8>>,
    {
        let mut new = Self {
            gl,
            was_pushed: DebugMarkerPusher::None,
        };

        let (can_use_khr, can_use_ext_marker) = unsafe {
            let extensions = extensions::get_global().read();
            (
                extensions.khr_debug && gl.PushDebugGroup.is_loaded(),
                extensions.ext_debug_marker && gl.PushGroupMarkerEXT.is_loaded(),
            )
        };

        if !can_use_khr && !can_use_ext_marker {
            // Event markers are not supported in this context
            return new;
        }

        let name = name_fn();
        let as_c = CString::new(name);

        if as_c.is_err() {
            log::warn!("Could not add OpenGL group marker  because it is not valid ascii",);
            return new;
        }

        let as_c = as_c.unwrap();

        unsafe {
            if can_use_khr {
                static DEBUG_GROUP_ID: AtomicU32 = AtomicU32::new(0);
                gl.PushDebugGroup(
                    opengl::DEBUG_SOURCE_APPLICATION,
                    DEBUG_GROUP_ID.fetch_add(1, Ordering::Relaxed),
                    as_c.count_bytes() as GLsizei,
                    as_c.as_ptr(),
                );
                new.was_pushed = DebugMarkerPusher::KhrDebug;
            } else if can_use_ext_marker {
                gl.PushGroupMarkerEXT(as_c.count_bytes() as GLsizei, as_c.as_ptr());
                new.was_pushed = DebugMarkerPusher::ExtDebugMarker;
            }
        }

        checkerr!(gl);

        new
    }
}

impl Drop for GlDebugMarkerGroup<'_> {
    #[inline(always)]
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        unsafe {
            match self.was_pushed {
                DebugMarkerPusher::None => (),
                DebugMarkerPusher::KhrDebug => self.gl.PopDebugGroup(),
                DebugMarkerPusher::ExtDebugMarker => self.gl.PopGroupMarkerEXT(),
            }
        }
    }
}
