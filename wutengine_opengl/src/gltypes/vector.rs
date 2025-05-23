use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3, Vec4};
use wutengine_graphics::color::Color;

/// A 2D OpenGL float vector
#[repr(C, align(8))] // Align = 2x base type
#[derive(Debug, Clone, Copy)]
pub(crate) struct GlVec2f {
    x: f32,
    y: f32,
}

unsafe impl Zeroable for GlVec2f {}
unsafe impl Pod for GlVec2f {}

impl From<Vec2> for GlVec2f {
    fn from(value: Vec2) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

/// A 3D OpenGL float vector
#[repr(C, align(16))] // Align = 4x base type, like vec4
#[derive(Debug, Clone, Copy)]
pub(crate) struct GlVec3f {
    x: f32,
    y: f32,
    z: f32,
}

unsafe impl Zeroable for GlVec3f {}
unsafe impl Pod for GlVec3f {}

impl From<Vec3> for GlVec3f {
    fn from(value: Vec3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

/// A 4D OpenGL float vector
#[repr(C, align(16))] // Align = 4x base type
#[derive(Debug, Clone, Copy)]
pub(crate) struct GlVec4f {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

unsafe impl Zeroable for GlVec4f {}
unsafe impl Pod for GlVec4f {}

impl From<Vec4> for GlVec4f {
    #[inline]
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
    #[inline]
    fn from(value: Color) -> Self {
        Self {
            x: value.r,
            y: value.g,
            z: value.b,
            w: value.a,
        }
    }
}
