use core::fmt::Display;

use serde::{Deserialize, Serialize};
use wutengine_graphics::color::Color;
use wutengine_windowing::window::WindowIdentifier;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CameraTarget {
    Window(WindowIdentifier),
    //Texture(TODO)
}

/// The different types of possible camera projections.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CameraProjection {
    /// Perspective-projecting camera.
    Perspective(FieldOfView),

    /// Orthographic-projecting camera. Value defines vertical viewing volume.
    /// Horizontal volume is determined through aspect ratio
    Orthographic(f32),
}

/// Field-of-view definition for a [CameraProjection]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FieldOfView {
    /// Vertical degrees
    Vertical(f32),

    /// Horizontal degrees
    Horizontal(f32),
}

impl FieldOfView {
    pub fn get_vertical(self, aspect_ratio: f32) -> f32 {
        match self {
            FieldOfView::Vertical(vfov) => vfov,
            FieldOfView::Horizontal(hfov) => {
                let h_rad = hfov.to_radians();

                let vfov_rad = 2.0 * f32::atan(f32::tan(h_rad * 0.5) * (1.0 / aspect_ratio));

                vfov_rad.to_degrees()
            }
        }
    }

    pub fn get_horizontal(self, aspect_ratio: f32) -> f32 {
        match self {
            FieldOfView::Vertical(vfov) => {
                let v_rad = vfov.to_radians();

                let hfov_rad = 2.0 * f32::atan(f32::tan(v_rad * 0.5) * aspect_ratio);

                hfov_rad.to_degrees()
            }
            FieldOfView::Horizontal(hfov) => hfov,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CameraBackground {
    None,
    Color(Color),
}

/// The configuration for the viewport of a camera
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
