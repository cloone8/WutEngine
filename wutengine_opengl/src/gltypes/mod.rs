use glam::Vec3;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GlVertex {
    x: f32,
    y: f32,
    z: f32,
}

impl From<Vec3> for GlVertex {
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
pub struct GlColorRgb {
    r: f32,
    g: f32,
    b: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GlColorRgba {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GlTexCoord {
    u: f32,
    v: f32,
}
