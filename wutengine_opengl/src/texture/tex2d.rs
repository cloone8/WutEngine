//! OpenGL 2D texture

use core::ffi::c_void;
use core::num::NonZero;

use thiserror::Error;
use wutengine_graphics::texture::TextureData;

use crate::error::checkerr;
use crate::opengl::types::{GLint, GLsizei, GLuint};
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
            //TODO: Make this configurable through the generic texture data
            gl.TexParameteri(
                opengl::TEXTURE_2D,
                opengl::TEXTURE_WRAP_S,
                opengl::REPEAT as GLint,
            );
            checkerr!(gl);

            gl.TexParameteri(
                opengl::TEXTURE_2D,
                opengl::TEXTURE_WRAP_T,
                opengl::REPEAT as GLint,
            );
            checkerr!(gl);

            gl.TexParameteri(
                opengl::TEXTURE_2D,
                opengl::TEXTURE_MIN_FILTER,
                opengl::LINEAR_MIPMAP_LINEAR as GLint,
            );
            checkerr!(gl);

            gl.TexParameteri(
                opengl::TEXTURE_2D,
                opengl::TEXTURE_MAG_FILTER,
                opengl::LINEAR as GLint,
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
