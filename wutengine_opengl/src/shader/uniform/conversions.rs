use thiserror::Error;
use wutengine_graphics::material::MaterialParameter;

use super::IntoGlUniformData;

#[derive(Debug, Clone, Error)]
pub enum GlUniformConversionError {
    #[error("Expected array of length {}, single value given", 0)]
    ArrayExpected(usize),

    #[error("Expected singular value, got array of length {}", 0)]
    ArrayNotExpected(usize),

    #[error(
        "Expected array of length {}, got array of length {}",
        expected,
        actual
    )]
    ArrayLengthMismatch { expected: usize, actual: usize },

    #[error("Unexpected parameter type")]
    UnexpectedType,
}

impl IntoGlUniformData for &MaterialParameter {
    type Error = GlUniformConversionError;

    fn as_float_buf(&self, float_vec_size: u8, array_len: usize) -> Result<Vec<f32>, Self::Error> {
        debug_assert_ne!(0, array_len, "Cannot convert into zero values");

        if array_len > 1 {
            if let MaterialParameter::Array(array) = self {
                if array.len() != array_len {
                    return Err(GlUniformConversionError::ArrayLengthMismatch {
                        expected: array_len,
                        actual: array.len(),
                    });
                }
            } else {
                return Err(GlUniformConversionError::ArrayExpected(array_len));
            }
        } else if let MaterialParameter::Array(array) = self {
            return Err(GlUniformConversionError::ArrayNotExpected(array.len()));
        }

        let mut output: Vec<f32> = Vec::with_capacity((float_vec_size as usize) * array_len);

        if let MaterialParameter::Array(array) = self {
            for element in array {
                output.extend(element.as_float_buf(float_vec_size, 1)?);
            }
        } else {
            match float_vec_size {
                1 => match self {
                    MaterialParameter::Float(val) => output.push(*val),
                    _ => return Err(GlUniformConversionError::UnexpectedType),
                },
                2 => match self {
                    MaterialParameter::Vec2(val) => {
                        output.push(val.x);
                        output.push(val.y);
                    }
                    _ => return Err(GlUniformConversionError::UnexpectedType),
                },
                3 => match self {
                    MaterialParameter::Vec3(val) => {
                        output.push(val.x);
                        output.push(val.y);
                        output.push(val.z);
                    }
                    _ => return Err(GlUniformConversionError::UnexpectedType),
                },
                4 => match self {
                    MaterialParameter::Vec4(val) => {
                        output.push(val.x);
                        output.push(val.y);
                        output.push(val.z);
                        output.push(val.w);
                    }
                    MaterialParameter::Color(val) => {
                        output.push(val.r);
                        output.push(val.g);
                        output.push(val.b);
                        output.push(val.a);
                    }
                    _ => return Err(GlUniformConversionError::UnexpectedType),
                },
                _ => unreachable!("Invalid float vector size: {}", float_vec_size),
            }
        }

        Ok(output)
    }
}
