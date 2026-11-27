/// A physical size in pixels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PixelSize {
    /// Width
    pub width: u32,

    /// Height
    pub height: u32,
}

impl PixelSize {
    /// Converts a logical size to a physical one by applying a scale factor.
    /// Each dimension is rounded to the nearest pixel
    #[must_use]
    #[inline]
    pub const fn from_logical(logical: LogicalSize, scale_factor: f64) -> Self {
        debug_assert!(
            scale_factor.is_normal() && scale_factor.is_sign_positive(),
            "Scale factor must be a positive, normal number"
        );

        Self {
            width: (logical.width * scale_factor).round() as u32,
            height: (logical.height * scale_factor).round() as u32,
        }
    }

    /// Converts this physical size to a logical one by removing a scale factor.
    /// See [`LogicalSize::from_pixels`]
    #[must_use]
    #[inline]
    pub const fn to_logical(self, scale_factor: f64) -> LogicalSize {
        LogicalSize::from_pixels(self, scale_factor)
    }
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

/// A logical size, in "points".
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct LogicalSize {
    /// Width
    pub width: f64,

    /// Height
    pub height: f64,
}

impl LogicalSize {
    /// Converts a physical size to a logical one by removing a scale factor
    #[must_use]
    #[inline]
    pub const fn from_pixels(pixels: PixelSize, scale_factor: f64) -> Self {
        debug_assert!(
            scale_factor.is_normal() && scale_factor.is_sign_positive(),
            "Scale factor must be a positive, normal number"
        );

        Self {
            width: pixels.width as f64 / scale_factor,
            height: pixels.height as f64 / scale_factor,
        }
    }

    /// Converts this logical size to a physical one by applying a scale factor.
    /// See [`PixelSize::from_logical`]
    #[must_use]
    #[inline]
    pub const fn to_pixels(self, scale_factor: f64) -> PixelSize {
        PixelSize::from_logical(self, scale_factor)
    }
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
