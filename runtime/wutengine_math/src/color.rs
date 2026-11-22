//! Color related types and functions

use core::fmt::Debug;
use core::fmt::Display;
use core::fmt::Formatter;
use core::fmt::Result;

use glam::Vec4Swizzles;

use crate::Vec3;
use crate::Vec3A;
use crate::Vec4;

#[derive(Clone, Copy, Default)]
// #[serde(default)]
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
#[derive(Clone, Copy, PartialEq)]
// #[serde(into = "SerializedColor", from = "SerializedColor")]
#[repr(transparent)]
pub struct Color(Vec4);

impl Color {
    /// Creates a new color from the given components
    #[inline]
    #[must_use]
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self(Vec4::new(r, g, b, a))
    }

    /// Creates a new color from the given bytes. `0u8` maps to `0.0f32`, `u8::MAX` maps to `1.0f32`
    #[inline]
    #[must_use]
    pub const fn new_from_bytes(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::new(
            r as f32 / 255.0,
            g as f32 / 255.0,
            b as f32 / 255.0,
            a as f32 / 255.0,
        )
    }

    /// Creates a new color from a hex code. If the hex code is invalid, returns [`None`]
    #[must_use]
    pub fn hex(hex: &str) -> Option<Self> {
        if !hex.is_ascii() {
            return None;
        }

        // Trim before removing the `#`, so whitespace on either side of the whole code is allowed
        let hex = hex.trim();

        let trimmed = match hex.strip_prefix("#") {
            Some(stripped) => stripped,
            None => hex,
        };

        let (num_hex_chars, hex_char_len) = match trimmed.len() {
            3 | 4 => {
                // Parse as short hex, one char per byte, doubled: "F0A" means "FF00AA"

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

            let value = u8::from_str_radix(hex_char, 16).ok()?;

            // Doubling a single hex digit is the same as multiplying it by 0x11: 0xF becomes 0xFF
            rgba[i] = if hex_char_len == 1 {
                value * 0x11
            } else {
                value
            };
        }

        Some(Self::from(rgba))
    }

    /// The red component
    #[inline]
    #[must_use]
    pub const fn r(self) -> f32 {
        self.0.to_array()[0]
    }

    /// The green component
    #[inline]
    #[must_use]
    pub const fn g(self) -> f32 {
        self.0.to_array()[1]
    }

    /// The blue component
    #[inline]
    #[must_use]
    pub const fn b(self) -> f32 {
        self.0.to_array()[2]
    }

    /// The alpha component
    #[inline]
    #[must_use]
    pub const fn a(self) -> f32 {
        self.0.to_array()[3]
    }

    /// Returns this color as a [`Vec4`]
    #[inline]
    #[must_use]
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

impl Color {
    /// Converts a linear color to an sRGB color
    #[must_use]
    pub fn linear_to_srgb(color: Self) -> Self {
        // The sRGB transfer function (IEC 61966-2-1): a linear segment near black, a 2.4 power curve above it.
        // Values are clamped to the displayable range first, as the surface can't store anything outside it
        let as_vec = color.as_vec4();
        let colors_simd = as_vec.xyz().to_vec3a();
        let channel = colors_simd.clamp(Vec3A::ZERO, Vec3A::ONE);

        let in_linear_range = channel.cmple(Vec3A::splat(0.003_130_8));

        // Both branches are computed for every channel; the mask then picks one per channel
        let linear = channel * 12.92;
        let curved = 1.055 * channel.powf(1.0 / 2.4) - 0.055;

        // Alpha is not a colour channel and stays linear
        Color(Vec3A::select(in_linear_range, linear, curved).extend(color.a()))
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
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
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
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

#[cfg(test)]
mod test {
    use super::Color;
    use super::SerializedColor;
    use crate::Vec3;
    use crate::Vec3A;
    use crate::Vec4;

    /// How far two floats may differ and still count as equal, for results of float arithmetic
    const TOLERANCE: f32 = 1e-6;

    /// Asserts that every component of `actual` is within [`TOLERANCE`] of `expected`
    #[track_caller]
    fn assert_close(actual: Color, expected: Color, what: &str) {
        let difference = (actual.as_vec4() - expected.as_vec4()).abs();

        assert!(
            difference.max_element() <= TOLERANCE,
            "{what}: expected {expected}, got {actual}"
        );
    }

    /// The constructor stores each component in its own slot, and the accessors read them back
    #[test]
    fn new_stores_components() {
        let color = Color::new(0.1, 0.2, 0.3, 0.4);

        assert_eq!(color.r(), 0.1, "red");
        assert_eq!(color.g(), 0.2, "green");
        assert_eq!(color.b(), 0.3, "blue");
        assert_eq!(color.a(), 0.4, "alpha");
        assert_eq!(
            color.as_vec4(),
            Vec4::new(0.1, 0.2, 0.3, 0.4),
            "as a vector"
        );
    }

    /// Byte 0 maps to 0.0 and byte 255 to 1.0, per component
    #[test]
    fn bytes_map_endpoints() {
        assert_eq!(
            Color::new_from_bytes(0, 255, 0, 255),
            Color::new(0.0, 1.0, 0.0, 1.0),
            "bytes 0 and 255"
        );
    }

    /// The long hex forms read two digits per component, with alpha optional
    #[test]
    fn hex_long_forms() {
        assert_eq!(
            Color::hex("#12AB9F"),
            Some(Color::new_from_bytes(0x12, 0xAB, 0x9F, 0xFF)),
            "six digits, alpha defaults to opaque"
        );
        assert_eq!(
            Color::hex("#12AB9F40"),
            Some(Color::new_from_bytes(0x12, 0xAB, 0x9F, 0x40)),
            "eight digits"
        );
        assert_eq!(
            Color::hex("12ab9f"),
            Color::hex("#12AB9F"),
            "without `#` and in lowercase"
        );
    }

    /// The short hex forms double each digit, as CSS does
    #[test]
    fn hex_short_forms_double_digits() {
        assert_eq!(Color::hex("#F0A"), Color::hex("#FF00AA"), "three digits");
        assert_eq!(Color::hex("#F0A8"), Color::hex("#FF00AA88"), "four digits");
        assert_eq!(Color::hex("#000"), Some(Color::BLACK), "black");
        assert_eq!(Color::hex("#FFF"), Some(Color::WHITE), "white");
    }

    /// Whitespace around the whole code is ignored, but not between the `#` and the digits
    #[test]
    fn hex_whitespace() {
        assert_eq!(
            Color::hex("  #ABC  "),
            Color::hex("#ABC"),
            "around the code"
        );
        assert_eq!(
            Color::hex("\t#ABC\n"),
            Color::hex("#ABC"),
            "tabs and newlines"
        );
        assert_eq!(Color::hex("# ABC"), None, "after the `#`");
    }

    /// Codes with invalid digits or lengths are rejected
    #[test]
    fn hex_invalid() {
        for invalid in [
            "",
            "#",
            "#F",
            "#FF",
            "#FFFFF",
            "#FFFFFFF",
            "#FFFFFFFFF",
            "#GGG",
            "#12345Z",
            "#ÿÿÿ",
            "##FFF",
        ] {
            assert_eq!(Color::hex(invalid), None, "{invalid:?}");
        }
    }

    /// The named constants have the documented components
    #[test]
    fn constants() {
        assert_eq!(Color::ZERO, Color::new(0.0, 0.0, 0.0, 0.0), "zero");
        assert_eq!(Color::ONE, Color::new(1.0, 1.0, 1.0, 1.0), "one");
        assert_eq!(Color::BLACK, Color::new(0.0, 0.0, 0.0, 1.0), "black");
        assert_eq!(Color::WHITE, Color::ONE, "white");
        assert_eq!(Color::RED, Color::new(1.0, 0.0, 0.0, 1.0), "red");
        assert_eq!(Color::GREEN, Color::new(0.0, 1.0, 0.0, 1.0), "green");
        assert_eq!(Color::BLUE, Color::new(0.0, 0.0, 1.0, 1.0), "blue");
    }

    /// Conversions from three components give an opaque color; from four, they keep the alpha
    #[test]
    fn from_floats() {
        let opaque = Color::new(0.25, 0.5, 0.75, 1.0);
        let translucent = Color::new(0.25, 0.5, 0.75, 0.5);

        assert_eq!(Color::from(Vec3::new(0.25, 0.5, 0.75)), opaque, "from Vec3");
        assert_eq!(
            Color::from(Vec3A::new(0.25, 0.5, 0.75)),
            opaque,
            "from Vec3A"
        );
        assert_eq!(Color::from((0.25, 0.5, 0.75)), opaque, "from a 3-tuple");
        assert_eq!(
            Color::from(Vec4::new(0.25, 0.5, 0.75, 0.5)),
            translucent,
            "from Vec4"
        );
        assert_eq!(
            Color::from((0.25, 0.5, 0.75, 0.5)),
            translucent,
            "from a 4-tuple"
        );
    }

    /// Conversions from bytes match the byte constructor; three bytes give an opaque color
    #[test]
    fn from_bytes() {
        let opaque = Color::new_from_bytes(10, 20, 30, 255);
        let translucent = Color::new_from_bytes(10, 20, 30, 40);

        assert_eq!(Color::from([10, 20, 30]), opaque, "from [u8; 3]");
        assert_eq!(Color::from(&[10, 20, 30]), opaque, "from &[u8; 3]");
        assert_eq!(
            Color::from((10u8, 20u8, 30u8)),
            opaque,
            "from a 3-tuple of bytes"
        );
        assert_eq!(Color::from([10, 20, 30, 40]), translucent, "from [u8; 4]");
        assert_eq!(Color::from(&[10, 20, 30, 40]), translucent, "from &[u8; 4]");
        assert_eq!(
            Color::from((10u8, 20u8, 30u8, 40u8)),
            translucent,
            "from a 4-tuple of bytes"
        );
    }

    /// sRGB encoding keeps black and white, and maps 50% linear grey to 188 of 255
    #[test]
    fn linear_to_srgb_known_values() {
        assert_close(Color::linear_to_srgb(Color::BLACK), Color::BLACK, "black");
        assert_close(Color::linear_to_srgb(Color::WHITE), Color::WHITE, "white");

        let grey = Color::linear_to_srgb(Color::new(0.5, 0.5, 0.5, 1.0));

        assert_close(
            grey,
            Color::new(0.735_357, 0.735_357, 0.735_357, 1.0),
            "50% grey",
        );
        assert_eq!((grey.r() * 255.0).round(), 188.0, "50% grey as a byte");
    }

    /// Values near black use the linear segment, each channel on its own
    #[test]
    fn linear_to_srgb_linear_segment() {
        let encoded = Color::linear_to_srgb(Color::new(0.001, 0.5, 0.002, 1.0));

        assert_close(
            encoded,
            Color::new(0.012_92, 0.735_357, 0.025_84, 1.0),
            "linear segment for red and blue, the curve for green",
        );
    }

    /// The two pieces of the curve meet at the threshold, without a jump
    #[test]
    fn linear_to_srgb_continuous() {
        let threshold = 0.003_130_8;

        let below = Color::linear_to_srgb(Color::new(threshold, 0.0, 0.0, 1.0)).r();
        let above = Color::linear_to_srgb(Color::new(threshold + 1e-6, 0.0, 0.0, 1.0)).r();

        assert!(
            (above - below).abs() < 1e-4,
            "jump at the threshold: {below} to {above}"
        );
    }

    /// Values outside 0 to 1 are clamped, and alpha is never changed
    #[test]
    fn linear_to_srgb_clamps_and_keeps_alpha() {
        let encoded = Color::linear_to_srgb(Color::new(-1.0, 2.0, 0.5, 0.25));

        assert_close(
            encoded,
            Color::new(0.0, 1.0, 0.735_357, 0.25),
            "clamped, alpha kept",
        );
    }

    /// `Display` and `Debug` show the four components in order
    #[test]
    fn formatting() {
        let color = Color::new(1.0, 0.0, 0.5, 0.25);

        assert_eq!(
            color.to_string(),
            "Color(r: 1, g: 0, b: 0.5, a: 0.25)",
            "Display"
        );
        assert_eq!(format!("{color:?}"), "Color(1.0, 0.0, 0.5, 0.25)", "Debug");
    }

    /// Converting to the serialized form and back gives the same color
    #[test]
    fn serialized_round_trip() {
        let color = Color::new(0.1, 0.2, 0.3, 0.4);

        assert_eq!(
            Color::from(SerializedColor::from(color)),
            color,
            "round trip"
        );
    }
}
