use glam::{Vec3, Vec4};
use wutengine_graphics::color::Color;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct GlVec3f {
    x: f32,
    y: f32,
    z: f32,
}

impl From<Vec3> for GlVec3f {
    fn from(value: Vec3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct GlVec4f {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

impl From<Vec4> for GlVec4f {
    fn from(value: Vec4) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
            w: value.w,
        }
    }
}

impl From<Color> for GlVec4f {
    fn from(value: Color) -> Self {
        Self {
            x: value.r,
            y: value.g,
            z: value.b,
            w: value.a,
        }
    }
}
