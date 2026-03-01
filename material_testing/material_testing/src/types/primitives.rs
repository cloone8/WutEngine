use glam::{Mat4, Vec2, Vec3, Vec4};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Padding<const N: usize>([u8; N]);

unsafe impl<const N: usize> bytemuck::Zeroable for Padding<N> {}
unsafe impl<const N: usize> bytemuck::Pod for Padding<N> {}

impl<const N: usize> Padding<N> {
    #[inline(always)]
    pub const fn new() -> Self {
        Self([0; N])
    }
}

impl<const N: usize> Default for Padding<N> {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct GVec2<T>([T; 2]);

unsafe impl bytemuck::Zeroable for GVec2<i32> {}
unsafe impl bytemuck::Pod for GVec2<i32> {}

unsafe impl bytemuck::Zeroable for GVec2<u32> {}
unsafe impl bytemuck::Pod for GVec2<u32> {}

unsafe impl bytemuck::Zeroable for GVec2<f32> {}
unsafe impl bytemuck::Pod for GVec2<f32> {}

impl From<Vec2> for GVec2<f32> {
    #[inline(always)]
    fn from(value: Vec2) -> Self {
        Self(value.to_array())
    }
}

impl GVec2<f32> {
    pub const ALIGN: usize = 8;
}

impl GVec2<u32> {
    pub const ALIGN: usize = 8;
}

impl GVec2<i32> {
    pub const ALIGN: usize = 8;
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct GVec3<T>([T; 3]);

unsafe impl bytemuck::Zeroable for GVec3<i32> {}
unsafe impl bytemuck::Pod for GVec3<i32> {}

unsafe impl bytemuck::Zeroable for GVec3<u32> {}
unsafe impl bytemuck::Pod for GVec3<u32> {}

unsafe impl bytemuck::Zeroable for GVec3<f32> {}
unsafe impl bytemuck::Pod for GVec3<f32> {}

impl From<Vec3> for GVec3<f32> {
    #[inline(always)]
    fn from(value: Vec3) -> Self {
        Self(value.to_array())
    }
}

impl GVec3<f32> {
    pub const ALIGN: usize = 16;
}

impl GVec3<u32> {
    pub const ALIGN: usize = 16;
}

impl GVec3<i32> {
    pub const ALIGN: usize = 16;
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct GVec4<T>([T; 4]);

unsafe impl bytemuck::Zeroable for GVec4<i32> {}
unsafe impl bytemuck::Pod for GVec4<i32> {}

unsafe impl bytemuck::Zeroable for GVec4<u32> {}
unsafe impl bytemuck::Pod for GVec4<u32> {}

unsafe impl bytemuck::Zeroable for GVec4<f32> {}
unsafe impl bytemuck::Pod for GVec4<f32> {}

impl From<Vec4> for GVec4<f32> {
    #[inline(always)]
    fn from(value: Vec4) -> Self {
        Self(value.to_array())
    }
}

impl From<[f32; 4]> for GVec4<f32> {
    #[inline(always)]
    fn from(value: [f32; 4]) -> Self {
        Self(value)
    }
}

impl GVec4<f32> {
    pub const ALIGN: usize = 16;
}

impl GVec4<u32> {
    pub const ALIGN: usize = 16;
}

impl GVec4<i32> {
    pub const ALIGN: usize = 16;
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct GMat4x4<T>([GVec4<T>; 4]);

unsafe impl bytemuck::Zeroable for GMat4x4<f32> {}
unsafe impl bytemuck::Pod for GMat4x4<f32> {}

impl GMat4x4<f32> {
    pub const ALIGN: usize = GVec4::<f32>::ALIGN;
}

impl From<Mat4> for GMat4x4<f32> {
    #[inline]
    fn from(value: Mat4) -> Self {
        let cols = value.to_cols_array_2d();
        Self([
            cols[0].into(),
            cols[1].into(),
            cols[2].into(),
            cols[3].into(),
        ])
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct GMat3x4<T>([GVec4<T>; 3]);

unsafe impl bytemuck::Zeroable for GMat3x4<f32> {}
unsafe impl bytemuck::Pod for GMat3x4<f32> {}

impl GMat3x4<f32> {
    pub const ALIGN: usize = GVec4::<f32>::ALIGN;
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct GMat4x3<T>([(GVec3<T>, Padding<1>); 4]);

unsafe impl bytemuck::Zeroable for GMat4x3<f32> {}
unsafe impl bytemuck::Pod for GMat4x3<f32> {}

impl GMat4x3<f32> {
    pub const ALIGN: usize = GVec3::<f32>::ALIGN;
}
