use glam::{Vec3, Vec4};

#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const BLACK: Self = Color::rgb(0.0, 0.0, 0.0);
    pub const WHITE: Self = Color::rgb(1.0, 1.0, 1.0);
    pub const RED: Self = Color::rgb(1.0, 0.0, 0.0);
    pub const GREEN: Self = Color::rgb(0.0, 1.0, 0.0);
    pub const BLUE: Self = Color::rgb(0.0, 0.0, 1.0);

    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

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
