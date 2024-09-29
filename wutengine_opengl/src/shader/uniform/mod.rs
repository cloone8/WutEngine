use core::alloc::Layout;
use core::ffi::CStr;
use core::num::NonZero;

use thiserror::Error;
use wutengine_graphics::material::MaterialParameter;

use crate::error::check_gl_err;
use crate::gltypes::GlMat4f;
use crate::opengl::types::{GLchar, GLenum, GLint, GLsizei, GLuint};
use crate::opengl::{self, Gl};

#[derive(Debug, Clone, Copy)]
pub(crate) struct UniformDescriptor {
    location: GLint,
    uniform_type: UniformType,
    uniform_count: GLsizei,
}

#[derive(Debug, Clone, Copy)]
enum UniformType {
    Float,
    Vec2f,
    Vec3f,
    Vec4f,
    Mat4,
}

impl UniformDescriptor {
    pub(crate) unsafe fn get_for(gl: &Gl, program: NonZero<GLuint>) -> Vec<(String, Self)> {
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
            check_gl_err!(gl);

            gl.GetProgramiv(handle, opengl::ACTIVE_UNIFORMS, &mut active_uniforms);
            check_gl_err!(gl);

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
                check_gl_err!(gl);

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

    /// Sets this uniform to the value contained in the
    /// given parameter, if it contains data that is able
    /// to be mapped to the type of the uniform.
    ///
    /// Returns whether the uniform has been set successfully
    pub(crate) fn set_with(self, gl: &Gl, data: &MaterialParameter) -> bool {
        if self.uniform_count != 1 {
            todo!("Arrays not yet handled");
        }

        match self.uniform_type {
            UniformType::Float => {
                todo!("Not yet supported");
            }
            UniformType::Vec2f => {
                todo!("Not yet supported");
            }
            UniformType::Vec3f => {
                todo!("Not yet supported");
            }
            UniformType::Vec4f => match data {
                MaterialParameter::Color(color) => {
                    let color = *color;
                    unsafe {
                        gl.Uniform4f(self.location, color.r, color.g, color.b, color.a);
                    }
                    check_gl_err!(gl);
                }
                _ => {
                    return false;
                }
            },
            UniformType::Mat4 => match data {
                MaterialParameter::Mat4(mat4) => {
                    let mat = GlMat4f::from(*mat4);
                    unsafe {
                        gl.UniformMatrix4fv(
                            self.location,
                            1,
                            opengl::FALSE,
                            (&mat as *const GlMat4f) as *const f32,
                        );
                        check_gl_err!(gl);
                    }
                }
                _ => {
                    return false;
                }
            },
        }

        true
    }
}

#[derive(Debug, Clone, Copy, Error)]
pub(crate) enum UniformTypeParsingError {
    #[error("Unknown GLenum type: 0x{:x}", 0)]
    UnknownType(GLenum),
}

impl TryFrom<GLenum> for UniformType {
    type Error = UniformTypeParsingError;

    fn try_from(value: GLenum) -> Result<Self, Self::Error> {
        let parsed = match value {
            opengl::FLOAT => UniformType::Float,
            opengl::FLOAT_VEC2 => UniformType::Vec2f,
            opengl::FLOAT_VEC3 => UniformType::Vec3f,
            opengl::FLOAT_VEC4 => UniformType::Vec4f,
            opengl::FLOAT_MAT4 => UniformType::Mat4,
            _ => return Err(UniformTypeParsingError::UnknownType(value)),
        };

        Ok(parsed)
    }
}
