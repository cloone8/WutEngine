use glam::Mat4;

use super::GlVec4f;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct GlMat4f {
    pub x_col: GlVec4f,
    pub y_col: GlVec4f,
    pub z_col: GlVec4f,
    pub w_col: GlVec4f,
}

impl From<Mat4> for GlMat4f {
    fn from(value: Mat4) -> Self {
        Self {
            x_col: value.x_axis.into(),
            y_col: value.y_axis.into(),
            z_col: value.z_axis.into(),
            w_col: value.w_axis.into(),
        }
    }
}
