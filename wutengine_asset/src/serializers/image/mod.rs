//! Custom [serde] serialization/deserialization implementations for types from the [image] crate

use image::{ColorType, DynamicImage, ImageBuffer};
use serde::{Deserialize, Serialize};

/// A serializable [DynamicImage]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializedDynamicImage {
    color_type: SerializedColorType,

    dims: (u32, u32),

    #[serde(with = "serde_bytes")]
    bytes: Vec<u8>,
}

/// A serializable [ColorType]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum SerializedColorType {
    L8,
    La8,
    Rgb8,
    Rgba8,

    L16,
    La16,
    Rgb16,
    Rgba16,
    Rgb32F,
    Rgba32F,
}

impl From<&DynamicImage> for SerializedDynamicImage {
    fn from(value: &DynamicImage) -> Self {
        Self {
            color_type: value.color().into(),
            dims: (value.width(), value.height()),
            bytes: value.as_bytes().to_vec(),
        }
    }
}

impl From<SerializedDynamicImage> for DynamicImage {
    fn from(value: SerializedDynamicImage) -> Self {
        macro_rules! cast_vec {
            () => {
                ImageBuffer::from_raw(
                    value.dims.0,
                    value.dims.1,
                    bytemuck::allocation::cast_vec(value.bytes),
                )
                .unwrap()
            };
        }
        match value.color_type {
            SerializedColorType::L8 => DynamicImage::ImageLuma8(
                ImageBuffer::from_raw(value.dims.0, value.dims.1, value.bytes).unwrap(),
            ),
            SerializedColorType::La8 => DynamicImage::ImageLumaA8(
                ImageBuffer::from_raw(value.dims.0, value.dims.1, value.bytes).unwrap(),
            ),
            SerializedColorType::Rgb8 => DynamicImage::ImageRgb8(
                ImageBuffer::from_raw(value.dims.0, value.dims.1, value.bytes).unwrap(),
            ),
            SerializedColorType::Rgba8 => DynamicImage::ImageRgba8(
                ImageBuffer::from_raw(value.dims.0, value.dims.1, value.bytes).unwrap(),
            ),
            SerializedColorType::L16 => DynamicImage::ImageLuma16(cast_vec!()),
            SerializedColorType::La16 => DynamicImage::ImageLumaA16(cast_vec!()),
            SerializedColorType::Rgb16 => DynamicImage::ImageRgb16(cast_vec!()),
            SerializedColorType::Rgba16 => DynamicImage::ImageRgba16(cast_vec!()),
            SerializedColorType::Rgb32F => DynamicImage::ImageRgb32F(cast_vec!()),
            SerializedColorType::Rgba32F => DynamicImage::ImageRgba32F(cast_vec!()),
        }
    }
}

impl From<ColorType> for SerializedColorType {
    fn from(value: ColorType) -> Self {
        match value {
            ColorType::L8 => SerializedColorType::L8,
            ColorType::La8 => SerializedColorType::La8,
            ColorType::Rgb8 => SerializedColorType::Rgb8,
            ColorType::Rgba8 => SerializedColorType::Rgba8,
            ColorType::L16 => SerializedColorType::L16,
            ColorType::La16 => SerializedColorType::La16,
            ColorType::Rgb16 => SerializedColorType::Rgb16,
            ColorType::Rgba16 => SerializedColorType::Rgba16,
            ColorType::Rgb32F => SerializedColorType::Rgba32F,
            ColorType::Rgba32F => SerializedColorType::Rgba32F,
            unknown => panic!("Unknown color format: {unknown:?}"),
        }
    }
}

impl From<SerializedColorType> for ColorType {
    fn from(value: SerializedColorType) -> Self {
        match value {
            SerializedColorType::L8 => ColorType::L8,
            SerializedColorType::La8 => ColorType::La8,
            SerializedColorType::Rgb8 => ColorType::Rgb8,
            SerializedColorType::Rgba8 => ColorType::Rgba8,
            SerializedColorType::L16 => ColorType::L16,
            SerializedColorType::La16 => ColorType::La16,
            SerializedColorType::Rgb16 => ColorType::Rgb16,
            SerializedColorType::Rgba16 => ColorType::Rgba16,
            SerializedColorType::Rgb32F => ColorType::Rgba32F,
            SerializedColorType::Rgba32F => ColorType::Rgba32F,
        }
    }
}

/// Implements [serde] serialization and deserialization for [image::DynamicImage]
pub mod dynamic_image {
    use image::DynamicImage;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use crate::serializers::image::SerializedDynamicImage;

    /// [image::DynamicImage] [serde] serializaiton implementation
    pub fn serialize<S>(image: &DynamicImage, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerializedDynamicImage::from(image).serialize(serializer)
    }

    /// [image::DynamicImage] [serde] deserialization implementation
    pub fn deserialize<'de, D>(deserializer: D) -> Result<DynamicImage, D::Error>
    where
        D: Deserializer<'de>,
    {
        SerializedDynamicImage::deserialize(deserializer).map(|as_ser| as_ser.into())
    }
}
