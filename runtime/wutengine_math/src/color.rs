//! Color related types and functions

use core::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};

use crate::{Vec3, Vec3A, Vec4};

#[derive(Clone, Copy, Serialize, Deserialize, Default)]
#[serde(default)]
struct SerializedColor {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl From<Color> for SerializedColor {
    #[inline]
    fn from(value: Color) -> Self {
        Self {
            r: value.r(),
            g: value.g(),
            b: value.b(),
            a: value.a(),
        }
    }
}

impl From<SerializedColor> for Color {
    #[inline]
    fn from(value: SerializedColor) -> Self {
        Self::new(value.r, value.g, value.b, value.a)
    }
}

/// A 32-bit-per-color RGBA color
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, bytemuck::NoUninit)]
#[serde(into = "SerializedColor", from = "SerializedColor")]
#[repr(transparent)]
pub struct Color(Vec4);

impl Color {
    /// Creates a new color from the given components
    #[inline]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self(Vec4::new(r, g, b, a))
    }

    /// Creates a new color from the given bytes. `0u8` maps to `0.0f32`, `u8::MAX` maps to `1.0f32`
    #[inline]
    pub const fn new_from_bytes(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::new(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        )
    }

    /// Creates a new color from a hex code. If the hex code is invalid, returns [`None`]
    pub fn hex(hex: &str) -> Option<Self> {
        if !hex.is_ascii() {
            return None;
        }

        let hex = match hex.strip_prefix("#") {
            Some(stripped) => stripped,
            None => hex,
        };

        let trimmed = hex.trim();

        let (num_hex_chars, hex_char_len) = match trimmed.len() {
            3 | 4 => {
                // Parse as short hex, one char per byte

                (trimmed.len(), 1usize)
            }
            6 | 8 => {
                // Parse as normal hex, two chars per byte

                (trimmed.len() / 2, 2usize)
            }
            _ => {
                // Invalid length
                return None;
            }
        };

        let mut rgba: [u8; 4] = [u8::MAX; 4];

        for i in 0..num_hex_chars {
            let hex_char = &trimmed[(i * hex_char_len)..((i + 1) * hex_char_len)];

            rgba[i] = u8::from_str_radix(hex_char, 16).ok()?;
        }

        Some(Self::from(rgba))
    }

    /// The red component
    #[inline]
    pub const fn r(self) -> f32 {
        self.0.to_array()[0]
    }

    /// The green component
    #[inline]
    pub const fn g(self) -> f32 {
        self.0.to_array()[1]
    }

    /// The blue component
    #[inline]
    pub const fn b(self) -> f32 {
        self.0.to_array()[2]
    }

    /// The alpha component
    #[inline]
    pub const fn a(self) -> f32 {
        self.0.to_array()[3]
    }

    /// Returns this color as a [`Vec4`]
    #[inline]
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
    #[inline]
    fn from(value: Vec4) -> Self {
        Self(value)
    }
}

impl From<(f32, f32, f32, f32)> for Color {
    #[inline]
    fn from(value: (f32, f32, f32, f32)) -> Self {
        Self::new(value.0, value.1, value.2, value.3)
    }
}

impl From<(f32, f32, f32)> for Color {
    #[inline]
    fn from(value: (f32, f32, f32)) -> Self {
        Self::new(value.0, value.1, value.2, 1.0)
    }
}

impl From<&[u8; 4]> for Color {
    #[inline]
    fn from(value: &[u8; 4]) -> Self {
        Self::new_from_bytes(value[0], value[1], value[2], value[3])
    }
}
impl From<[u8; 4]> for Color {
    #[inline]
    fn from(value: [u8; 4]) -> Self {
        Self::new_from_bytes(value[0], value[1], value[2], value[3])
    }
}

impl From<&[u8; 3]> for Color {
    #[inline]
    fn from(value: &[u8; 3]) -> Self {
        Self::new_from_bytes(value[0], value[1], value[2], u8::MAX)
    }
}

impl From<[u8; 3]> for Color {
    #[inline]
    fn from(value: [u8; 3]) -> Self {
        Self::new_from_bytes(value[0], value[1], value[2], u8::MAX)
    }
}

impl From<(u8, u8, u8, u8)> for Color {
    #[inline]
    fn from(value: (u8, u8, u8, u8)) -> Self {
        Self::new_from_bytes(value.0, value.1, value.2, value.3)
    }
}

impl From<(u8, u8, u8)> for Color {
    #[inline]
    fn from(value: (u8, u8, u8)) -> Self {
        Self::new_from_bytes(value.0, value.1, value.2, u8::MAX)
    }
}
