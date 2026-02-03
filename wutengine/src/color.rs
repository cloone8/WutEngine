//! Color related types and functions

use core::fmt::{Debug, Display};

use crate::math::{Vec3, Vec3A, Vec4};

/// A 32-bit-per-color RGBA color
#[derive(Clone, Copy, PartialEq)]
pub struct Color(Vec4);

impl Color {
    /// Creates a new color from the given components
    #[inline(always)]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self(Vec4::new(r, g, b, a))
    }

    /// The red component
    #[inline(always)]
    pub const fn r(self) -> f32 {
        self.0.to_array()[0]
    }

    /// The green component
    #[inline(always)]
    pub const fn g(self) -> f32 {
        self.0.to_array()[1]
    }

    /// The blue component
    #[inline(always)]
    pub const fn b(self) -> f32 {
        self.0.to_array()[2]
    }

    /// The alpha component
    #[inline(always)]
    pub const fn a(self) -> f32 {
        self.0.to_array()[3]
    }

    /// Returns this color as a [Vec4]
    #[inline(always)]
    pub const fn as_vec4(self) -> Vec4 {
        self.0
    }
}

/// Default colors
impl Color {
    /// All zero
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0, 0.0);

    /// All one
    pub const ONE: Self = Self::new(1.0, 1.0, 1.0, 1.0);

    /// Fully black, all zero except for a 1.0 alpha
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);

    /// Fully white, all one
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);

    /// All zero with red and alpha set to 1.0
    pub const RED: Self = Self::new(1.0, 0.0, 0.0, 1.0);

    /// All zero with green and alpha set to 1.0
    pub const GREEN: Self = Self::new(0.0, 1.0, 0.0, 1.0);

    /// All zero with blue and alpha set to 1.0
    pub const BLUE: Self = Self::new(0.0, 0.0, 1.0, 1.0);
}

impl Display for Color {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Color(r: {}, g: {}, b: {}, a: {})",
            self.r(),
            self.g(),
            self.b(),
            self.a()
        )
    }
}

impl Debug for Color {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Color")
            .field(&self.0.x)
            .field(&self.0.y)
            .field(&self.0.z)
            .field(&self.0.w)
            .finish()
    }
}

impl From<Vec3A> for Color {
    #[inline]
    fn from(value: Vec3A) -> Self {
        let as_array = value.to_array();

        Self(Vec4::new(as_array[0], as_array[1], as_array[2], 1.0))
    }
}

impl From<Vec3> for Color {
    #[inline]
    fn from(value: Vec3) -> Self {
        Self(Vec4::new(value.x, value.y, value.z, 1.0))
    }
}

impl From<Vec4> for Color {
    #[inline(always)]
    fn from(value: Vec4) -> Self {
        Self(value)
    }
}

impl From<Color> for wgpu::Color {
    #[inline]
    fn from(value: Color) -> Self {
        Self {
            r: value.r() as f64,
            g: value.g() as f64,
            b: value.b() as f64,
            a: value.a() as f64,
        }
    }
}
