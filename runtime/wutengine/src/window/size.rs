use winit::dpi::LogicalSize as WinitLogicalSize;
use winit::dpi::PhysicalSize as WinitPhysicalSize;

/// A physical size in pixels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PixelSize {
    /// Width
    pub width: u32,

    /// Height
    pub height: u32,
}

impl From<(u32, u32)> for PixelSize {
    #[inline]
    fn from(value: (u32, u32)) -> Self {
        Self {
            width: value.0,
            height: value.1,
        }
    }
}

impl From<PixelSize> for (u32, u32) {
    #[inline]
    fn from(value: PixelSize) -> Self {
        (value.width, value.height)
    }
}

impl From<WinitPhysicalSize<u32>> for PixelSize {
    #[inline]
    fn from(value: WinitPhysicalSize<u32>) -> Self {
        Self {
            width: value.width,
            height: value.height,
        }
    }
}

impl From<PixelSize> for WinitPhysicalSize<u32> {
    #[inline]
    fn from(value: PixelSize) -> Self {
        Self {
            width: value.width,
            height: value.height,
        }
    }
}

/// A logical size, in "points".
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct LogicalSize {
    /// Width
    pub width: f64,

    /// Height
    pub height: f64,
}

impl From<(f64, f64)> for LogicalSize {
    #[inline]
    fn from(value: (f64, f64)) -> Self {
        Self {
            width: value.0,
            height: value.1,
        }
    }
}

impl From<LogicalSize> for (f64, f64) {
    #[inline]
    fn from(value: LogicalSize) -> Self {
        (value.width, value.height)
    }
}

impl From<WinitLogicalSize<f64>> for LogicalSize {
    #[inline]
    fn from(value: WinitLogicalSize<f64>) -> Self {
        Self {
            width: value.width,
            height: value.height,
        }
    }
}

impl From<LogicalSize> for WinitLogicalSize<f64> {
    #[inline]
    fn from(value: LogicalSize) -> Self {
        Self {
            width: value.width,
            height: value.height,
        }
    }
}
