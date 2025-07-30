use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};
use wutengine_math::Vec4;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize, Pod, Zeroable)]
#[repr(C)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    pub const RED: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };

    pub const GREEN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };

    pub const BLUE: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };

    pub const YELLOW: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
}

impl From<Color> for wgpu::Color {
    #[inline(always)]
    fn from(value: Color) -> Self {
        Self {
            r: f64::from(value.r),
            g: f64::from(value.g),
            b: f64::from(value.b),
            a: f64::from(value.a),
        }
    }
}

impl Color {
    pub const fn from_vec4(v: Vec4) -> Self {
        let as_array = v.to_array();
        Self {
            r: as_array[0],
            g: as_array[1],
            b: as_array[2],
            a: as_array[3],
        }
    }

    pub const fn to_vec4(self) -> Vec4 {
        Vec4::new(self.r, self.g, self.b, self.a)
    }
}

impl From<Vec4> for Color {
    #[inline(always)]
    fn from(value: Vec4) -> Self {
        Self::from_vec4(value)
    }
}

impl From<Color> for Vec4 {
    #[inline(always)]
    fn from(value: Color) -> Self {
        value.to_vec4()
    }
}
