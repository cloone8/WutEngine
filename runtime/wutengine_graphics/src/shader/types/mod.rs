//! Types supported by WGSL shaders

mod primitives;

pub use primitives::*;
use wutengine_assets::assets::shader::BufferBaseType;
use wutengine_assets::assets::shader::ShaderVertexAttributeType;
use wutengine_util_macro::VariantName;

use crate::material::MaterialParameter;
use wutengine_math::Vec4;

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
    /// Returns a zeroed parameter of the given type, or [`None`] if materials can't set that type
    pub(crate) const fn zeroed(ty: BufferBaseType) -> Option<Self> {
        Some(match ty {
            BufferBaseType::Float => Self::Flt(bytemuck::zeroed()),
            BufferBaseType::Uint => Self::Uint(bytemuck::zeroed()),
            BufferBaseType::Sint => Self::Int(bytemuck::zeroed()),
            BufferBaseType::Vec2f => Self::Vec2f(bytemuck::zeroed()),
            BufferBaseType::Vec3f => Self::Vec3f(bytemuck::zeroed()),
            BufferBaseType::Vec4f => Self::Vec4f(bytemuck::zeroed()),
            BufferBaseType::Vec2u => Self::Vec2u(bytemuck::zeroed()),
            BufferBaseType::Vec3u => Self::Vec3u(bytemuck::zeroed()),
            BufferBaseType::Vec4u => Self::Vec4u(bytemuck::zeroed()),
            BufferBaseType::Vec2i => Self::Vec2i(bytemuck::zeroed()),
            BufferBaseType::Vec3i => Self::Vec3i(bytemuck::zeroed()),
            BufferBaseType::Vec4i => Self::Vec4i(bytemuck::zeroed()),
            BufferBaseType::Mat4x4 => Self::Mat4x4(bytemuck::zeroed()),
            _ => return None,
        })
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

    /// Sets the value of this parameter from an external [`MaterialParameter`], casting
    /// if possible. Will not change the type of this [`ShaderBufferParameter`]
    #[inline]
    #[expect(clippy::todo, reason = "Casting is a lot of work")]
    pub fn set_from(&mut self, value: &MaterialParameter) -> bool {
        match self {
            Self::Flt(cur) => {
                if let MaterialParameter::Flt(f) = value {
                    *cur = *f;
                    return true;
                }
            }
            Self::Uint(cur) => {
                if let MaterialParameter::Uint(u) = value {
                    *cur = *u;
                    return true;
                }
            }
            Self::Int(cur) => {
                if let MaterialParameter::Int(i) = value {
                    *cur = *i;
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
    /// Updates the value of this [`ShaderOpaqueParameter`] from the given [`MaterialParameter`]
    #[inline]
    pub fn set_from(&mut self, value: MaterialParameter) -> bool {
        //TODO: Add error handling for not-yet-loaded assets?
        match self {
            Self::Texture2D(cur) => {
                if let MaterialParameter::Texture2D(tex) = value {
                    *cur = tex.get_view().clone();
                    return true;
                }
            }
            Self::Sampler(cur) => {
                if let MaterialParameter::Sampler(smp) = value {
                    *cur = smp.get_wgpu().clone();
                    return true;
                }
            }
        }

        false
    }

    /// Returns the [`wgpu::BindingResource`] corresponding to this parameter
    #[inline]
    pub(crate) fn to_binding_resource(&self) -> wgpu::BindingResource<'_> {
        match self {
            Self::Texture2D(texture_view) => wgpu::BindingResource::TextureView(texture_view),
            Self::Sampler(sampler) => wgpu::BindingResource::Sampler(sampler),
        }
    }
}

/// Returns the [`wgpu::VertexFormat`] corresponding to this [`ShaderVertexAttributeType`]
pub const fn shader_attr_wgpu_vertex_format(attr: ShaderVertexAttributeType) -> wgpu::VertexFormat {
    match attr {
        ShaderVertexAttributeType::Position => wgpu::VertexFormat::Float32x3,
        ShaderVertexAttributeType::Normal => wgpu::VertexFormat::Float32x3,
        ShaderVertexAttributeType::Uv { .. } => wgpu::VertexFormat::Float32x2,
        ShaderVertexAttributeType::Color => wgpu::VertexFormat::Float32x4,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Test primitive sizes and alignments according to the [`WebGPU spec`](https://www.w3.org/TR/WGSL/#alignment-and-size)
    #[test]
    fn size_align_primitives() {
        assert_eq!(4, size_of::<f32>());
        assert_eq!(4, align_of::<f32>());
        assert_eq!(4, size_of::<u32>());
        assert_eq!(4, align_of::<u32>());
        assert_eq!(4, size_of::<i32>());
        assert_eq!(4, align_of::<i32>());
    }

    /// Test vector sizes and alignments according to the [`WebGPU spec`](https://www.w3.org/TR/WGSL/#alignment-and-size)
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

    /// Test matrix sizes and alignments according to the [`WebGPU spec`](https://www.w3.org/TR/WGSL/#alignment-and-size)
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
