//! Texture functionality for the WutEngine OpenGL renderer

use tex2d::GlTexture2D;
use wutengine_graphics::image::{ColorType, DynamicImage};
use wutengine_graphics::texture::TextureData;

use crate::opengl::types::GLenum;
use crate::opengl::{self, Gl};

pub(crate) mod tex2d;

/// A generic wrapper over the different kinds of OpenGL textures
#[derive(Debug)]
pub(crate) enum GlTexture {
    /// A 2D texture
    Tex2D(GlTexture2D),
}

impl GlTexture {
    /// Destroys this texture
    pub(crate) fn destroy(self, gl: &Gl) {
        match self {
            GlTexture::Tex2D(tex2d) => tex2d.destroy(gl),
        }
    }

    /// Uploads the given data to this texture
    pub(crate) fn upload_data(&mut self, gl: &Gl, data: &TextureData) {
        match self {
            GlTexture::Tex2D(tex2d) => tex2d.upload_data(gl, data),
        }
    }
}

/// The OpenGL format for a [DynamicImage]. Get this with [determine_internal_format]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GlImageFormat {
    /// The desired closest matching internal texture format
    texture_internal_format: GLenum,

    /// The source image pixel format
    source_pixel_format: GLenum,

    /// The source image pixel per-component datatype
    source_pixel_data_type: GLenum,
}

/// Returns the format of the given image, as used in (for example)
/// the parameters of [Gl::TexImage2D]
fn determine_image_format(image: &DynamicImage) -> Option<GlImageFormat> {
    let (internal, pixel_fmt, pixel_data) = match image.color() {
        ColorType::L8 => (opengl::R8, opengl::RED, opengl::UNSIGNED_BYTE),
        ColorType::La8 => todo!(),
        ColorType::Rgb8 => (opengl::RGB8, opengl::RGB, opengl::UNSIGNED_BYTE),
        ColorType::Rgba8 => (opengl::RGBA8, opengl::RGBA, opengl::UNSIGNED_BYTE),
        ColorType::L16 => (opengl::R16, opengl::RED, opengl::UNSIGNED_SHORT),
        ColorType::La16 => todo!(),
        ColorType::Rgb16 => (opengl::RGB16, opengl::RGB, opengl::UNSIGNED_SHORT),
        ColorType::Rgba16 => (opengl::RGBA16, opengl::RGBA, opengl::UNSIGNED_SHORT),
        ColorType::Rgb32F => (opengl::RGB32F, opengl::RGB, opengl::FLOAT),
        ColorType::Rgba32F => (opengl::RGBA32F, opengl::RGBA, opengl::FLOAT),
        _ => return None,
    };

    Some(GlImageFormat {
        texture_internal_format: internal,
        source_pixel_format: pixel_fmt,
        source_pixel_data_type: pixel_data,
    })
}
