//! Types supported by WGSL shaders

use glam::{Mat4, Vec2, Vec3, Vec4};

mod primitives;

pub use primitives::*;

use crate::graphics::material::MaterialParameter;

#[derive(
    Debug,
    Clone,
    Copy,
    derive_more::IsVariant,
    derive_more::Unwrap,
    derive_more::TryUnwrap,
    derive_more::From,
)]
pub enum NativeShaderBufferParameter {
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

impl NativeShaderBufferParameter {
    /// Alignment on the GPU of this data type.
    ///
    /// Taken from [the WebGPU specification](https://www.w3.org/TR/WGSL/#alignment-and-size)
    #[inline]
    pub const fn align(&self) -> usize {
        match self {
            Self::Flt(_) => 4,
            Self::Uint(_) => 4,
            Self::Int(_) => 4,
            Self::Vec2f(_) => 8,
            Self::Vec3f(_) => 16,
            Self::Vec4f(_) => 16,
            Self::Vec2u(_) => 8,
            Self::Vec3u(_) => 16,
            Self::Vec4u(_) => 16,
            Self::Vec2i(_) => 8,
            Self::Vec3i(_) => 16,
            Self::Vec4i(_) => 16,
            Self::Mat4x4(_) => 16,
        }
    }

    /// Size on the GPU of this data type. Also corresponds to the size on the CPU
    ///
    /// Taken from [the WebGPU specification](https://www.w3.org/TR/WGSL/#alignment-and-size)
    #[inline]
    pub const fn size(&self) -> usize {
        match self {
            Self::Flt(_) => size_of::<f32>(),
            Self::Uint(_) => size_of::<u32>(),
            Self::Int(_) => size_of::<i32>(),
            Self::Vec2f(_) => size_of::<GVec2<f32>>(),
            Self::Vec3f(_) => size_of::<GVec3<f32>>(),
            Self::Vec4f(_) => size_of::<GVec4<f32>>(),
            Self::Vec2u(_) => size_of::<GVec2<u32>>(),
            Self::Vec3u(_) => size_of::<GVec3<u32>>(),
            Self::Vec4u(_) => size_of::<GVec4<u32>>(),
            Self::Vec2i(_) => size_of::<GVec2<i32>>(),
            Self::Vec3i(_) => size_of::<GVec3<i32>>(),
            Self::Vec4i(_) => size_of::<GVec4<i32>>(),
            Self::Mat4x4(_) => size_of::<GMat4x4<f32>>(),
        }
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
                    true
                } else {
                    false
                }
            }
            Self::Uint(cur) => {
                if let MaterialParameter::Uint(u) = value {
                    *cur = u;
                    true
                } else {
                    false
                }
            }
            Self::Int(cur) => {
                if let MaterialParameter::Int(i) = value {
                    *cur = i;
                    true
                } else {
                    false
                }
            }
            Self::Vec2f(cur) => {
                if let MaterialParameter::Vec2(v) = value {
                    *cur = v.into();
                    true
                } else {
                    false
                }
            }
            Self::Vec3f(cur) => {
                if let MaterialParameter::Vec3(v) = value {
                    *cur = v.into();
                    true
                } else {
                    false
                }
            }
            Self::Vec4f(cur) => {
                if let MaterialParameter::Vec4(v) = value {
                    *cur = v.into();
                    true
                } else {
                    false
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
                    true
                } else {
                    false
                }
            }
        }
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
pub enum NativeShaderOpaqueParameter {
    Texture2D(wgpu::TextureView),
    Sampler(wgpu::Sampler),
}

impl NativeShaderOpaqueParameter {
    #[inline]
    pub fn set_from(&mut self, _value: MaterialParameter) -> bool {
        todo!()
    }

    #[inline]
    pub(crate) fn to_binding_resource(&self) -> wgpu::BindingResource {
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
