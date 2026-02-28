//! Types supported by WGSL shaders

mod primitives;

pub use primitives::*;
use serde::{Deserialize, Serialize};

use crate::MaterialParameter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShaderBufferParameterType {
    Flt,
    Uint,
    Int,
    Vec2f,
    Vec3f,
    Vec4f,
    Vec2u,
    Vec3u,
    Vec4u,
    Vec2i,
    Vec3i,
    Vec4i,
    Mat4x4,
}

impl ShaderBufferParameterType {
    /// Alignment on the GPU of this data type.
    ///
    /// Taken from [the WebGPU specification](https://www.w3.org/TR/WGSL/#alignment-and-size)
    #[inline]
    pub const fn align(self) -> usize {
        match self {
            Self::Flt => 4,
            Self::Uint => 4,
            Self::Int => 4,
            Self::Vec2f => 8,
            Self::Vec3f => 16,
            Self::Vec4f => 16,
            Self::Vec2u => 8,
            Self::Vec3u => 16,
            Self::Vec4u => 16,
            Self::Vec2i => 8,
            Self::Vec3i => 16,
            Self::Vec4i => 16,
            Self::Mat4x4 => 16,
        }
    }

    /// Size on the GPU of this data type. Also corresponds to the size on the CPU
    ///
    /// Taken from [the WebGPU specification](https://www.w3.org/TR/WGSL/#alignment-and-size)
    #[inline]
    pub const fn size(self) -> usize {
        match self {
            Self::Flt => size_of::<f32>(),
            Self::Uint => size_of::<u32>(),
            Self::Int => size_of::<i32>(),
            Self::Vec2f => size_of::<GVec2<f32>>(),
            Self::Vec3f => size_of::<GVec3<f32>>(),
            Self::Vec4f => size_of::<GVec4<f32>>(),
            Self::Vec2u => size_of::<GVec2<u32>>(),
            Self::Vec3u => size_of::<GVec3<u32>>(),
            Self::Vec4u => size_of::<GVec4<u32>>(),
            Self::Vec2i => size_of::<GVec2<i32>>(),
            Self::Vec3i => size_of::<GVec3<i32>>(),
            Self::Vec4i => size_of::<GVec4<i32>>(),
            Self::Mat4x4 => size_of::<GMat4x4<f32>>(),
        }
    }

    #[inline]
    pub const fn to_default_value(self) -> ShaderBufferParameter {
        match self {
            Self::Flt => ShaderBufferParameter::Flt(bytemuck::zeroed()),
            Self::Uint => ShaderBufferParameter::Uint(bytemuck::zeroed()),
            Self::Int => ShaderBufferParameter::Int(bytemuck::zeroed()),
            Self::Vec2f => ShaderBufferParameter::Vec2f(bytemuck::zeroed()),
            Self::Vec3f => ShaderBufferParameter::Vec3f(bytemuck::zeroed()),
            Self::Vec4f => ShaderBufferParameter::Vec4f(bytemuck::zeroed()),
            Self::Vec2u => ShaderBufferParameter::Vec2u(bytemuck::zeroed()),
            Self::Vec3u => ShaderBufferParameter::Vec3u(bytemuck::zeroed()),
            Self::Vec4u => ShaderBufferParameter::Vec4u(bytemuck::zeroed()),
            Self::Vec2i => ShaderBufferParameter::Vec2i(bytemuck::zeroed()),
            Self::Vec3i => ShaderBufferParameter::Vec3i(bytemuck::zeroed()),
            Self::Vec4i => ShaderBufferParameter::Vec4i(bytemuck::zeroed()),
            Self::Mat4x4 => ShaderBufferParameter::Mat4x4(bytemuck::zeroed()),
        }
    }
}

impl<'a> From<&'a ShaderBufferParameterType> for ShaderBufferParameterType {
    #[inline(always)]
    fn from(value: &'a ShaderBufferParameterType) -> Self {
        *value
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShaderOpaqueParameterType {
    Sampler,
    #[serde(rename = "texture_2d")]
    Texture2D,
}

impl ShaderOpaqueParameterType {
    pub fn to_default_value(self) -> ShaderOpaqueParameter {
        todo!()
    }

    #[inline]
    pub const fn to_wgpu_binding_type(self) -> wgpu::BindingType {
        match self {
            Self::Sampler => wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            Self::Texture2D => wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
        }
    }
}

#[derive(
    Debug,
    Clone,
    Copy,
    derive_more::IsVariant,
    derive_more::Unwrap,
    derive_more::TryUnwrap,
    derive_more::From,
)]
pub enum ShaderBufferParameter {
    Flt(f32),
    Uint(u32),
    Int(i32),
    Vec2f(GVec2<f32>),
    Vec3f(GVec3<f32>),
    Vec4f(GVec4<f32>),
    Vec2u(GVec2<u32>),
    Vec3u(GVec3<u32>),
    Vec4u(GVec4<u32>),
    Vec2i(GVec2<i32>),
    Vec3i(GVec3<i32>),
    Vec4i(GVec4<i32>),
    Mat4x4(GMat4x4<f32>),
}

impl ShaderBufferParameter {
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
        self.get_type().align()
    }

    /// Size on the GPU of this data type. Also corresponds to the size on the CPU
    ///
    /// Taken from [the WebGPU specification](https://www.w3.org/TR/WGSL/#alignment-and-size)
    #[inline]
    pub const fn size(&self) -> usize {
        self.get_type().size()
    }

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

    #[inline]
    pub fn set_from(&mut self, value: MaterialParameter) -> bool {
        //TODO: Implement automatic conversions
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
            Self::Vec4f(cur) => {
                if let MaterialParameter::Vec4(v) = value {
                    *cur = v.into();
                    return true;
                }
            }
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

#[derive(
    Debug,
    Clone,
    derive_more::IsVariant,
    derive_more::Unwrap,
    derive_more::TryUnwrap,
    derive_more::From,
)]
pub enum ShaderOpaqueParameter {
    Texture2D(wgpu::TextureView),
    Sampler(wgpu::Sampler),
}

impl ShaderOpaqueParameter {
    #[inline]
    pub fn set_from(&mut self, _value: MaterialParameter) -> bool {
        todo!()
    }

    #[inline]
    pub(crate) fn to_binding_resource(&self) -> wgpu::BindingResource<'_> {
        match self {
            Self::Texture2D(texture_view) => wgpu::BindingResource::TextureView(texture_view),
            Self::Sampler(sampler) => wgpu::BindingResource::Sampler(sampler),
        }
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
