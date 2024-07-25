use glam::{Vec3, Vec4};

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
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
