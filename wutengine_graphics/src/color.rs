//! Color datatypes and conversions

use glam::{Vec3, Vec4};

/// RGBA color consisting of one 32-bit float per component
#[derive(Debug, Clone, Copy)]
pub struct Color {
    /// Red value
    pub r: f32,

    /// Green value
    pub g: f32,

    /// Blue value
    pub b: f32,

    /// Alpha value
    pub a: f32,
}

impl Color {
    /// Black (0,0,0)
    pub const BLACK: Self = Color::rgb(0.0, 0.0, 0.0);

    /// White (1,1,1)
    pub const WHITE: Self = Color::rgb(1.0, 1.0, 1.0);

    /// Red (1,0,0)
    pub const RED: Self = Color::rgb(1.0, 0.0, 0.0);

    /// Green (0,1,0)
    pub const GREEN: Self = Color::rgb(0.0, 1.0, 0.0);

    /// Blue (0,0,1)
    pub const BLUE: Self = Color::rgb(0.0, 0.0, 1.0);

    /// Construct a new color from three R/G/B components.
    /// Alpha is set to 1.0
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Construct a new color from four R/G/B/A components.
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

impl From<Vec3> for Color {
    fn from(value: Vec3) -> Self {
        Self::rgb(value.x, value.y, value.z)
    }
}

impl From<Vec4> for Color {
    fn from(value: Vec4) -> Self {
        Self::rgba(value.x, value.y, value.z, value.w)
    }
}
