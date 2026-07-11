use core::fmt::Display;

/// The configuration for the viewport of a [Camera]
#[derive(Debug, Clone, Copy)]
pub struct CameraViewport {
    /// Location of the left side of the viewport, as expressed as a fraction of the window. From 0.0-1.0
    pub x: f32,

    /// Location of the bottom of the viewport, as expressed as a fraction of the window. From 0.0-1.0
    pub y: f32,

    /// Width of the viewport, as expressed as a fraction of the window. From 0.0-1.0
    pub w: f32,

    /// Height viewport, as expressed as a fraction of the window. From 0.0-1.0
    pub h: f32,
}

impl CameraViewport {
    /// Camera viewport representing an entire window
    pub const FULL_WINDOW: Self = Self {
        x: 0.0,
        y: 0.0,
        w: 1.0,
        h: 1.0,
    };

    /// Checks that the viewport is configured to valid values
    pub const fn is_valid(&self) -> bool {
        self.x >= 0.0
            && self.x < 1.0
            && self.y >= 0.0
            && self.y < 1.0
            && self.w > 0.0
            && self.w <= 1.0
            && self.h > 0.0
            && self.h <= 1.0
    }

    /// Given a full window size, returns the size that this viewport would take,
    /// not accounting for any viewport areas that are cut off due to viewport positioning
    pub const fn scale_size(self, full_size: (u32, u32)) -> (u32, u32) {
        (
            (self.w * (full_size.0 as f32)) as u32,
            (self.h * (full_size.1 as f32)) as u32,
        )
    }
}

impl Default for CameraViewport {
    fn default() -> Self {
        Self::FULL_WINDOW
    }
}

impl Display for CameraViewport {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Viewport(offset=({}, {}), dimensions=({}, {}))",
            self.x, self.y, self.w, self.h
        )
    }
}
