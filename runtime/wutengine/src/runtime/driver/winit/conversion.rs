//! [`winit`] conversions, gated by the `interactive` feature

pub(super) use winit::dpi::LogicalSize as WinitLogicalSize;
pub(super) use winit::dpi::PhysicalSize as WinitPhysicalSize;

use crate::window::LogicalSize;
use crate::window::PixelSize;

#[inline]
#[must_use]
pub(super) const fn pixel_size_from_winit(value: WinitPhysicalSize<u32>) -> PixelSize {
    PixelSize {
        width: value.width,
        height: value.height,
    }
}

#[inline]
#[must_use]
#[expect(unused, reason = "Will probably be used later")]
pub(super) const fn pixel_size_to_winit(value: PixelSize) -> WinitPhysicalSize<u32> {
    WinitPhysicalSize {
        width: value.width,
        height: value.height,
    }
}

#[inline]
#[must_use]
#[expect(unused, reason = "Will probably be used later")]
pub(super) const fn logical_size_from_winit(value: WinitLogicalSize<f64>) -> LogicalSize {
    LogicalSize {
        width: value.width,
        height: value.height,
    }
}

#[inline]
#[must_use]
pub(super) const fn logical_size_to_winit(value: LogicalSize) -> WinitLogicalSize<f64> {
    WinitLogicalSize {
        width: value.width,
        height: value.height,
    }
}
