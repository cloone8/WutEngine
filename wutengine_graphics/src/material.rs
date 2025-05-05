//! Module for the data of a WutEngine material. A material describes the way a given mesh is rendered

use std::collections::HashMap;

use glam::{Mat4, Vec3, Vec4};
use image::{DynamicImage, ImageBuffer};

use crate::color::Color;
use crate::renderer::RendererTexture2DId;
use crate::shader::ShaderVariantId;
use crate::shader::uniform::UniformType;
use crate::texture::{TextureData, TextureFiltering, TextureWrapping, WrappingMethod};

/// The data of a material
#[derive(Debug, Clone, Default)]
pub struct MaterialData {
    /// The shader used to render this material. If [None], this material will not render
    pub shader: Option<ShaderVariantId>,

    /// Any material parameters
    pub parameters: HashMap<String, MaterialParameter>,
}

/// The value of a material parameter in a [MaterialData]
#[derive(Debug, Clone)]
pub enum MaterialParameter {
    /// A 32-bit unsigned integer
    U32(u32),

    /// An array of 32-bit unsigned integers
    U32Array(Vec<u32>),

    /// A 3D vector
    Vec3(Vec3),

    /// An array of 3D vectors,
    Vec3Array(Vec<Vec3>),

    /// A 4D vector
    Vec4(Vec4),

    /// An array of 4D vectors,
    Vec4Array(Vec<Vec4>),

    /// A 4x4 matrix
    Mat4(Mat4),

    /// An array of 4x4 matrices
    Mat4Array(Vec<Mat4>),

    /// A texture
    Texture2D(RendererTexture2DId),

    /// An array of textures
    Texture2DArray(Vec<RendererTexture2DId>),
}

impl MaterialParameter {
    #[inline]
    pub fn get_type(&self) -> UniformType {
        match self {
            MaterialParameter::U32(_) => UniformType::U32,
            MaterialParameter::U32Array(items) => {
                UniformType::Array(Box::new(UniformType::U32), items.len())
            }
            MaterialParameter::Vec3(_) => UniformType::Vec3,
            MaterialParameter::Vec3Array(items) => {
                UniformType::Array(Box::new(UniformType::Vec3), items.len())
            }
            MaterialParameter::Vec4(_) => UniformType::Vec4,
            MaterialParameter::Vec4Array(items) => {
                UniformType::Array(Box::new(UniformType::Vec4), items.len())
            }
            MaterialParameter::Mat4(_) => UniformType::Mat4,
            MaterialParameter::Mat4Array(items) => {
                UniformType::Array(Box::new(UniformType::Mat4), items.len())
            }
            MaterialParameter::Texture2D(_) => UniformType::Tex2D,
            MaterialParameter::Texture2DArray(items) => {
                UniformType::Array(Box::new(UniformType::Tex2D), items.len())
            }
        }
    }
}

/// Returns a new copy of the default texture, which is a 2x2 repeating pink-green image
#[profiling::function]
pub fn get_default_texture2d<const SIZE: u32>() -> TextureData {
    const {
        assert!(SIZE.is_power_of_two());
    }

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
