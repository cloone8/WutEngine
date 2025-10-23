use serde::{Deserialize, Serialize};
use wutengine_graphics::wgpu;

/// The internal camera color texture format used while rendering. A subset of the
/// [wgpu::TextureFormat] enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CameraColorFormat {
    /// Red channel only. 8 bit integer per channel. [0, 255] converted to/from float [0, 1] in shader.
    R8Unorm,

    /// Red channel only. 8 bit integer per channel. [&minus;127, 127] converted to/from float [&minus;1, 1] in shader.
    R8Snorm,

    /// Red channel only. 16 bit float per channel. Float in shader.
    R16Float,

    /// Red channel only. 32 bit float per channel. Float in shader.
    R32Float,

    /// Red and green channels. 32 bit float per channel. Float in shader.
    Rg32Float,

    /// Red, green, blue, and alpha channels. 8 bit integer per channel. [0, 255] converted to/from float [0, 1] in shader.
    Rgba8Unorm,

    /// Red, green, blue, and alpha channels. 8 bit integer per channel. [&minus;127, 127] converted to/from float [&minus;1, 1] in shader.
    Rgba8Snorm,

    /// Red, green, blue, and alpha channels. 16 bit float per channel. Float in shader.
    Rgba16Float,

    /// Red, green, blue, and alpha channels. 32 bit float per channel. Float in shader.
    Rgba32Float,
}

impl From<CameraColorFormat> for wgpu::TextureFormat {
    #[inline(always)]
    fn from(value: CameraColorFormat) -> Self {
        match value {
            CameraColorFormat::R8Unorm => wgpu::TextureFormat::R8Unorm,
            CameraColorFormat::R8Snorm => wgpu::TextureFormat::R8Snorm,
            CameraColorFormat::R16Float => wgpu::TextureFormat::R16Float,
            CameraColorFormat::R32Float => wgpu::TextureFormat::R32Float,
            CameraColorFormat::Rg32Float => wgpu::TextureFormat::Rg32Float,
            CameraColorFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
            CameraColorFormat::Rgba8Snorm => wgpu::TextureFormat::Rgba8Snorm,
            CameraColorFormat::Rgba16Float => wgpu::TextureFormat::Rgba16Float,
            CameraColorFormat::Rgba32Float => wgpu::TextureFormat::Rgba32Float,
        }
    }
}

/// The internal camera depth texture format used while rendering. A subset of the
/// [wgpu::TextureFormat] enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CameraDepthStencilFormat {
    /// Special depth format with 16 bit integer depth.
    Depth16Unorm,

    /// Special depth format with at least 24 bit integer depth.
    Depth24Plus,

    /// Stencil format with 8 bit integer stencil.
    Stencil8,

    /// Special depth/stencil format with at least 24 bit integer depth and 8 bits integer stencil.
    Depth24PlusStencil8,

    /// Special depth format with 32 bit floating point depth.
    Depth32Float,
}

impl From<CameraDepthStencilFormat> for wgpu::TextureFormat {
    #[inline(always)]
    fn from(value: CameraDepthStencilFormat) -> Self {
        match value {
            CameraDepthStencilFormat::Depth16Unorm => wgpu::TextureFormat::Depth16Unorm,
            CameraDepthStencilFormat::Depth24Plus => wgpu::TextureFormat::Depth24Plus,
            CameraDepthStencilFormat::Stencil8 => wgpu::TextureFormat::Stencil8,
            CameraDepthStencilFormat::Depth24PlusStencil8 => {
                wgpu::TextureFormat::Depth24PlusStencil8
            }
            CameraDepthStencilFormat::Depth32Float => wgpu::TextureFormat::Depth32Float,
        }
    }
}
