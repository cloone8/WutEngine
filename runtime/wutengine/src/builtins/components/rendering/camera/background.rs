use wutengine_graphics::wgpu;
use wutengine_math::Color;

use crate::graphics;

/// The background of the [`super::Camera`] viewport
#[derive(Debug, Clone, Copy)]
pub enum CameraBackground {
    /// No specific background. Probably contains the contents of the previous frame
    None,

    /// A specific background color
    Color(Color),
}

impl CameraBackground {
    /// Converts this background config to a [`wgpu::LoadOp`]
    pub fn to_wgpu_load_op(self) -> wgpu::LoadOp<wgpu::Color> {
        match self {
            Self::None => wgpu::LoadOp::Load,
            Self::Color(color) => wgpu::LoadOp::Clear(graphics::to_wgpu_color(color)),
        }
    }
}
