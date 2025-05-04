use core::convert::Infallible;

use bytemuck::NoUninit;
use wutengine_graphics::material::MaterialParameter;

use crate::gltypes::{GlMat4f, GlVec3f, GlVec4f};
use crate::opengl::types::{GLboolean, GLfloat, GLuint};

const fn vec2_size_align<T>() -> (usize, usize) {
    (size_of::<T>() * 2, size_of::<T>() * 2)
}

const fn vec3_size_align<T>() -> (usize, usize) {
    vec4_size_align::<T>()
}

const fn vec4_size_align<T>() -> (usize, usize) {
    (size_of::<T>() * 4, size_of::<T>() * 4)
}

const fn std140_scalar_array_size_align(base_size: usize, count: usize) -> (usize, usize) {
    let (size_vec4, _) = const { vec4_size_align::<GLfloat>() };
    let size_element = base_size.next_multiple_of(size_vec4);

    (size_element * count, size_element)
}

pub(crate) fn param_to_std140_size_align(param: &MaterialParameter) -> (usize, usize) {
    match param {
        MaterialParameter::U32(_) => (size_of::<GLuint>(), size_of::<GLuint>()),
        MaterialParameter::U32Array(items) => {
            std140_scalar_array_size_align(size_of::<GLuint>(), items.len())
        }
        MaterialParameter::Vec3(_) => vec3_size_align::<GLfloat>(),
        MaterialParameter::Vec3Array(items) => {
            std140_scalar_array_size_align(vec3_size_align::<GLfloat>().0, items.len())
        }
        MaterialParameter::Vec4(_) => vec4_size_align::<GLfloat>(),
        MaterialParameter::Vec4Array(items) => {
            std140_scalar_array_size_align(vec4_size_align::<GLfloat>().0, items.len())
        }
        MaterialParameter::Mat4(_) => {
            std140_scalar_array_size_align(vec4_size_align::<GLfloat>().0, 4)
        }
        MaterialParameter::Mat4Array(items) => {
            std140_scalar_array_size_align(vec4_size_align::<GLfloat>().0, 4 * items.len())
        }
        MaterialParameter::Texture2D(_) => todo!(),
        MaterialParameter::Texture2DArray(_) => todo!(),
    }
}

#[inline(always)]
fn extend_and_pad(buf: &mut Vec<u8>, align: usize, elem: &impl NoUninit) {
    let bytes = bytemuck::bytes_of(elem);
    buf.extend_from_slice(bytes);
    buf.resize(buf.len().next_multiple_of(align), 0);
}

#[profiling::function]
pub(crate) fn param_to_std140_buffer(param: &MaterialParameter) -> Vec<u8> {
    let (size, align) = param_to_std140_size_align(param);

    let mut buffer: Vec<u8> = Vec::with_capacity(size);

    match param {
        MaterialParameter::U32(x) => {
            extend_and_pad(&mut buffer, align, x);
        }
        MaterialParameter::U32Array(items) => {
            for &elem in items {
                extend_and_pad(&mut buffer, align, &elem);
            }
        }
        MaterialParameter::Vec3(vec3) => {
            extend_and_pad(&mut buffer, align, &GlVec3f::from(*vec3));
        }
        MaterialParameter::Vec3Array(items) => {
            for &elem in items {
                let as_gl = GlVec3f::from(elem);
                extend_and_pad(&mut buffer, align, &as_gl);
            }
        }
        MaterialParameter::Vec4(vec4) => {
            extend_and_pad(&mut buffer, align, &GlVec4f::from(*vec4));
        }
        MaterialParameter::Vec4Array(items) => {
            for &elem in items {
                let as_gl = GlVec4f::from(elem);
                extend_and_pad(&mut buffer, align, &as_gl);
            }
        }
        MaterialParameter::Mat4(mat4) => {
            extend_and_pad(&mut buffer, align, &GlMat4f::from(*mat4));
        }
        MaterialParameter::Mat4Array(items) => {
            for &elem in items {
                let as_gl = GlMat4f::from(elem);
                extend_and_pad(&mut buffer, align, &as_gl);
            }
        }
        MaterialParameter::Texture2D(_) => todo!(),
        MaterialParameter::Texture2DArray(_) => todo!(),
    }

    assert_eq!(size, buffer.len());
    buffer
}
