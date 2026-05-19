//! Types supported by WGSL shaders

mod primitives;

pub use primitives::*;
use wutengine_asset::assets::shader::ShaderBufferParameterType;
use wutengine_asset::assets::shader::ShaderOpaqueParameterType;
use wutengine_asset::assets::shader::ShaderVertexAttributeType;
use wutengine_util_macro::VariantName;

use crate::graphics::material::MaterialParameter;
use crate::graphics::sampler::DEFAULT_SAMPLER;
use crate::graphics::texture::DEFAULT_TEXTURE;
use crate::math::Vec4;

/// Alignment on the GPU of this data type.
///
/// Taken from [the WebGPU specification](https://www.w3.org/TR/WGSL/#alignment-and-size)
pub const fn shader_buffer_param_align(bt: ShaderBufferParameterType) -> usize {
    match bt {
        ShaderBufferParameterType::Flt => 4,
        ShaderBufferParameterType::Uint => 4,
        ShaderBufferParameterType::Int => 4,
        ShaderBufferParameterType::Vec2f => GVec2::<f32>::ALIGN,
        ShaderBufferParameterType::Vec3f => GVec3::<f32>::ALIGN,
        ShaderBufferParameterType::Vec4f => GVec4::<f32>::ALIGN,
        ShaderBufferParameterType::Vec2u => GVec2::<u32>::ALIGN,
        ShaderBufferParameterType::Vec3u => GVec3::<u32>::ALIGN,
        ShaderBufferParameterType::Vec4u => GVec4::<u32>::ALIGN,
        ShaderBufferParameterType::Vec2i => GVec2::<i32>::ALIGN,
        ShaderBufferParameterType::Vec3i => GVec3::<i32>::ALIGN,
        ShaderBufferParameterType::Vec4i => GVec4::<i32>::ALIGN,
        ShaderBufferParameterType::Mat4x4 => GMat4x4::<f32>::ALIGN,
    }
}

/// Size on the GPU of this data type. Also corresponds to the size on the CPU
///
/// Taken from [the WebGPU specification](https://www.w3.org/TR/WGSL/#alignment-and-size)
pub const fn shader_buffer_param_size(bt: ShaderBufferParameterType) -> usize {
    match bt {
        ShaderBufferParameterType::Flt => size_of::<f32>(),
        ShaderBufferParameterType::Uint => size_of::<u32>(),
        ShaderBufferParameterType::Int => size_of::<i32>(),
        ShaderBufferParameterType::Vec2f => size_of::<GVec2<f32>>(),
        ShaderBufferParameterType::Vec3f => size_of::<GVec3<f32>>(),
        ShaderBufferParameterType::Vec4f => size_of::<GVec4<f32>>(),
        ShaderBufferParameterType::Vec2u => size_of::<GVec2<u32>>(),
        ShaderBufferParameterType::Vec3u => size_of::<GVec3<u32>>(),
        ShaderBufferParameterType::Vec4u => size_of::<GVec4<u32>>(),
        ShaderBufferParameterType::Vec2i => size_of::<GVec2<i32>>(),
        ShaderBufferParameterType::Vec3i => size_of::<GVec3<i32>>(),
        ShaderBufferParameterType::Vec4i => size_of::<GVec4<i32>>(),
        ShaderBufferParameterType::Mat4x4 => size_of::<GMat4x4<f32>>(),
    }
}

/// Returns the default value for a given buffer parameter type
pub const fn shader_buffer_param_default_value(
    bt: ShaderBufferParameterType,
) -> ShaderBufferParameter {
    match bt {
        ShaderBufferParameterType::Flt => ShaderBufferParameter::Flt(bytemuck::zeroed()),
        ShaderBufferParameterType::Uint => ShaderBufferParameter::Uint(bytemuck::zeroed()),
        ShaderBufferParameterType::Int => ShaderBufferParameter::Int(bytemuck::zeroed()),
        ShaderBufferParameterType::Vec2f => ShaderBufferParameter::Vec2f(bytemuck::zeroed()),
        ShaderBufferParameterType::Vec3f => ShaderBufferParameter::Vec3f(bytemuck::zeroed()),
        ShaderBufferParameterType::Vec4f => ShaderBufferParameter::Vec4f(bytemuck::zeroed()),
        ShaderBufferParameterType::Vec2u => ShaderBufferParameter::Vec2u(bytemuck::zeroed()),
        ShaderBufferParameterType::Vec3u => ShaderBufferParameter::Vec3u(bytemuck::zeroed()),
        ShaderBufferParameterType::Vec4u => ShaderBufferParameter::Vec4u(bytemuck::zeroed()),
        ShaderBufferParameterType::Vec2i => ShaderBufferParameter::Vec2i(bytemuck::zeroed()),
        ShaderBufferParameterType::Vec3i => ShaderBufferParameter::Vec3i(bytemuck::zeroed()),
        ShaderBufferParameterType::Vec4i => ShaderBufferParameter::Vec4i(bytemuck::zeroed()),
        ShaderBufferParameterType::Mat4x4 => ShaderBufferParameter::Mat4x4(bytemuck::zeroed()),
    }
}

/// Returns the default valuye for a given opaque parameter type
pub fn shader_opaque_param_default_value(ot: ShaderOpaqueParameterType) -> ShaderOpaqueParameter {
    match ot {
        ShaderOpaqueParameterType::Sampler => {
            ShaderOpaqueParameter::Sampler(DEFAULT_SAMPLER.get_wgpu().clone())
        }
        ShaderOpaqueParameterType::Texture2D => {
            ShaderOpaqueParameter::Texture2D(DEFAULT_TEXTURE.get_view().clone())
        }
    }
}

/// Returns the [wgpu::BindingType] corresponding to an opaque parameter
pub const fn shader_opaque_param_wgpu_binding_type(
    ot: ShaderOpaqueParameterType,
) -> wgpu::BindingType {
    match ot {
        ShaderOpaqueParameterType::Sampler => {
            wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)
        }
        ShaderOpaqueParameterType::Texture2D => wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
    }
}

/// A shader buffer parameter. These represent the parameter types that have a concrete bit-value that can be stored
/// in a buffer, as opposed to "opaque" values like texture handles
#[derive(
    Debug,
    Clone,
    Copy,
    derive_more::IsVariant,
    derive_more::Unwrap,
    derive_more::TryUnwrap,
    derive_more::From,
    VariantName,
)]
pub enum ShaderBufferParameter {
    /// 32-bit float
    Flt(f32),

    /// 32-bit unsigned int
    Uint(u32),

    /// 32-bit signed int
    Int(i32),

    /// 2-component float vec
    Vec2f(GVec2<f32>),

    /// 3-component float vec
    Vec3f(GVec3<f32>),

    /// 4-component float vec
    Vec4f(GVec4<f32>),

    /// 2-component unsigned int vec
    Vec2u(GVec2<u32>),

    /// 3-component unsigned int vec
    Vec3u(GVec3<u32>),

    /// 4-component unsigned int vec
    Vec4u(GVec4<u32>),

    /// 2-component signed int vec
    Vec2i(GVec2<i32>),

    /// 3-component signed int vec
    Vec3i(GVec3<i32>),

    /// 4-component signed int vec
    Vec4i(GVec4<i32>),

    /// 4x4 float matrix
    Mat4x4(GMat4x4<f32>),
}

impl ShaderBufferParameter {
    /// Returns the type of this buffer parameter
    #[inline]
    pub const fn get_type(&self) -> ShaderBufferParameterType {
        match self {
            Self::Flt(_) => ShaderBufferParameterType::Flt,
            Self::Uint(_) => ShaderBufferParameterType::Uint,
            Self::Int(_) => ShaderBufferParameterType::Int,
            Self::Vec2f(_) => ShaderBufferParameterType::Vec2f,
            Self::Vec3f(_) => ShaderBufferParameterType::Vec3f,
            Self::Vec4f(_) => ShaderBufferParameterType::Vec4f,
            Self::Vec2u(_) => ShaderBufferParameterType::Vec2u,
            Self::Vec3u(_) => ShaderBufferParameterType::Vec3u,
            Self::Vec4u(_) => ShaderBufferParameterType::Vec4u,
            Self::Vec2i(_) => ShaderBufferParameterType::Vec2i,
            Self::Vec3i(_) => ShaderBufferParameterType::Vec3i,
            Self::Vec4i(_) => ShaderBufferParameterType::Vec4i,
            Self::Mat4x4(_) => ShaderBufferParameterType::Mat4x4,
        }
    }

    /// Alignment on the GPU of this data type.
    ///
    /// Taken from [the WebGPU specification](https://www.w3.org/TR/WGSL/#alignment-and-size)
    #[inline]
    pub const fn align(&self) -> usize {
        shader_buffer_param_align(self.get_type())
    }

    /// Size on the GPU of this data type. Also corresponds to the size on the CPU
    ///
    /// Taken from [the WebGPU specification](https://www.w3.org/TR/WGSL/#alignment-and-size)
    #[inline]
    pub const fn size(&self) -> usize {
        shader_buffer_param_size(self.get_type())
    }

    /// Returns this buffer parameter as a raw byte vector
    #[inline]
    pub fn bytes(&self) -> &[u8] {
        match self {
            Self::Flt(x) => bytemuck::bytes_of(x),
            Self::Uint(x) => bytemuck::bytes_of(x),
            Self::Int(x) => bytemuck::bytes_of(x),
            Self::Vec2f(x) => bytemuck::bytes_of(x),
            Self::Vec3f(x) => bytemuck::bytes_of(x),
            Self::Vec4f(x) => bytemuck::bytes_of(x),
            Self::Vec2u(x) => bytemuck::bytes_of(x),
            Self::Vec3u(x) => bytemuck::bytes_of(x),
            Self::Vec4u(x) => bytemuck::bytes_of(x),
            Self::Vec2i(x) => bytemuck::bytes_of(x),
            Self::Vec3i(x) => bytemuck::bytes_of(x),
            Self::Vec4i(x) => bytemuck::bytes_of(x),
            Self::Mat4x4(x) => bytemuck::bytes_of(x),
        }
    }

    /// Sets the value of this parameter from an external [MaterialParameter], casting
    /// if possible. Will not change the type of this [ShaderBufferParameter]
    #[inline]
    #[expect(clippy::todo, reason = "Casting is a lot of work")]
    pub fn set_from(&mut self, value: MaterialParameter) -> bool {
        match self {
            Self::Flt(cur) => {
                if let MaterialParameter::Flt(f) = value {
                    *cur = f;
                    return true;
                }
            }
            Self::Uint(cur) => {
                if let MaterialParameter::Uint(u) = value {
                    *cur = u;
                    return true;
                }
            }
            Self::Int(cur) => {
                if let MaterialParameter::Int(i) = value {
                    *cur = i;
                    return true;
                }
            }
            Self::Vec2f(cur) => {
                if let MaterialParameter::Vec2(v) = value {
                    *cur = v.into();
                    return true;
                }
            }
            Self::Vec3f(cur) => {
                if let MaterialParameter::Vec3(v) = value {
                    *cur = v.into();
                    return true;
                }
            }
            Self::Vec4f(cur) => match value {
                MaterialParameter::Vec4(v) => {
                    *cur = v.into();
                    return true;
                }
                MaterialParameter::Vec2(v) => {
                    *cur = Vec4::new(v.x, v.y, 0.0, 0.0).into();
                    return true;
                }
                MaterialParameter::Vec3(v) => {
                    *cur = Vec4::new(v.x, v.y, v.z, 0.0).into();
                    return true;
                }
                MaterialParameter::Color(c) => {
                    *cur = c.as_vec4().into();
                    return true;
                }
                _ => {}
            },
            Self::Vec2u(_) => todo!(),
            Self::Vec3u(_) => todo!(),
            Self::Vec4u(_) => todo!(),
            Self::Vec2i(_) => todo!(),
            Self::Vec3i(_) => todo!(),
            Self::Vec4i(_) => todo!(),
            Self::Mat4x4(cur) => {
                if let MaterialParameter::Mat4(mat) = value {
                    *cur = mat.into();
                    return true;
                }
            }
        }

        false
    }
}

impl<'a> From<&'a ShaderBufferParameter> for ShaderBufferParameterType {
    #[inline(always)]
    fn from(value: &'a ShaderBufferParameter) -> Self {
        value.get_type()
    }
}

/// An opaque shader parameter, representing things like texture handles, sampler objects, and other
/// non-bit-valued parameters
#[derive(
    Debug,
    Clone,
    derive_more::IsVariant,
    derive_more::Unwrap,
    derive_more::TryUnwrap,
    derive_more::From,
    VariantName,
)]
pub enum ShaderOpaqueParameter {
    /// A 2D texture
    Texture2D(wgpu::TextureView),

    /// A sampler object
    Sampler(wgpu::Sampler),
}

impl ShaderOpaqueParameter {
    /// Updates the value of this [ShaderOpaqueParameter] from the given [MaterialParameter]
    #[inline]
    pub fn set_from(&mut self, value: MaterialParameter) -> bool {
        //TODO: Add error handling for not-yet-loaded assets?
        match self {
            Self::Texture2D(cur) => {
                if let MaterialParameter::Texture2D(tex) = value {
                    *cur = tex.get_ref().unwrap().get_view().clone();
                    return true;
                }
            }
            Self::Sampler(cur) => {
                if let MaterialParameter::Sampler(smp) = value {
                    *cur = smp.get_ref().unwrap().get_wgpu().clone();
                    return true;
                }
            }
        }

        false
    }

    /// Returns the [wgpu::BindingResource] corresponding to this parameter
    #[inline]
    pub(crate) fn to_binding_resource(&self) -> wgpu::BindingResource<'_> {
        match self {
            Self::Texture2D(texture_view) => wgpu::BindingResource::TextureView(texture_view),
            Self::Sampler(sampler) => wgpu::BindingResource::Sampler(sampler),
        }
    }
}

/// Returns the [wgpu::VertexFormat] corresponding to this [ShaderVertexAttributeType]
pub const fn shader_attr_wgpu_vertex_format(attr: ShaderVertexAttributeType) -> wgpu::VertexFormat {
    match attr {
        ShaderVertexAttributeType::Position => wgpu::VertexFormat::Float32x3,
        ShaderVertexAttributeType::Uv { .. } => wgpu::VertexFormat::Float32x2,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Test primitive sizes and alignments according to the [WebGPU spec](https://www.w3.org/TR/WGSL/#alignment-and-size)
    #[test]
    fn size_align_primitives() {
        assert_eq!(4, size_of::<f32>());
        assert_eq!(4, align_of::<f32>());
        assert_eq!(4, size_of::<u32>());
        assert_eq!(4, align_of::<u32>());
        assert_eq!(4, size_of::<i32>());
        assert_eq!(4, align_of::<i32>());
    }

    /// Test vector sizes and alignments according to the [WebGPU spec](https://www.w3.org/TR/WGSL/#alignment-and-size)
    #[test]
    fn size_align_vecs() {
        assert_eq!(8, size_of::<GVec2<f32>>());
        assert_eq!(8, size_of::<GVec2<u32>>());
        assert_eq!(8, size_of::<GVec2<i32>>());
        assert_eq!(8, GVec2::<f32>::ALIGN);
        assert_eq!(8, GVec2::<u32>::ALIGN);
        assert_eq!(8, GVec2::<i32>::ALIGN);

        assert_eq!(12, size_of::<GVec3<f32>>());
        assert_eq!(12, size_of::<GVec3<u32>>());
        assert_eq!(12, size_of::<GVec3<i32>>());
        assert_eq!(16, GVec3::<f32>::ALIGN);
        assert_eq!(16, GVec3::<u32>::ALIGN);
        assert_eq!(16, GVec3::<i32>::ALIGN);

        assert_eq!(16, size_of::<GVec4<f32>>());
        assert_eq!(16, size_of::<GVec4<u32>>());
        assert_eq!(16, size_of::<GVec4<i32>>());
        assert_eq!(16, GVec4::<f32>::ALIGN);
        assert_eq!(16, GVec4::<u32>::ALIGN);
        assert_eq!(16, GVec4::<i32>::ALIGN);
    }

    /// Test matrix sizes and alignments according to the [WebGPU spec](https://www.w3.org/TR/WGSL/#alignment-and-size)
    #[test]
    fn size_align_matrices() {
        assert_eq!(64, size_of::<GMat4x4::<f32>>());
        assert_eq!(16, GMat4x4::<f32>::ALIGN);

        assert_eq!(64, size_of::<GMat4x3::<f32>>());
        assert_eq!(16, GMat4x3::<f32>::ALIGN);

        assert_eq!(48, size_of::<GMat3x4::<f32>>());
        assert_eq!(16, GMat3x4::<f32>::ALIGN);
    }
}
