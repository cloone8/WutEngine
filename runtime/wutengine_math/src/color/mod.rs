//! Color related types and functions

use core::fmt::Debug;
use core::fmt::Display;
use core::fmt::Formatter;
use core::fmt::Result;

use glam::Vec4Swizzles;

use crate::Vec3A;
use crate::Vec4;

pub mod css;

/// A 32-bit-per-color linear RGBA color
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Color {
    /// Red
    pub r: f32,

    /// Green
    pub g: f32,

    /// Blue
    pub b: f32,

    /// Alpha
    pub a: f32,
}

impl Color {
    /// Creates an opaque color from linear components
    #[inline]
    #[must_use]
    pub const fn linear(r: f32, g: f32, b: f32) -> Self {
        Self::linear_rgba(r, g, b, 1.0)
    }

    /// Creates a color from linear components and an alpha
    #[inline]
    #[must_use]
    pub const fn linear_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Creates a color from a [`Vec4`] holding linear components, with the alpha in `w`
    #[inline]
    #[must_use]
    pub const fn from_linear_vec4(vec: Vec4) -> Self {
        let [r, g, b, a] = vec.to_array();

        Self::linear_rgba(r, g, b, a)
    }

    /// Creates an opaque color from sRGB-encoded components, as CSS and image editors use them
    #[inline]
    #[must_use]
    pub fn srgb(r: f32, g: f32, b: f32) -> Self {
        Self::srgba(r, g, b, 1.0)
    }

    /// Creates a color from sRGB-encoded components and an alpha. The alpha is not a colour channel and is used as
    /// given
    #[must_use]
    pub fn srgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self::linear_rgba(srgb_to_linear(r), srgb_to_linear(g), srgb_to_linear(b), a)
    }

    /// Creates an opaque color from sRGB-encoded bytes, as CSS and image editors use them
    #[inline]
    #[must_use]
    pub fn srgb_u8(r: u8, g: u8, b: u8) -> Self {
        Self::srgba_u8(r, g, b, u8::MAX)
    }

    /// Creates a color from sRGB-encoded bytes and an alpha byte. `0` maps to `0.0` and `255` to `1.0`; the alpha is
    /// not a colour channel and only scaled
    #[must_use]
    pub fn srgba_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::srgba(
            f32::from(r) / 255.0,
            f32::from(g) / 255.0,
            f32::from(b) / 255.0,
            f32::from(a) / 255.0,
        )
    }

    /// Returns the unconverted data as an array of floats
    #[inline]
    #[must_use]
    pub const fn as_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Returns the unconverted data as a [`Vec4`]
    #[inline]
    #[must_use]
    pub const fn as_vec4(self) -> Vec4 {
        Vec4::from_array(self.as_array())
    }

    /// Creates a color from a hex code in sRGB, as CSS reads it: `#RGB`, `#RGBA`, `#RRGGBB` or `#RRGGBBAA`, with
    /// the `#` optional and the alpha opaque when left out. If the hex code is invalid, returns [`None`]
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

        let [r, g, b, a] = rgba;

        Some(Self::srgba_u8(r, g, b, a))
    }
}

/// Decodes one sRGB-encoded channel to linear, with the exact sRGB transfer function (IEC 61966-2-1): a linear
/// segment near black, a 2.4 power curve above it. Not clamped, so values above 1 stay usable as HDR values
#[inline]
fn srgb_to_linear(encoded: f32) -> f32 {
    if encoded <= 0.040_45 {
        encoded / 12.92
    } else {
        ((encoded + 0.055) / 1.055).powf(2.4)
    }
}

/// Basic colors. The CSS named colors are in [`css`]
impl Color {
    /// Fully transparent: black with a zero alpha, as CSS `transparent`
    pub const TRANSPARENT: Self = Self::linear_rgba(0.0, 0.0, 0.0, 0.0);

    /// Opaque black
    pub const BLACK: Self = Self::linear(0.0, 0.0, 0.0);

    /// Opaque white
    pub const WHITE: Self = Self::linear(1.0, 1.0, 1.0);

    /// Pure red: only the red channel, at full intensity
    pub const RED: Self = Self::linear(1.0, 0.0, 0.0);

    /// Pure green: only the green channel, at full intensity. This is CSS [`lime`](css::LIME); CSS
    /// [`green`](css::GREEN) is darker
    pub const GREEN: Self = Self::linear(0.0, 1.0, 0.0);

    /// Pure blue: only the blue channel, at full intensity
    pub const BLUE: Self = Self::linear(0.0, 0.0, 1.0);
}

impl Color {
    /// Encodes this color to sRGB, returned as `[r, g, b, a]`. Not clamped: values outside 0 to 1 stay outside it,
    /// so HDR values survive. The alpha is returned unchanged
    #[must_use]
    pub fn to_srgb(self) -> [f32; 4] {
        // The sRGB transfer function (IEC 61966-2-1): a linear segment near black, a 2.4 power curve above it.
        // Negative values take the linear segment, so the power curve never sees them
        let channel = self.as_vec4().xyz().to_vec3a();

        let in_linear_range = channel.cmple(Vec3A::splat(0.003_130_8));

        // Both branches are computed for every channel; the mask then picks one per channel
        let linear = channel * 12.92;
        let curved = 1.055 * channel.powf(1.0 / 2.4) - 0.055;

        // Alpha is not a colour channel and stays linear
        Vec3A::select(in_linear_range, linear, curved)
            .extend(self.a)
            .to_array()
    }

    /// Encodes this color to sRGB bytes, returned as `[r, g, b, a]`. Each channel is clamped to 0 to 1 and rounded
    /// to the nearest byte, so bytes from [`Self::srgba_u8`] come back unchanged. The alpha is only scaled
    #[must_use]
    pub fn to_srgba_u8(self) -> [u8; 4] {
        let [r, g, b, a] = self.to_srgb();

        [r, g, b, a].map(|channel| (channel.clamp(0.0, 1.0) * 255.0).round() as u8)
    }

    /// Encodes this color as an sRGB hex code, as CSS writes it: `#rrggbb` when opaque, `#rrggbbaa` otherwise. Lossy
    /// in the same way as [`Self::to_srgba_u8`]: HDR values are clamped
    #[must_use]
    pub fn to_hex(self) -> String {
        let [r, g, b, a] = self.to_srgba_u8();

        if a == u8::MAX {
            format!("#{r:02x}{g:02x}{b:02x}")
        } else {
            format!("#{r:02x}{g:02x}{b:02x}{a:02x}")
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "Color(r: {}, g: {}, b: {}, a: {})",
            self.r, self.g, self.b, self.a
        )
    }
}

#[cfg(test)]
mod test {
    use super::Color;
    use crate::Vec4;

    /// How far two floats may differ and still count as equal, for results of float arithmetic
    const TOLERANCE: f32 = 1e-6;

    /// Asserts that every component of `actual` is within [`TOLERANCE`] of `expected`
    #[track_caller]
    fn assert_close(actual: [f32; 4], expected: [f32; 4], what: &str) {
        let difference = (Vec4::from_array(actual) - Vec4::from_array(expected)).abs();

        assert!(
            difference.max_element() <= TOLERANCE,
            "{what}: expected {expected:?}, got {actual:?}"
        );
    }

    #[test]
    fn check_layout() {
        let color = Color::linear_rgba(0.1, 0.2, 0.3, 0.4);

        assert_eq!(size_of::<f32>() * 4, size_of::<Color>(), "Unexpected size");
        assert_eq!(size_of::<[f32; 4]>(), size_of::<Color>(), "Unexpected size");

        assert_eq!(align_of::<f32>(), align_of::<Color>(), "Unexpected align");
        assert_eq!(
            align_of::<[f32; 4]>(),
            align_of::<Color>(),
            "Unexpected align"
        );

        let as_array = color.as_array();

        assert_eq!(color.r, as_array[0], "Mismatch R");
        assert_eq!(color.g, as_array[1], "Mismatch G");
        assert_eq!(color.b, as_array[2], "Mismatch B");
        assert_eq!(color.a, as_array[3], "Mismatch A");
    }

    /// The linear constructors store the components as given; without an alpha, the color is opaque
    #[test]
    fn linear_stores_components() {
        let color = Color::linear_rgba(0.1, 0.2, 0.3, 0.4);

        assert_eq!(color.as_array(), [0.1, 0.2, 0.3, 0.4], "with alpha");
        assert_eq!(
            Color::linear(0.1, 0.2, 0.3),
            Color::linear_rgba(0.1, 0.2, 0.3, 1.0),
            "opaque"
        );
        assert_eq!(
            color.as_vec4(),
            Vec4::new(0.1, 0.2, 0.3, 0.4),
            "as a vector"
        );
        assert_eq!(
            Color::from_linear_vec4(Vec4::new(0.1, 0.2, 0.3, 0.4)),
            color,
            "from a vector"
        );
    }

    /// The sRGB constructors decode the colour channels: 50% sRGB grey is about 21% linear
    #[test]
    fn srgb_decodes() {
        assert_close(
            Color::srgb(0.0, 1.0, 0.5).as_array(),
            [0.0, 1.0, 0.214_041_14, 1.0],
            "black, white and grey",
        );
        assert_eq!(
            Color::srgb(0.1, 0.2, 0.3),
            Color::srgba(0.1, 0.2, 0.3, 1.0),
            "opaque"
        );
    }

    /// Values near black use the linear segment of the curve
    #[test]
    fn srgb_linear_segment() {
        assert_close(
            Color::srgb(0.04, 0.0, 0.0).as_array(),
            [0.003_095_975, 0.0, 0.0, 1.0],
            "below the threshold",
        );
    }

    /// Decoding isn't clamped, so values above 1 stay usable as HDR values
    #[test]
    fn srgb_not_clamped() {
        let decoded = Color::srgb(1.5, 0.0, 0.0).r;

        assert!(
            (decoded - 2.537_155).abs() < 1e-5,
            "1.5 sRGB decodes to about 2.537, got {decoded}"
        );
    }

    /// Alpha is not a colour channel and is never decoded
    #[test]
    fn srgb_keeps_alpha() {
        assert_eq!(Color::srgba(0.5, 0.5, 0.5, 0.5).a, 0.5, "float alpha");
        assert_eq!(
            Color::srgba_u8(128, 128, 128, 128).a,
            128.0 / 255.0,
            "byte alpha, only scaled"
        );
    }

    /// The byte constructors decode like the float ones, with byte 0 as 0.0 and byte 255 as 1.0
    #[test]
    fn srgb_bytes() {
        assert_eq!(
            Color::srgb_u8(0, 255, 0),
            Color::linear(0.0, 1.0, 0.0),
            "bytes 0 and 255"
        );
        assert_eq!(
            Color::srgb_u8(128, 64, 32),
            Color::srgb(128.0 / 255.0, 64.0 / 255.0, 32.0 / 255.0),
            "same as the float constructor"
        );
        assert_eq!(
            Color::srgb_u8(10, 20, 30),
            Color::srgba_u8(10, 20, 30, 255),
            "opaque"
        );
    }

    /// Every byte survives decoding and encoding again, when encoding rounds to the nearest byte
    #[test]
    fn srgb_bytes_round_trip() {
        for byte in 0..=u8::MAX {
            let encoded = Color::srgb_u8(byte, byte, byte).to_srgb();

            assert_eq!((encoded[0] * 255.0).round(), f32::from(byte), "byte {byte}");
        }
    }

    /// The long hex forms read two digits per component, with alpha optional
    #[test]
    fn hex_long_forms() {
        assert_eq!(
            Color::hex("#12AB9F"),
            Some(Color::srgba_u8(0x12, 0xAB, 0x9F, 0xFF)),
            "six digits, alpha defaults to opaque"
        );
        assert_eq!(
            Color::hex("#12AB9F40"),
            Some(Color::srgba_u8(0x12, 0xAB, 0x9F, 0x40)),
            "eight digits"
        );
        assert_eq!(
            Color::hex("12ab9f"),
            Color::hex("#12AB9F"),
            "without `#` and in lowercase"
        );
    }

    /// Hex codes are sRGB, as in CSS: `#808080` is about 22% linear
    #[test]
    fn hex_is_srgb() {
        let grey = Color::hex("#808080").expect("valid hex code");

        assert_close(
            grey.as_array(),
            [0.215_860_5, 0.215_860_5, 0.215_860_5, 1.0],
            "mid grey",
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
        assert_eq!(
            Color::TRANSPARENT,
            Color::linear_rgba(0.0, 0.0, 0.0, 0.0),
            "transparent"
        );
        assert_eq!(Color::BLACK, Color::linear(0.0, 0.0, 0.0), "black");
        assert_eq!(Color::WHITE, Color::linear(1.0, 1.0, 1.0), "white");
        assert_eq!(Color::RED, Color::linear(1.0, 0.0, 0.0), "red");
        assert_eq!(Color::GREEN, Color::linear(0.0, 1.0, 0.0), "green");
        assert_eq!(Color::BLUE, Color::linear(0.0, 0.0, 1.0), "blue");
    }

    /// sRGB encoding keeps black and white, and maps 50% linear grey to 188 of 255
    #[test]
    fn to_srgb_known_values() {
        assert_close(Color::BLACK.to_srgb(), [0.0, 0.0, 0.0, 1.0], "black");
        assert_close(Color::WHITE.to_srgb(), [1.0, 1.0, 1.0, 1.0], "white");

        let grey = Color::linear(0.5, 0.5, 0.5).to_srgb();

        assert_close(grey, [0.735_357, 0.735_357, 0.735_357, 1.0], "50% grey");
        assert_eq!((grey[0] * 255.0).round(), 188.0, "50% grey as a byte");
    }

    /// Values near black use the linear segment, each channel on its own
    #[test]
    fn to_srgb_linear_segment() {
        let encoded = Color::linear(0.001, 0.5, 0.002).to_srgb();

        assert_close(
            encoded,
            [0.012_92, 0.735_357, 0.025_84, 1.0],
            "linear segment for red and blue, the curve for green",
        );
    }

    /// The two pieces of the curve meet at the threshold, without a jump
    #[test]
    fn to_srgb_continuous() {
        let threshold = 0.003_130_8;

        let below = Color::linear(threshold, 0.0, 0.0).to_srgb()[0];
        let above = Color::linear(threshold + 1e-6, 0.0, 0.0).to_srgb()[0];

        assert!(
            (above - below).abs() < 1e-4,
            "jump at the threshold: {below} to {above}"
        );
    }

    /// Encoding isn't clamped: negative values take the linear segment and values above 1 stay above 1. Alpha is
    /// never changed
    #[test]
    fn to_srgb_not_clamped_and_keeps_alpha() {
        let encoded = Color::linear_rgba(-0.001, 2.537_155, 0.5, 0.25).to_srgb();

        assert_close(
            [encoded[0], 0.0, encoded[2], encoded[3]],
            [-0.012_92, 0.0, 0.735_357, 0.25],
            "negative red, alpha kept",
        );
        assert!(
            (encoded[1] - 1.5).abs() < 1e-5,
            "2.537 linear encodes back to 1.5, got {}",
            encoded[1]
        );
    }

    /// Decoding and encoding are each other's inverse within 0 to 1
    #[test]
    fn srgb_round_trip() {
        let srgb = [0.0, 0.02, 0.25, 0.5, 0.75, 1.0];

        for value in srgb {
            let encoded = Color::srgb(value, value, value).to_srgb();

            assert_close(encoded, [value, value, value, 1.0], "round trip");
        }
    }

    /// `Display` and `Debug` show the four components in order
    #[test]
    fn formatting() {
        let color = Color::linear_rgba(1.0, 0.0, 0.5, 0.25);

        assert_eq!(
            color.to_string(),
            "Color(r: 1, g: 0, b: 0.5, a: 0.25)",
            "Display"
        );
        assert_eq!(
            format!("{color:?}"),
            "Color { r: 1.0, g: 0.0, b: 0.5, a: 0.25 }",
            "Debug"
        );
    }
}
