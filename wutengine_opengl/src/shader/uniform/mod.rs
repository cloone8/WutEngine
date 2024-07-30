use core::alloc::Layout;
use core::ffi::CStr;
use core::num::NonZero;

use thiserror::Error;

use crate::opengl::types::{GLchar, GLenum, GLint, GLsizei, GLuint};
use crate::opengl::{self, Gl};
mod conversions;

pub use conversions::GlUniformConversionError;

#[derive(Debug, Clone, Copy)]
pub struct UniformDescriptor {
    location: GLint,
    uniform_type: UniformType,
    uniform_count: GLsizei,
}

#[derive(Debug, Clone, Copy)]
enum UniformType {
    Float { count: u8 },
    Int { count: u8 },
    Uint { count: u8 },
    Matrix { rows: u8, cols: u8 },
}

impl UniformDescriptor {
    pub unsafe fn get_for(gl: &Gl, program: NonZero<GLuint>) -> Vec<(String, Self)> {
        let handle = program.get();
        let mut output: Vec<(String, Self)> = Vec::new();

        let mut uniform_max_name_len: GLint = -1;
        let mut active_uniforms: GLint = -1;

        unsafe {
            gl.GetProgramiv(
                handle,
                opengl::ACTIVE_UNIFORM_MAX_LENGTH,
                &mut uniform_max_name_len,
            );

            gl.GetProgramiv(handle, opengl::ACTIVE_UNIFORMS, &mut active_uniforms);

            if active_uniforms == 0 {
                log::debug!("No active uniforms");
                // No active uniforms that we can use
                return output;
            }

            debug_assert_ne!(
                -1, uniform_max_name_len,
                "Undefined name length for uniforms"
            );

            let uniform_name_layout =
                Layout::array::<GLchar>(uniform_max_name_len as usize).unwrap();
            let uniform_name_buf: *mut u8 = std::alloc::alloc_zeroed(uniform_name_layout);

            for i in 0..active_uniforms {
                let mut actual_name_len: GLsizei = 0;
                let mut uniform_size: GLint = 0;
                let mut uniform_type: GLenum = 0;

                gl.GetActiveUniform(
                    handle,
                    i as GLuint,
                    uniform_max_name_len,
                    &mut actual_name_len,
                    &mut uniform_size,
                    &mut uniform_type,
                    uniform_name_buf as *mut GLchar,
                );

                debug_assert!(actual_name_len < uniform_max_name_len);
                debug_assert!(actual_name_len > 0);

                let name_byte_slice =
                    std::slice::from_raw_parts(uniform_name_buf, (actual_name_len + 1) as usize);
                let uniform_name_str = CStr::from_bytes_with_nul(name_byte_slice)
                    .unwrap()
                    .to_str()
                    .expect("Invalid UTF-8 in uniform name")
                    .to_string();

                let as_descriptor = Self {
                    location: i,
                    uniform_type: UniformType::try_from(uniform_type)
                        .expect("Unsupported OpenGL uniform type returned"),
                    uniform_count: uniform_size,
                };

                output.push((uniform_name_str, as_descriptor));
            }

            std::alloc::dealloc(uniform_name_buf, uniform_name_layout);
        }

        output
    }

    pub unsafe fn set_with<T: IntoGlUniformData>(self, gl: &Gl, data: T) -> Result<(), T::Error> {
        match self.uniform_type {
            UniformType::Float { count } => {
                let as_float_buf = data.as_float_buf(count, self.uniform_count as usize)?;

                debug_assert_eq!(
                    (self.uniform_count as usize) * (count as usize),
                    as_float_buf.len(),
                    "Buffer with invalid length returned"
                );

                match count {
                    0 => unreachable!("Float vectors of length 0 are not possible"),
                    1 => gl.Uniform1fv(self.location, self.uniform_count, as_float_buf.as_ptr()),
                    2 => gl.Uniform2fv(self.location, self.uniform_count, as_float_buf.as_ptr()),
                    3 => gl.Uniform3fv(self.location, self.uniform_count, as_float_buf.as_ptr()),
                    4 => gl.Uniform4fv(self.location, self.uniform_count, as_float_buf.as_ptr()),
                    _ => unreachable!("Float vectors larger than 4 are not possible"),
                }
            }
            UniformType::Int { count } => todo!(),
            UniformType::Uint { count } => todo!(),
            UniformType::Matrix { rows, cols } => todo!(),
        }

        Ok(())
    }
}

pub trait IntoGlUniformData {
    type Error;

    fn as_float_buf(&self, float_vec_size: u8, array_len: usize) -> Result<Vec<f32>, Self::Error>;
}

#[derive(Debug, Clone, Copy, Error)]
pub enum UniformTypeParsingError {
    #[error("Unknown GLenum type: 0x{:x}", 0)]
    UnknownType(GLenum),
}

impl TryFrom<GLenum> for UniformType {
    type Error = UniformTypeParsingError;

    fn try_from(value: GLenum) -> Result<Self, Self::Error> {
        let parsed = match value {
            opengl::FLOAT => UniformType::Float { count: 1 },
            opengl::FLOAT_VEC2 => UniformType::Float { count: 2 },
            opengl::FLOAT_VEC3 => UniformType::Float { count: 3 },
            opengl::FLOAT_VEC4 => UniformType::Float { count: 4 },
            _ => return Err(UniformTypeParsingError::UnknownType(value)),
        };

        Ok(parsed)
    }
}
