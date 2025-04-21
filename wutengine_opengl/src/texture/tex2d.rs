//! OpenGL 2D texture

use core::ffi::c_void;
use core::num::NonZero;

use thiserror::Error;
use wutengine_graphics::texture::{TextureData, TextureFiltering, TextureWrapping, WrappingMethod};

use crate::error::checkerr;
use crate::opengl::types::{GLenum, GLint, GLsizei, GLuint};
use crate::opengl::{self, Gl};

use super::determine_image_format;

/// An OpenGL texture
#[derive(Debug)]
pub(crate) struct GlTexture2D {
    handle: Option<NonZero<GLuint>>,
}

/// An error while creating a [GlTexture]
#[derive(Debug, Error)]
pub(crate) enum CreateErr {
    /// Zero returned
    #[error("OpenGL returned a zero buffer")]
    Zero,
}

#[profiling::all_functions]
impl GlTexture2D {
    /// Creates a new uninitialized OpenGL texture
    pub(crate) fn new(gl: &Gl) -> Result<Self, CreateErr> {
        let mut handle: GLuint = 0;

        unsafe {
            gl.GenTextures(1, &raw mut handle);
        }

        checkerr!(gl);

        let handle = NonZero::new(handle).ok_or(CreateErr::Zero)?;

        Ok(Self {
            handle: Some(handle),
        })
    }

    /// Binds this texture
    pub(crate) fn bind(&mut self, gl: &Gl) {
        let handle = self.handle.expect("Texture already destroyed");

        unsafe {
            gl.BindTexture(opengl::TEXTURE_2D, handle.get());
        }
        checkerr!(gl);
    }

    /// Unbinds this texture
    pub(crate) fn unbind(&mut self, gl: &Gl) {
        assert!(self.handle.is_some(), "Texture already destroyed");

        unsafe {
            gl.BindTexture(opengl::TEXTURE_2D, 0);
        }
        checkerr!(gl);
    }

    /// Destroys this buffer
    pub(crate) fn destroy(mut self, gl: &Gl) {
        let handle = self.handle.take().expect("Texture already destroyed");
        let as_int = handle.get();

        unsafe {
            gl.DeleteTextures(1, &raw const as_int);
        }

        checkerr!(gl);
    }

    /// Uploads the given data to this texture. Automatically binds and unbinds the
    /// texture. After this function returns, the texture will be _unbound_
    pub(crate) fn upload_data(&mut self, gl: &Gl, data: &TextureData) {
        log::trace!("Uploading texture data");

        // For consistency, just bind to the first texture unit
        unsafe {
            gl.ActiveTexture(opengl::TEXTURE0);
        }

        self.bind(gl);

        unsafe {
            let (wrap_s, wrap_t) = get_u_v_wrapping(data.wrapping);

            gl.TexParameteri(opengl::TEXTURE_2D, opengl::TEXTURE_WRAP_S, wrap_s as GLint);
            checkerr!(gl);

            gl.TexParameteri(opengl::TEXTURE_2D, opengl::TEXTURE_WRAP_T, wrap_t as GLint);
            checkerr!(gl);

            let (filter_min, filter_mag) = get_min_mag_filter(data.filtering);

            gl.TexParameteri(
                opengl::TEXTURE_2D,
                opengl::TEXTURE_MIN_FILTER,
                filter_min as GLint,
            );
            checkerr!(gl);

            gl.TexParameteri(
                opengl::TEXTURE_2D,
                opengl::TEXTURE_MAG_FILTER,
                filter_mag as GLint,
            );
            checkerr!(gl);

            // Now upload the actual data
            let img_fmt = determine_image_format(&data.imagedata).expect("Unknown color format");

            let width = GLsizei::try_from(data.imagedata.width()).expect("Image too wide");
            let height = GLsizei::try_from(data.imagedata.height()).expect("Image too high");

            log::trace!(
                "Source image size {}x{}, determined format: {:#?}",
                width,
                height,
                img_fmt
            );

            gl.TexImage2D(
                opengl::TEXTURE_2D,
                0,
                img_fmt.texture_internal_format as GLint,
                width,
                height,
                0,
                img_fmt.source_pixel_format,
                img_fmt.source_pixel_data_type,
                data.imagedata.as_bytes().as_ptr() as *const c_void,
            );

            checkerr!(gl);

            gl.GenerateMipmap(opengl::TEXTURE_2D);

            checkerr!(gl);
        }

        self.unbind(gl);

        log::trace!("Texture uploaded");
    }
}

impl Drop for GlTexture2D {
    fn drop(&mut self) {
        if cfg!(debug_assertions) {
            if let Some(handle) = self.handle {
                log::warn!("GL texture {} dropped without being destroyed!", handle);
            }
        }
    }
}

/// Converts a [TextureFiltering] struct to an opengl min/mag filter.
/// Returns `(min, mag)`
fn get_min_mag_filter(filter: TextureFiltering) -> (GLenum, GLenum) {
    match filter {
        TextureFiltering::Linear => (opengl::LINEAR_MIPMAP_LINEAR, opengl::LINEAR),
        TextureFiltering::Nearest => (opengl::NEAREST_MIPMAP_NEAREST, opengl::NEAREST),
    }
}

/// Converts a [TextureWrapping] struct to two opengl wrapping method enums, one per axis.
/// Returns (u, v)
const fn get_u_v_wrapping(wrapping: TextureWrapping) -> (GLenum, GLenum) {
    match wrapping {
        TextureWrapping::Both(wrapping_method) => {
            let mthd = wrapping_method_to_enum(wrapping_method);
            (mthd, mthd)
        }
        TextureWrapping::PerAxis { u, v } => {
            (wrapping_method_to_enum(u), wrapping_method_to_enum(v))
        }
    }
}

const fn wrapping_method_to_enum(method: WrappingMethod) -> GLenum {
    match method {
        WrappingMethod::Repeat => opengl::REPEAT,
        WrappingMethod::Mirror => opengl::MIRRORED_REPEAT,
        WrappingMethod::Clamp => opengl::CLAMP_TO_EDGE,
    }
}
