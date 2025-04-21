//! Module for the data of a WutEngine material. A material describes the way a given mesh is rendered

use std::collections::HashMap;

use glam::{Mat4, Vec4};
use image::{DynamicImage, ImageBuffer};

use crate::color::Color;
use crate::renderer::RendererTextureId;
use crate::shader::ShaderId;
use crate::texture::{TextureData, TextureFiltering, TextureWrapping, WrappingMethod};

/// The data of a material
#[derive(Debug, Clone, Default)]
pub struct MaterialData {
    /// The shader used to render this material. If [None], this material will not render
    pub shader: Option<ShaderId>,

    /// Any material parameters
    pub parameters: HashMap<String, MaterialParameter>,
}

/// The value of a material parameter in a [MaterialData]
#[derive(Debug, Clone)]
pub enum MaterialParameter {
    /// A boolean
    Boolean(bool),

    /// An array of booleans
    BooleanArray(Vec<bool>),

    /// A color
    Color(Color),

    /// An array of colors
    ColorArray(Vec<Color>),

    /// A 4D vector
    Vec4(Vec4),

    /// An array of 4D vectors,
    Vec4Array(Vec<Vec4>),

    /// A 4x4 matrix
    Mat4(Mat4),

    /// An array of 4x4 matrices
    Mat4Array(Vec<Mat4>),

    /// A texture
    Texture(RendererTextureId),

    /// An array of textures
    TextureArray(Vec<RendererTextureId>),
}

impl MaterialParameter {
    /// The default value for a missing [MaterialParameter::Boolean]
    pub const DEFAULT_BOOL: MaterialParameter = MaterialParameter::Boolean(false);

    /// The default value for a missing [MaterialParameter::Color]
    pub const DEFAULT_COLOR: MaterialParameter = MaterialParameter::Color(Color::BLACK);

    /// The default value for a missing [MaterialParameter::Vec4]
    pub const DEFAULT_VEC4: MaterialParameter = MaterialParameter::Vec4(Vec4::ZERO);

    /// The default value for a missing [MaterialParameter::Mat4]
    pub const DEFAULT_MAT4: MaterialParameter = MaterialParameter::Mat4(Mat4::ZERO);
}

/// Returns a new copy of the default texture, which is a 2x2 repeating pink-green image
#[profiling::function]
pub fn get_default_texture<const SIZE: u32>() -> TextureData {
    assert!(SIZE.is_power_of_two());

    TextureData {
        imagedata: DynamicImage::ImageRgb8(ImageBuffer::from_fn(SIZE, SIZE, |x, y| {
            let coord = x + (y % 2);

            if coord % 2 == 0 {
                image::Rgb([0xff, 0x00, 0x7f]) // pink
            } else {
                image::Rgb([0xcc, 0xff, 0x0b]) // green
            }
        })),
        filtering: TextureFiltering::Nearest,
        wrapping: TextureWrapping::Both(WrappingMethod::Repeat),
    }
}
