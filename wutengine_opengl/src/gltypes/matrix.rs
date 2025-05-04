use bytemuck::{Pod, Zeroable};
use glam::Mat4;

use super::GlVec4f;
use super::array::GlArray;

/// A 4x4 OpenGL float matrix
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct GlMat4f {
    pub cols: GlArray<GlVec4f, 4>,
}

unsafe impl Zeroable for GlMat4f {}
unsafe impl Pod for GlMat4f {}

impl From<Mat4> for GlMat4f {
    #[inline]
    fn from(value: Mat4) -> Self {
        Self {
            cols: [
                value.x_axis.into(),
                value.y_axis.into(),
                value.z_axis.into(),
                value.w_axis.into(),
            ]
            .into(),
        }
    }
}
